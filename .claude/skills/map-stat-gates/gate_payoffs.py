"""Rank stat-gated dialogue REPLIES by what they DO (payoff), not by count.

The histogram (gate_histogram.py) answers "how much content sits behind a
threshold". This answers the better question: *which gates are worth it* — by
joining every transition gated on CheckStat(<obj>,N,STAT) to its action_text and
journal_text and classifying the payoff.

Design: this surfaces the RAW action/journal per gated reply (and the full
trigger, so co-conditions/timing are visible) and applies only a COARSE category
as a sort key. The agent reads the actual payoffs and judges value — the script
does not pretend a SetGlobal is worthless or an item is priceless.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from collections import defaultdict

CHECK = re.compile(r"CheckStat(GT|LT)\(\s*([A-Za-z0-9_]+)\s*,\s*(-?\d+)\s*,\s*([A-Za-z_]+)\s*\)")
PSC = re.compile(r"PermanentStatChange\(\s*([A-Za-z0-9_]+)\s*,\s*([A-Z]+)\s*,\s*(RAISE|LOWER)\s*,\s*(\d+)")
XP = re.compile(r"(?:AddexperienceParty|GiveExperience\([^,]*,)\s*(\d+)")
DEFAULT_OBJECTS = {"protagonist", "player1"}
CORE_STATS = {"STR", "DEX", "CON", "INT", "WIS", "CHR", "MAXHITPOINTS", "LEVEL"}

# coarse categories, best-first
ORDER = ["STAT_CORE", "COMPANION", "XP_BIG", "ITEM", "QUEST", "SKILL", "XP_small",
         "STORY", "TRAVEL", "STATE", "FLAVOR"]
HIGH = {"STAT_CORE", "COMPANION", "XP_BIG", "ITEM", "QUEST"}


def default_iecli() -> str:
    for profile in ("release", "debug"):
        for name in ("iecli.exe", "iecli"):
            cand = os.path.join("target", profile, name)
            if os.path.isfile(cand):
                return cand
    return os.path.join("target", "release", "iecli.exe")


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--game", required=True)
    p.add_argument("--protagonist", default=None,
                   help="object to match (default: Protagonist or Player1; 'any' = no filter)")
    p.add_argument("--high-only", action="store_true", help="only print/emit high-value payoffs")
    p.add_argument("--iecli", default=None)
    p.add_argument("--json", action="store_true")
    return p.parse_args()


def run_json(iecli: str, args: list[str]):
    out = subprocess.run(
        [iecli, *args], capture_output=True, text=True, encoding="utf-8", errors="replace"
    )
    if out.returncode != 0 or out.stdout is None:
        return None
    try:
        return json.loads(out.stdout)
    except (json.JSONDecodeError, TypeError):
        return None


def classify(action: str, journal: str | None) -> str:
    a = action or ""
    m = PSC.search(a)
    if m:
        return "STAT_CORE" if m.group(2).upper() in CORE_STATS else "SKILL"
    if "JoinParty" in a:
        return "COMPANION"
    xs = [int(n) for n in XP.findall(a)]
    if xs:
        return "XP_BIG" if max(xs) >= 5000 else "XP_small"
    if re.search(r"CreateItem|GiveItem", a):
        return "ITEM"
    if journal:
        return "QUEST"
    if "StartCutScene" in a:
        return "STORY"
    if re.search(r"EscapeArea|MoveToArea|LeaveAreaLUA|MoveBetweenAreas", a):
        return "TRAVEL"
    return "STATE" if a.strip() else "FLAVOR"


def collect(iecli: str, game: str, obj_filter: set[str] | None):
    dlgs = [d["resource_name"] for d in (run_json(iecli, ["list", "--game", game, "--type", "DLG", "--format", "json"]) or [])]
    # dedup key -> record (collapses the same gated payoff reached by several replies)
    seen: dict[tuple, dict] = {}
    total = len(dlgs)
    for i, name in enumerate(dlgs):
        if i % 150 == 0:
            print(f"  ...{i}/{total} dialogues", file=sys.stderr)
        j = run_json(iecli, ["dump", "--game", game, "--resource", name, "--format", "json",
                             "--strings", "resolved"])
        if not j:
            continue
        for st in j.get("states", []):
            for tr in st.get("transitions", []):
                trig = tr.get("trigger_text") or ""
                if "CheckStat" not in trig:
                    continue
                jr = tr.get("journal_text")
                jr = jr.get("text") if isinstance(jr, dict) else jr
                jr = re.sub(r"\s+", " ", jr).strip() if jr else jr
                action = re.sub(r"\s+", " ", tr.get("action_text") or "").strip()
                for m in CHECK.finditer(trig):
                    if m.group(1) != "GT" or m.group(2).lower() not in (obj_filter or {m.group(2).lower()}):
                        continue
                    stat, need = m.group(4).upper(), int(m.group(3)) + 1
                    cat = classify(action, jr)
                    key = (name, stat, need, cat, action[:120])
                    rec = seen.get(key)
                    if rec:
                        rec["replies"] += 1
                        continue
                    seen[key] = {
                        "dlg": name.replace(".DLG", ""), "stat": stat, "need": need,
                        "category": cat, "action": action[:200],
                        "journal": (jr or "")[:120], "trigger": re.sub(r"\s+", " ", trig).strip()[:160],
                        "replies": 1,
                    }
    return list(seen.values())


def emit_text(records) -> None:
    by_stat = defaultdict(list)
    for r in records:
        by_stat[r["stat"]].append(r)
    for stat in sorted(by_stat, key=lambda s: -sum(1 for r in by_stat[s] if r["category"] in HIGH)):
        rs = by_stat[stat]
        counts = defaultdict(int)
        for r in rs:
            counts[r["category"]] += 1
        high = sum(counts[c] for c in HIGH)
        print(f"\n===== {stat}: {len(rs)} distinct gated payoffs | {high} high-value =====")
        print("  " + "  ".join(f"{c}={counts[c]}" for c in ORDER if counts[c]))
        for r in sorted([r for r in rs if r["category"] in HIGH],
                        key=lambda r: (ORDER.index(r["category"]), -r["need"])):
            jn = f"  📓 {r['journal']}" if r["journal"] else ""
            print(f"    [{r['category']:<9} >= {r['need']:>2}] {r['dlg']:<10} {r['action'][:80]}{jn}")


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

    records = collect(iecli, args.game, obj_filter)
    if args.high_only:
        records = [r for r in records if r["category"] in HIGH]

    if args.json:
        json.dump(records, sys.stdout, indent=2)
        sys.stdout.write("\n")
    else:
        emit_text(records)
    return 0


if __name__ == "__main__":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass
    raise SystemExit(main())
