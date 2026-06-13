"""Catalogue an Infinity Engine game's permanent stat economy + stat gear.

Two scans, both mechanical (the agent reconciles exclusivity and synthesises a
build from the output):

1. DLG actions -- PermanentStatChange(<obj>, STAT, RAISE|LOWER, N): the in-game
   permanent stat gains/losses and which dialogue grants each. (This is the
   PST mechanism; BG/IWD raise stats via items, picked up by scan 2.)
2. ITM / SPL effects -- ability-score opcodes (Strength/Dexterity/Constitution/
   Intelligence/Wisdom/Charisma), split into permanent (consumed) vs
   while-equipped (tattoos, rings). Relies on iecli decoding effect opcodes
   correctly for the game (PST has its own opcode table -- ensure that fix is in).

NOTE: counts here are RAW. Many dialogue grants are mutually-exclusive branches
of one conversation; the agent must read the trees (iecli dump <DLG>) to resolve
what is actually achievable in a single playthrough before summing.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from collections import defaultdict

PSC_RE = re.compile(
    r"PermanentStatChange\(\s*([A-Za-z0-9_]+)\s*,\s*([A-Z]+)\s*,\s*(RAISE|LOWER)\s*,\s*(\d+)\s*\)"
)
STAT_NAMES = {"Strength", "Dexterity", "Constitution", "Intelligence", "Wisdom", "Charisma"}
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
        help="object for PermanentStatChange (default: Protagonist or Player1; 'any' = no filter)",
    )
    p.add_argument("--no-spl", action="store_true", help="skip SPL scan (faster)")
    p.add_argument("--iecli", default=None, help="path to iecli (default: target/release|debug)")
    p.add_argument("--json", action="store_true", help="emit JSON instead of text report")
    return p.parse_args()


def run_json(iecli: str, args: list[str]):
    # iecli emits UTF-8; force it (Windows would otherwise decode as cp1252 and
    # crash on stray bytes in modded resource strings).
    out = subprocess.run(
        [iecli, *args], capture_output=True, text=True, encoding="utf-8", errors="replace"
    )
    if out.returncode != 0 or out.stdout is None:
        return None
    try:
        return json.loads(out.stdout)
    except (json.JSONDecodeError, TypeError):
        return None


def list_resrefs(iecli: str, game: str, rtype: str) -> list[str]:
    data = run_json(iecli, ["list", "--game", game, "--type", rtype, "--format", "json"]) or []
    return [d["resource_name"] for d in data]


def signed(v: int) -> int:
    return v - 4294967296 if v > 2147483647 else v


def scan_dialogue_grants(iecli, game, obj_filter):
    rows = []  # {dlg, stat, dir, amt}
    dlgs = list_resrefs(iecli, game, "DLG")
    total = len(dlgs)
    for i, name in enumerate(dlgs):
        if i % 100 == 0:
            print(f"  ...DLG {i}/{total}", file=sys.stderr)
        j = run_json(iecli, ["dump", "--game", game, "--resource", name, "--format", "json"])
        if not j:
            continue
        for entry in j.get("actions", []):
            text = entry.get("text")
            if not text or "PermanentStatChange" not in text:
                continue
            for m in PSC_RE.finditer(text):
                who = m.group(1).lower()
                if obj_filter is not None and who not in obj_filter:
                    continue
                rows.append({"dlg": name.replace(".DLG", ""), "stat": m.group(2),
                             "dir": m.group(3), "amt": int(m.group(4))})
    return rows


def scan_item_effects(iecli, game, rtype):
    rows = []  # {res, name, stat, amt, timing, permanent}
    refs = list_resrefs(iecli, game, rtype)
    total = len(refs)
    for i, name in enumerate(refs):
        if i % 150 == 0:
            print(f"  ...{rtype} {i}/{total}", file=sys.stderr)
        j = run_json(iecli, ["dump", "--game", game, "--resource", name, "--format", "json"])
        if not j:
            continue
        ident = (((j.get("header") or {}).get("identified_name") or {}).get("text")) or ""
        effects = list(j.get("global_effects") or [])
        for ab in j.get("abilities") or []:
            effects.extend(ab.get("effects") or [])
        for e in effects:
            dec = ((e.get("opcode") or {}).get("decoded")) or ""
            if dec not in STAT_NAMES:
                continue
            timing = ((e.get("timing") or {}).get("decoded")) or ""
            rows.append({
                "res": name.replace(f".{rtype}", ""), "name": ident, "stat": dec,
                "amt": signed(e.get("parameter1", 0)),
                "timing": timing, "permanent": "permanent" in timing.lower(),
            })
    return rows


def emit_text(grants, items) -> None:
    print("=== PERMANENT dialogue stat grants (RAW -- resolve exclusivity by reading the DLGs) ===")
    by_stat = defaultdict(list)
    for r in grants:
        by_stat[r["stat"]].append(r)
    for stat in sorted(by_stat, key=lambda s: -sum(x["amt"] for x in by_stat[s] if x["dir"] == "RAISE")):
        raises = [r for r in by_stat[stat] if r["dir"] == "RAISE"]
        lowers = [r for r in by_stat[stat] if r["dir"] == "LOWER"]
        print(f"\n  {stat}: +{sum(r['amt'] for r in raises)} naive (RAISE), "
              f"-{sum(r['amt'] for r in lowers)} (LOWER)")
        for r in sorted(raises, key=lambda x: x["dlg"]):
            print(f"    RAISE +{r['amt']}  [{r['dlg']}]")
        for r in sorted(lowers, key=lambda x: x["dlg"]):
            print(f"    LOWER -{r['amt']}  [{r['dlg']}]")

    print("\n=== ITEM/SPELL stat effects (equip = while-worn, permanent = consumed) ===")
    by_stat = defaultdict(list)
    for r in items:
        by_stat[r["stat"]].append(r)
    for stat in sorted(by_stat):
        print(f"\n  {stat}:")
        for r in sorted(by_stat[stat], key=lambda x: (-x["permanent"], -x["amt"], x["res"])):
            kind = "PERMANENT" if r["permanent"] else "equip"
            sign = f"+{r['amt']}" if r["amt"] >= 0 else str(r["amt"])
            label = r["name"] or r["res"]
            print(f"    {sign:>3} {stat:<12} {kind:<9} {label}  ({r['res']})")


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

    grants = scan_dialogue_grants(iecli, args.game, obj_filter)
    items = scan_item_effects(iecli, args.game, "ITM")
    if not args.no_spl:
        items += scan_item_effects(iecli, args.game, "SPL")

    if args.json:
        json.dump({"dialogue_grants": grants, "item_effects": items}, sys.stdout, indent=2)
        sys.stdout.write("\n")
    else:
        emit_text(grants, items)
    return 0


if __name__ == "__main__":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass
    raise SystemExit(main())
