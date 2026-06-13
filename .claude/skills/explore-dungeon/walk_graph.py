"""Walk an IE dungeon's area graph from a starting ARE.

Follows Travel regions via `iecli dump`, reports reachable areas, edges,
one-way exits, dead links, and orphans (Override-source AREs sharing a
prefix with the reached set but never reached).
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from collections import deque


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--game", required=True, help="game install directory")
    p.add_argument("--start", required=True, help="starting ARE resref (e.g. AR4300)")
    p.add_argument(
        "--iecli",
        default=os.path.join("target", "debug", "iecli.exe"),
        help="path to iecli executable (default: target/debug/iecli.exe)",
    )
    p.add_argument("--max-depth", type=int, default=None, help="limit BFS depth")
    p.add_argument("--json", action="store_true", help="emit JSON instead of text report")
    return p.parse_args()


def dump_are(iecli: str, game: str, resref: str) -> dict | None:
    out = subprocess.run(
        [iecli, "dump", "--game", game, "--resource", f"{resref}.ARE"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if out.returncode != 0:
        return None
    try:
        return json.loads(out.stdout)
    except json.JSONDecodeError:
        return None


def list_aes(iecli: str, game: str) -> list[dict]:
    out = subprocess.run(
        [iecli, "list", "--game", game, "--type", "are", "--format", "json"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if out.returncode != 0:
        return []
    try:
        return json.loads(out.stdout)
    except json.JSONDecodeError:
        return []


def walk(iecli: str, game: str, start: str, max_depth: int | None) -> dict:
    start = start.upper()
    visited: set[str] = set()
    edges: list[dict] = []  # {from, to, region, entrance, dest_exists}
    depth: dict[str, int] = {start: 0}
    queue: deque[str] = deque([start])
    parse_failures: list[str] = []

    while queue:
        cur = queue.popleft()
        if cur in visited:
            continue
        visited.add(cur)
        if max_depth is not None and depth[cur] >= max_depth:
            continue
        a = dump_are(iecli, game, cur)
        if a is None:
            parse_failures.append(cur)
            continue
        for r in a.get("regions", []):
            if (r.get("region_type") or {}).get("decoded") != "Travel":
                continue
            dest = r.get("destination_area")
            if not dest:
                continue
            dest_resref = dest["resref"].upper()
            edges.append(
                {
                    "from": cur,
                    "to": dest_resref,
                    "region": r["name"],
                    "entrance": r.get("destination_entrance"),
                    "dest_exists": dest.get("exists", False),
                }
            )
            if dest.get("exists") and dest_resref not in visited:
                if dest_resref not in depth:
                    depth[dest_resref] = depth[cur] + 1
                queue.append(dest_resref)

    return {
        "start": start,
        "reached": sorted(visited),
        "edges": edges,
        "parse_failures": parse_failures,
        "depth": depth,
    }


def find_orphans(all_aes: list[dict], reached: set[str]) -> list[dict]:
    """Override AREs that share a 3-char prefix with the reached set but were not reached.

    Each orphan is annotated with `_prefix` and `_prefix_popularity` (how many
    reached areas share that prefix), and the list is sorted so that prefixes
    claimed by many reached areas come first — the dungeon's own resref series
    surfaces above unrelated vanilla overrides.
    """
    prefix_count: dict[str, int] = {}
    for r in reached:
        if len(r) >= 3:
            prefix_count[r[:3]] = prefix_count.get(r[:3], 0) + 1

    out: list[dict] = []
    for a in all_aes:
        resref = a.get("resref", "").upper()
        if not resref or len(resref) < 3:
            continue
        if a.get("source_kind") != "Override":
            continue
        if resref in reached:
            continue
        prefix = resref[:3]
        if prefix not in prefix_count:
            continue
        annotated = dict(a)
        annotated["_prefix"] = prefix
        annotated["_prefix_popularity"] = prefix_count[prefix]
        out.append(annotated)

    out.sort(key=lambda a: (-a["_prefix_popularity"], a["resref"]))
    return out


def find_one_way(edges: list[dict]) -> list[tuple[str, str]]:
    pairs = {(e["from"], e["to"]) for e in edges}
    return sorted([(a, b) for (a, b) in pairs if (b, a) not in pairs])


def find_dead_links(edges: list[dict]) -> list[dict]:
    seen: set[tuple[str, str]] = set()
    out: list[dict] = []
    for e in edges:
        if e["dest_exists"]:
            continue
        key = (e["from"], e["to"])
        if key in seen:
            continue
        seen.add(key)
        out.append(e)
    return out


def emit_text(walk_result: dict, orphans: list[dict]) -> None:
    reached = walk_result["reached"]
    edges = walk_result["edges"]
    print(f"=== walked from {walk_result['start']} ===")
    print(f"reached {len(reached)} areas:")
    for r in reached:
        print(f"  {r}  (depth {walk_result['depth'].get(r, '?')})")

    unique_edges = sorted({(e["from"], e["to"]) for e in edges})
    print(f"\nedges ({len(unique_edges)}):")
    for a, b in unique_edges:
        # Find the region/entrance for the first occurrence.
        ex = next(e for e in edges if (e["from"], e["to"]) == (a, b))
        marker = "" if ex["dest_exists"] else "  [DEAD]"
        print(f"  {a} -> {b}  via {ex['region']!r} entrance={ex['entrance']!r}{marker}")

    one_way = find_one_way(edges)
    print(f"\none-way exits ({len(one_way)}):")
    for a, b in one_way:
        print(f"  {a} -> {b}  (no return)")

    dead = find_dead_links(edges)
    print(f"\ndead links ({len(dead)}):")
    for e in dead:
        print(f"  {e['from']} -> {e['to']}  via {e['region']!r}  [destination ARE missing]")

    if walk_result["parse_failures"]:
        print(f"\nparse failures ({len(walk_result['parse_failures'])}):")
        for r in walk_result["parse_failures"]:
            print(f"  {r}")

    print(f"\norphans ({len(orphans)}) -- Override AREs sharing prefix but unreached:")
    print("  (ranked by prefix popularity: prefixes claimed by more reached areas come first)")
    for o in orphans:
        pop = o.get("_prefix_popularity", 0)
        prefix = o.get("_prefix", "")
        print(f"  {o['resref']}  [{prefix}* shared by {pop} reached]  ({o.get('source_path', '?')})")
    if not orphans:
        print("  none")


def main() -> int:
    args = parse_args()

    if not os.path.isfile(args.iecli):
        print(f"iecli not found at {args.iecli}; build with `cargo build`", file=sys.stderr)
        return 2

    walk_result = walk(args.iecli, args.game, args.start, args.max_depth)
    all_aes = list_aes(args.iecli, args.game)
    orphans = find_orphans(all_aes, set(walk_result["reached"]))

    if args.json:
        json.dump(
            {
                **walk_result,
                "orphans": orphans,
                "one_way": find_one_way(walk_result["edges"]),
                "dead_links": find_dead_links(walk_result["edges"]),
            },
            sys.stdout,
            indent=2,
        )
        sys.stdout.write("\n")
    else:
        emit_text(walk_result, orphans)

    return 0


if __name__ == "__main__":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass
    raise SystemExit(main())
