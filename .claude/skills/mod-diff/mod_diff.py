"""Summarise what a mod changed, by resource type.

Thin layer over `iecli override-diff`. Two modes:

* Shadow report (no reference): every override resource that also exists in a
  BIFF is a *shadow* -- the live game differs from stock. `override_only`
  resources are mod additions with no stock version.
* Reference diff (`--against <dir-or-file>`): byte/hash compare the live
  override against a clean reference (e.g. a vanilla install's biff-extracted
  files) -> added / removed / changed. This is the day-one "what did the mod
  touch" map for a fresh mod.

Output buckets the results by resource type (DLG/ITM/CRE/ARE/...), so you can
scope the investigation to the delta instead of re-sweeping the whole game.
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from collections import defaultdict


def default_iecli() -> str:
    for profile in ("release", "debug"):
        for name in ("iecli.exe", "iecli"):
            cand = os.path.join("target", profile, name)
            if os.path.isfile(cand):
                return cand
    return os.path.join("target", "release", "iecli.exe")


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--game", required=True, help="modded game install directory")
    p.add_argument("--against", default=None, help="clean reference dir or file (Mode B)")
    p.add_argument("--type", default=None, help="filter to one resource type, e.g. DLG")
    p.add_argument("--iecli", default=None, help="path to iecli (default: target/release|debug)")
    p.add_argument("--json", action="store_true", help="pass through raw override-diff JSON")
    return p.parse_args()


def res_type(name: str) -> str:
    return name.rsplit(".", 1)[-1].upper() if "." in name else "?"


def entry_name(entry) -> str:
    if isinstance(entry, str):
        return entry
    if isinstance(entry, dict):
        for key in ("resource", "resref", "name", "path"):
            if key in entry:
                return str(entry[key])
    return str(entry)


def bucket(names) -> dict[str, list[str]]:
    out: dict[str, list[str]] = defaultdict(list)
    for n in names:
        name = entry_name(n)
        out[res_type(name)].append(name)
    return {k: sorted(v) for k, v in sorted(out.items())}


def print_buckets(title: str, names) -> None:
    b = bucket(names)
    total = sum(len(v) for v in b.values())
    print(f"\n=== {title} ({total}) ===")
    if not total:
        print("  none")
        return
    for rtype, items in b.items():
        head = ", ".join(items[:8]) + (f", +{len(items) - 8} more" if len(items) > 8 else "")
        print(f"  {rtype:<5} {len(items):>4}  {head}")


def main() -> int:
    args = parse_args()
    iecli = args.iecli or default_iecli()
    if not os.path.isfile(iecli):
        print(f"iecli not found at {iecli}; build with `cargo build --release`", file=sys.stderr)
        return 2

    cmd = [iecli, "override-diff", "--game", args.game, "--format", "json"]
    if args.against:
        cmd += ["--against", args.against]
    if args.type:
        cmd += ["--type", args.type]
    out = subprocess.run(cmd, capture_output=True, text=True)
    if out.returncode != 0:
        print(out.stderr or "override-diff failed", file=sys.stderr)
        return 1
    try:
        data = json.loads(out.stdout)
    except json.JSONDecodeError:
        print("could not parse override-diff JSON", file=sys.stderr)
        return 1

    if args.json:
        json.dump(data, sys.stdout, indent=2)
        sys.stdout.write("\n")
        return 0

    # Mode B (reference diff): added / removed / changed.
    if any(k in data for k in ("added", "removed", "changed")):
        print(f"# Reference diff vs {args.against}")
        print_buckets("ADDED by mod (not in reference)", data.get("added", []))
        print_buckets("CHANGED by mod (differs from reference)", data.get("changed", []))
        print_buckets("REMOVED (in reference, gone from install)", data.get("removed", []))
        return 0

    # Mode A (shadow report): override_only + shadows (identical flag).
    shadows = data.get("shadows", [])
    changed = [s for s in shadows if isinstance(s, dict) and s.get("identical") is False]
    benign = [s for s in shadows if isinstance(s, dict) and s.get("identical") is True]
    print("# Shadow report (override vs stock biff -- no reference given)")
    print_buckets("MOD-ADDED (override only, no stock version)", data.get("override_only", []))
    print_buckets("MOD-CHANGED (override shadows + differs from stock biff)", changed)
    print_buckets("benign shadows (override re-ships a byte-identical stock file)", benign)
    counts = data.get("counts")
    if counts:
        print(f"\ncounts: {json.dumps(counts)}")
    return 0


if __name__ == "__main__":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass
    raise SystemExit(main())
