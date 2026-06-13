"""Describe each room of an IE dungeon walked from a starting ARE.

Walks Travel regions in DFS order with side-rooms inline, prints a
per-room narrative (WED, area-script status, exits, traps, actor
tally), and flags faction transitions between consecutive rooms.
"""
from __future__ import annotations

import argparse
import collections
import json
import os
import re
import subprocess
import sys


# Faction classification: ordered (first match wins). Use word-boundary regex.
# Tweak as new mods introduce new creature names.
FACTIONS: list[tuple[str, re.Pattern[str]]] = [
    ("undead",    re.compile(r"\b(Shadow|Spectre|Specter|Wraith|Wight|Skeleton|Vampiric|Zombie|Ghoul|Ghast|Lich|Mummy|Revenant)\b", re.I)),
    ("drow",      re.compile(r"\b(Drow|Auvryndar|Jabress)\b", re.I)),
    ("duergar",   re.compile(r"\bDuergar\b", re.I)),
    ("gnoll",     re.compile(r"\b(Gnoll|Flind)\b", re.I)),
    ("spider",    re.compile(r"\bSpider\b", re.I)),
    ("bugbear",   re.compile(r"\bBugbear\b", re.I)),
    ("orc",       re.compile(r"\bOrc\b", re.I)),
    ("hobgoblin", re.compile(r"\bHobgoblin\b", re.I)),
    ("ogre",      re.compile(r"\b(Ogre|Half Ogre)\b", re.I)),
    ("basilisk",  re.compile(r"\bBasilisk\b", re.I)),
    ("bandit",    re.compile(r"\b(Bandit|Brigand|Blacktalon|Black Talon)\b", re.I)),
    ("amnian",    re.compile(r"\bAmnian\b", re.I)),
    ("bat",       re.compile(r"\bBat\b", re.I)),
    ("slave",     re.compile(r"\bSlave\b", re.I)),
    ("dog",       re.compile(r"\bDog\b", re.I)),
    ("miner",     re.compile(r"\bMiner\b", re.I)),
]

TRANSITION_THRESHOLD = 0.30  # 30% of room actors


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--game", required=True)
    p.add_argument("--start", required=True)
    p.add_argument(
        "--iecli",
        default=os.path.join("target", "debug", "iecli.exe"),
    )
    p.add_argument("--max-depth", type=int, default=None)
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


def dfs_order(iecli: str, game: str, start: str, max_depth: int | None) -> list[str]:
    """DFS traversal preserving Travel-region order so side-rooms appear inline."""
    start = start.upper()
    visited: set[str] = set()
    order: list[str] = []
    stack: list[tuple[str, int]] = [(start, 0)]
    while stack:
        cur, depth = stack.pop()
        if cur in visited:
            continue
        visited.add(cur)
        order.append(cur)
        if max_depth is not None and depth >= max_depth:
            continue
        a = dump_are(iecli, game, cur)
        if a is None:
            continue
        # Reverse so the first listed Travel exit is processed first (DFS via stack).
        for r in reversed(a.get("regions", [])):
            if (r.get("region_type") or {}).get("decoded") != "Travel":
                continue
            dest = r.get("destination_area")
            if not dest or not dest.get("exists"):
                continue
            d = dest["resref"].upper()
            if d not in visited:
                stack.append((d, depth + 1))
    return order


def actor_display_name(ac: dict) -> str:
    cre = ac.get("cre_file") or {}
    short = (cre.get("short_name") or {}).get("text")
    return short or ac.get("name") or "?"


def classify(name: str) -> str:
    for label, pattern in FACTIONS:
        if pattern.search(name):
            return label
    return "other"


def faction_breakdown(actors: list[dict]) -> dict[str, int]:
    counts: collections.Counter[str] = collections.Counter()
    for ac in actors:
        counts[classify(actor_display_name(ac))] += 1
    return dict(counts)


def named_bosses(actors: list[dict]) -> list[str]:
    """Actors whose display name doesn't fall into any common faction (treated as named/unique)."""
    out: list[str] = []
    seen: set[str] = set()
    for ac in actors:
        name = actor_display_name(ac)
        if classify(name) != "other":
            continue
        if name in seen:
            continue
        seen.add(name)
        out.append(name)
    return out


def transition_lines(prev: dict[str, int] | None, cur: dict[str, int]) -> list[str]:
    """Return TRANSITION lines describing faction shifts vs the previous room.

    Flags a transition when:
      - the dominant faction changes (single most-numerous, ignoring 'other')
      - a faction crossing TRANSITION_THRESHOLD appears that wasn't previously present
      - a faction previously above TRANSITION_THRESHOLD has disappeared
    """
    if prev is None:
        return []
    cur_total = sum(cur.values()) or 1
    prev_total = sum(prev.values()) or 1

    def dominant(d: dict[str, int]) -> str | None:
        ranked = [(c, n) for n, c in d.items() if n != "other"]
        ranked.sort(reverse=True)
        return ranked[0][1] if ranked else None

    notes: list[str] = []
    pd, cd = dominant(prev), dominant(cur)
    if pd != cd and (pd or cd):
        notes.append(f"dominant faction: {pd or 'none'} -> {cd or 'none'}")

    prev_major = {f for f, n in prev.items() if f != "other" and n / prev_total >= TRANSITION_THRESHOLD}
    cur_major = {f for f, n in cur.items() if f != "other" and n / cur_total >= TRANSITION_THRESHOLD}
    appeared = sorted(cur_major - prev_major)
    disappeared = sorted(prev_major - cur_major)
    if appeared:
        notes.append(f"new faction(s): {', '.join(appeared)}")
    if disappeared:
        notes.append(f"faction(s) gone: {', '.join(disappeared)}")

    if notes:
        return ["*** TRANSITION: " + "; ".join(notes) + " ***"]
    return []


