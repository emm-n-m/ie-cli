"""Histogram Infinity Engine dialogue stat-checks by required value.

Scans every DLG's state/transition trigger text for CheckStat* checks on the
protagonist and reports, per stat, how many gated trigger-conditions require
each value -- i.e. which stat values unlock the most dialogue content. Works
for any IE game (BG / IWD / PST).

Mechanical only: this surfaces the histogram; the agent interprets which
breakpoints are worth hitting. The count is *distinct gated trigger conditions*
(not every player reply), which is a good proxy for "how much content sits
behind this threshold".
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from collections import defaultdict

# CheckStatGT(Object, N, STAT) / CheckStatLT(Object, N, STAT). The object name
# varies by game (Protagonist in PST, Player1 in BG/IWD); STAT is a STATS.IDS
# token (STR, DEX, CON, INT, WIS, CHR, ...).
CHECK_RE = re.compile(
    r"CheckStat(GT|LT)\(\s*([A-Za-z0-9_]+)\s*,\s*(-?\d+)\s*,\s*([A-Za-z_]+)\s*\)"
)
DEFAULT_OBJECTS = {"protagonist", "player1"}


def default_iecli() -> str:
    for profile in ("release", "debug"):
        for name in ("iecli.exe", "iecli"):
            cand = os.path.join("target", profile, name)
            if os.path.isfile(cand):
                return cand
    return os.path.join("target", "release", "iecli.exe")


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--game", required=True, help="game install directory")
    p.add_argument(
        "--protagonist",
        default=None,
        help="object name to match (default: Protagonist or Player1; 'any' = no filter)",
    )
    p.add_argument("--iecli", default=None, help="path to iecli (default: target/release|debug)")
    p.add_argument("--json", action="store_true", help="emit JSON instead of text report")
    return p.parse_args()


def run_json(iecli: str, args: list[str]):
    out = subprocess.run([iecli, *args], capture_output=True, text=True)
    if out.returncode != 0:
        return None
    try:
        return json.loads(out.stdout)
    except json.JSONDecodeError:
        return None


def list_resrefs(iecli: str, game: str, rtype: str) -> list[str]:
    data = run_json(iecli, ["list", "--game", game, "--type", rtype, "--format", "json"]) or []
    return [d["resource_name"] for d in data]


def collect(iecli: str, game: str, obj_filter: set[str] | None):
    dlgs = list_resrefs(iecli, game, "DLG")
    gt: dict[str, dict[int, int]] = defaultdict(lambda: defaultdict(int))   # need >= value
    lt: dict[str, dict[int, int]] = defaultdict(lambda: defaultdict(int))   # low-stat branch < value
    gt_dlgs: dict[str, set[str]] = defaultdict(set)
    total = len(dlgs)
    for i, name in enumerate(dlgs):
        if i % 100 == 0:
            print(f"  ...{i}/{total} dialogues", file=sys.stderr)
        j = run_json(iecli, ["dump", "--game", game, "--resource", name, "--format", "json"])
        if not j:
            continue
        for bucket in ("state_triggers", "transition_triggers"):
            for entry in j.get(bucket, []):
                text = entry.get("text")
                if not text or "CheckStat" not in text:
                    continue
                for m in CHECK_RE.finditer(text):
                    op = m.group(1)
                    who = m.group(2).lower()
                    val = int(m.group(3))
                    stat = m.group(4).upper()
                    if obj_filter is not None and who not in obj_filter:
                        continue
                    if op == "GT":
                        gt[stat][val + 1] += 1
                        gt_dlgs[stat].add(name)
                    else:
                        lt[stat][val] += 1
    return gt, lt, gt_dlgs, total


def emit_text(gt, lt, gt_dlgs) -> None:
    stats = sorted(gt.keys(), key=lambda s: -sum(gt[s].values()))
    for stat in stats:
        total_branches = sum(gt[stat].values())
        print(f"\n=== {stat} : {total_branches} up-gated branches across "
              f"{len(gt_dlgs[stat])} dialogues ===")
        for need in sorted(gt[stat]):
            print(f"  need >= {need:<3} : {gt[stat][need]:>4} branches")
    # low-stat branches are alternates you only see at LOW values; summarise.
    if lt:
        print("\n--- low-stat (CheckStatLT) branch counts, for reference ---")
        for stat in sorted(lt, key=lambda s: -sum(lt[s].values())):
            print(f"  {stat:<5} {sum(lt[stat].values()):>4} low-stat branches")


def main() -> int:
    args = parse_args()
    iecli = args.iecli or default_iecli()
    if not os.path.isfile(iecli):
        print(f"iecli not found at {iecli}; build with `cargo build --release`", file=sys.stderr)
        return 2

    if args.protagonist and args.protagonist.lower() in {"any", "all", "*"}:
        obj_filter = None
    elif args.protagonist:
        obj_filter = {args.protagonist.lower()}
    else:
        obj_filter = set(DEFAULT_OBJECTS)

    gt, lt, gt_dlgs, total = collect(iecli, args.game, obj_filter)

    if args.json:
        out = {
            "protagonist_filter": sorted(obj_filter) if obj_filter else "any",
            "dialogues_scanned": total,
            "gt": {s: dict(sorted(v.items())) for s, v in gt.items()},
            "lt": {s: dict(sorted(v.items())) for s, v in lt.items()},
            "gt_dialogue_counts": {s: len(d) for s, d in gt_dlgs.items()},
        }
        json.dump(out, sys.stdout, indent=2)
        sys.stdout.write("\n")
    else:
        emit_text(gt, lt, gt_dlgs)
    return 0


if __name__ == "__main__":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass
    raise SystemExit(main())