def trap_summary(regions: list[dict]) -> list[str]:
    out: list[str] = []
    for r in regions:
        rt = (r.get("region_type") or {}).get("decoded")
        if rt != "Trigger":
            continue
        sc = (r.get("region_script") or {}).get("resource_name")
        if sc:
            out.append(f"{r['name']!r} -> {sc}")
        else:
            out.append(r["name"])
    return out


def describe(resref: str, a: dict, prev_factions: dict[str, int] | None) -> dict[str, int]:
    h = a["header"]
    ds = a["deferred_sections"]
    travel = [r for r in a["regions"] if (r["region_type"]["decoded"] or "") == "Travel"]
    triggers = [r for r in a["regions"] if (r["region_type"]["decoded"] or "") == "Trigger"]
    info = [r for r in a["regions"] if (r["region_type"]["decoded"] or "") == "Info"]

    factions = faction_breakdown(a["actors"])
    for line in transition_lines(prev_factions, factions):
        print(line)

    print(f"\n=== {resref} ===")
    wed = h.get("wed_resource")
    script = h["area_script"]
    print(
        f"  wed={wed}  area_script={script['resource_name']} "
        f"(exists={script['exists']})"
    )
    print(
        f"  area_type=0x{h['area_type_flags']['raw']:x} "
        f"{h['area_type_flags']['decoded']}"
    )
    print(
        f"  counts: actors={len(a['actors'])} regions={len(a['regions'])} "
        f"containers={ds['containers_count']} doors={ds['doors_count']} "
        f"spawn_points={ds['spawn_points_count']} entrances={ds['entrances_count']} "
        f"ambients={ds['ambients_count']} animations={ds['animations_count']}"
    )

    print("  exits:")
    if not travel:
        print("    (none)")
    for r in travel:
        dest = r.get("destination_area") or {}
        bb = r["bounding_box"]
        flags = r["flags"]["decoded"]
        flag_str = f" flags={flags}" if flags else ""
        sc = (r.get("region_script") or {}).get("resource_name")
        sc_str = f" script={sc}" if sc else ""
        marker = "" if dest.get("exists") else "  [DEAD]"
        print(
            f"    {r['name']!r:18} -> {dest.get('resource_name','?'):14} "
            f"entrance={r.get('destination_entrance')!r:18} "
            f"box=({bb['top_left']['x']},{bb['top_left']['y']})-"
            f"({bb['bottom_right']['x']},{bb['bottom_right']['y']})"
            f"{flag_str}{sc_str}{marker}"
        )

    if triggers:
        print(f"  traps/triggers ({len(triggers)}):")
        for line in trap_summary(triggers):
            print(f"    {line}")
    if info:
        print(f"  info points ({len(info)}):")
        for r in info:
            print(f"    {r['name']!r}")

    if a["actors"]:
        # Tally by short_name
        tally = collections.Counter()
        for ac in a["actors"]:
            tally[actor_display_name(ac)] += 1
        roster = ", ".join(f"{n}x{c}" for n, c in tally.most_common())
        print(f"  actors ({len(a['actors'])}): {roster}")
        print(f"  factions: {factions}")
        bosses = named_bosses(a["actors"])
        if bosses:
            print(f"  named/unique: {bosses}")
        actor_scripts = sorted(
            {
                slot["resource_name"]
                for ac in a["actors"]
                for slot in (ac.get("scripts") or {}).values()
                if slot and slot.get("resource_name")
            }
        )
        if actor_scripts:
            print(f"  actor scripts: {actor_scripts}")
        actors_with_dialog = [
            actor_display_name(ac) for ac in a["actors"] if ac.get("dialog")
        ]
        if actors_with_dialog:
            print(f"  actors with dialog: {actors_with_dialog}")
    else:
        print("  actors: none")

    return factions


def main() -> int:
    args = parse_args()

    if not os.path.isfile(args.iecli):
        print(f"iecli not found at {args.iecli}; build with `cargo build`", file=sys.stderr)
        return 2

    order = dfs_order(args.iecli, args.game, args.start, args.max_depth)
    print(f"DFS traversal order: {order}\n")

    # Track WED reuse across the dungeon for a final note.
    wed_to_areas: dict[str, list[str]] = collections.defaultdict(list)

    prev_factions: dict[str, int] | None = None
    for resref in order:
        a = dump_are(args.iecli, args.game, resref)
        if a is None:
            print(f"\n=== {resref} === [PARSE FAILED]")
            continue
        wed = (a["header"].get("wed_resource") or "").upper()
        if wed:
            wed_to_areas[wed].append(resref)
        prev_factions = describe(resref, a, prev_factions)

    # Final pass: WED tileset reuse.
    print("\n=== WED tileset reuse ===")
    reused = {wed: areas for wed, areas in wed_to_areas.items() if len(areas) > 1}
    if reused:
        for wed, areas in sorted(reused.items()):
            print(f"  {wed}: {areas}")
    else:
        print("  (no WED reuse detected)")

    return 0


if __name__ == "__main__":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass
    raise SystemExit(main())
