"""Trace Infinity Engine script timers that gate a quest/companion event.

Given an NPC resref prefix (e.g. HEXXAT, JAN) or explicit resources, dumps the
matching DLG/BCS via `iecli`, then finds every `(Real)SetGlobalTimer` (the SET)
and `(Real)GlobalTimer(Not)Expired` (the CHECK), reporting each timer's
duration converted to real time, classified game-time vs real-time.

Why the game-vs-real split matters: `SetGlobalTimer` runs on the game clock, so
resting and fast-travel skip it; `RealSetGlobalTimer` runs on wall-clock seconds
that keep counting while paused and are NOT skippable. The same numeric constant
means very different waits in each (FOUR_HOURS = 1200 = 4 game hours OR 1200 real
seconds ≈ 20 min). See reference: IE script timer semantics.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile

# Fallback game-time constants (1 game day = 7200 units, 1 game hour = 300).
# The real map is loaded from the game's GTIMES.IDS at runtime; this only
# covers the common names in case that extraction fails.
GTIMES = {
    "ONE_HOUR": 300, "FOUR_HOURS": 1200, "ONE_DAY": 7200,
    "FIVE_DAYS": 36000, "SEVEN_DAYS": 50400,
}
SCOPES = ("GLOBAL", "LOCALS", "MYAREA", "KAPUTZ")
SET_RE = re.compile(r'(Real)?SetGlobalTimer\("([^"]*)","([^"]*)",([^)]+)\)')
CHK_RE = re.compile(r'(Real)?GlobalTimer(Not)?Expired\("([^"]*)","([^"]*)"\)')


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--game", required=True, help="game install directory")
    p.add_argument(
        "--prefix",
        action="append",
        default=[],
        help="NPC resref prefix to scan, e.g. HEXXAT or JAN (repeatable)",
    )
    p.add_argument(
        "--resource",
        action="append",
        default=[],
        help="explicit resource to scan, e.g. LISSA.DLG (repeatable)",
    )
    p.add_argument("--timer", help="only report timers whose name contains this substring")
    p.add_argument(
        "--iecli",
        default=os.path.join("target", "release", "iecli.exe"),
        help="path to iecli executable (default: target/release/iecli.exe)",
    )
    p.add_argument("--json", action="store_true", help="emit JSON instead of narrative")
    return p.parse_args()


def run_json(iecli: str, args: list[str]):
    out = subprocess.run([iecli, *args], capture_output=True, text=True)
    if out.returncode != 0:
        return None
    try:
        return json.loads(out.stdout)
    except json.JSONDecodeError:
        return None


def list_resources(iecli: str, game: str, rtype: str) -> list[dict]:
    return run_json(iecli, ["list", "--game", game, "--type", rtype, "--format", "json"]) or []


def load_gtimes(iecli: str, game: str) -> dict[str, int]:
    """Load the authoritative game-time constant map from the game's GTIMES.IDS.

    Falls back to the builtin GTIMES dict if extraction fails.
    """
    with tempfile.TemporaryDirectory() as tmp:
        path = os.path.join(tmp, "gtimes.ids")
        out = subprocess.run(
            [iecli, "dump-raw", "--game", game, "--resource", "GTIMES.IDS", "--output", path],
            capture_output=True, text=True,
        )
        if out.returncode != 0 or not os.path.isfile(path):
            return {}
        try:
            with open(path, encoding="utf-8", errors="ignore") as f:
                raw = f.read().replace("\x00", "")
        except OSError:
            return {}
    parsed: dict[str, int] = {}
    for line in raw.splitlines():
        m = re.match(r"\s*(-?\d+)\s+(\w+)\s*$", line)
        if m:
            parsed[m.group(2).upper()] = int(m.group(1))
    return parsed


def resolve_targets(iecli: str, game: str, prefixes: list[str], resources: list[str]) -> list[str]:
    """Return a deduped list of '<RESREF>.<TYPE>' targets to dump."""
    targets: set[str] = set()
    for r in resources:
        targets.add(r.upper())
    if prefixes:
        ups = [p.upper() for p in prefixes]
        for rtype in ("dlg", "bcs"):
            for item in list_resources(iecli, game, rtype):
                resref = (item.get("resref") or "").upper()
                if any(resref.startswith(p) for p in ups):
                    targets.add(f"{resref}.{item['type']}")
    return sorted(targets)


def split_scope_name(packed: str) -> tuple[str, str]:
    """BCS SetGlobalTimer packs scope+name into one string ('GLOBALOHH_x')."""
    for s in SCOPES:
        if packed.startswith(s):
            return s, packed[len(s):]
    m = re.match(r"(AR[0-9A-Za-z]{4})(.+)", packed)
    if m:
        return m.group(1), m.group(2)
    return "", packed


def duration_to_int(tok: str):
    tok = tok.strip()
    if re.fullmatch(r"-?\d+", tok):
        return int(tok)
    return GTIMES.get(tok.upper())


def convert(n: int, is_real: bool) -> str:
    if is_real:
        return f"{n} real seconds ≈ {n/60:.0f} min of actual play — NOT skippable by resting/travel"
    if n % 7200 == 0:
        return f"{n} game-seconds = {n//7200} game day(s) — rest/travel counts toward it"
    if n % 300 == 0:
        return f"{n} game-seconds = {n//300} game hour(s) — rest/travel counts toward it"
    return f"{n} game-seconds ≈ {n/7200:.2f} game days — rest/travel counts toward it"


def scan_dlg(dlg: dict) -> tuple[list[dict], list[dict]]:
    sets, checks = [], []
    blobs = []
    for key in ("actions", "state_triggers", "transition_triggers"):
        for item in dlg.get(key, []):
            blobs.append((key, item.get("text", "")))
    for key, text in blobs:
        for m in SET_RE.finditer(text):
            sets.append({
                "real": bool(m.group(1)), "name": m.group(2), "scope": m.group(3),
                "raw": m.group(4).strip(), "where": key, "context": text.strip(),
            })
        for m in CHK_RE.finditer(text):
            checks.append({
                "real": bool(m.group(1)), "negated": bool(m.group(2)),
                "name": m.group(3), "scope": m.group(4),
                "where": key, "context": text.strip(),
            })
    return sets, checks


def block_actions(block: dict) -> list[dict]:
    out = []
    for r in block.get("responses", []):
        out += r.get("actions", [])
    return out


def scan_bcs(bcs: dict) -> tuple[list[dict], list[dict]]:
    sets, checks = [], []
    for block in bcs.get("blocks", []):
        sibling_triggers = [t.get("name") for t in block.get("triggers", [])]
        for a in block_actions(block):
            nm = a.get("name") or ""
            if nm in ("SetGlobalTimer", "RealSetGlobalTimer"):
                scope, name = split_scope_name((a.get("string_args") or [""])[0])
                sets.append({
                    "real": nm.startswith("Real"), "name": name, "scope": scope,
                    "raw": str((a.get("int_args") or [0])[0]),  # duration = int_args[0] (post-fix)
                    "where": "bcs-action", "context": "",
                })
        for t in block.get("triggers", []):
            nm = t.get("name") or ""
            mt = re.fullmatch(r"(Real)?GlobalTimer(Not)?Expired", nm)
            if mt:
                sa = t.get("string_args") or ["", ""]
                # describe what the block does when the timer fires
                act_names = [a.get("name") for a in block_actions(block)]
                checks.append({
                    "real": bool(mt.group(1)), "negated": bool(mt.group(2)),
                    "name": sa[0], "scope": sa[1] if len(sa) > 1 else "",
                    "where": "bcs-trigger",
                    "context": "gated by: " + ", ".join(n for n in sibling_triggers if n)
                               + " -> " + ", ".join(n for n in act_names if n),
                })
    return sets, checks


def main() -> int:
    args = parse_args()
    if not os.path.isfile(args.iecli):
        print(f"iecli not found at {args.iecli}; build with `cargo build --release`", file=sys.stderr)
        return 2
    if not args.prefix and not args.resource:
        print("supply at least one --prefix or --resource", file=sys.stderr)
        return 2

    GTIMES.update(load_gtimes(args.iecli, args.game))
    targets = resolve_targets(args.iecli, args.game, args.prefix, args.resource)
    all_sets, all_checks = [], []
    scanned, failed = [], []
    for tgt in targets:
        dump = run_json(args.iecli, ["dump", "--game", args.game, "--resource", tgt])
        if dump is None:
            failed.append(tgt)
            continue
        scanned.append(tgt)
        if tgt.upper().endswith(".DLG"):
            s, c = scan_dlg(dump)
        else:
            s, c = scan_bcs(dump)
        for x in s + c:
            x["resource"] = tgt
        all_sets += s
        all_checks += c

    if args.timer:
        f = args.timer.lower()
        all_sets = [x for x in all_sets if f in x["name"].lower()]
        all_checks = [x for x in all_checks if f in x["name"].lower()]

    # group by timer name
    names = sorted({x["name"] for x in all_sets} | {x["name"] for x in all_checks})

    if args.json:
        json.dump({"scanned": scanned, "failed": failed,
                   "sets": all_sets, "checks": all_checks}, sys.stdout, indent=2)
        sys.stdout.write("\n")
        return 0

    def dedupe(items: list[dict], keys: tuple[str, ...]) -> list[tuple[dict, int]]:
        groups: dict[tuple, list[dict]] = {}
        for it in items:
            groups.setdefault(tuple(it.get(k) for k in keys), []).append(it)
        return [(g[0], len(g)) for g in groups.values()]

    print(f"scanned {len(scanned)} resource(s); {len(names)} timer(s) found")
    if failed:
        print(f"(could not dump: {', '.join(failed)})")
    for name in names:
        sets = [x for x in all_sets if x["name"] == name]
        checks = [x for x in all_checks if x["name"] == name]
        reals = {x["real"] for x in sets + checks}
        label = "MIXED real+game — likely a scripting inconsistency" if len(reals) > 1 \
            else "REAL-time" if True in reals else "game-time"
        print(f"\n=== {name} ({label}) ===")
        for s, count in dedupe(sets, ("resource", "real", "raw")):
            n = duration_to_int(s["raw"])
            conv = convert(n, s["real"]) if n is not None else f"(unrecognized duration token {s['raw']!r})"
            fn = "RealSetGlobalTimer" if s["real"] else "SetGlobalTimer"
            tag = f" (x{count})" if count > 1 else ""
            print(f"  SET   {fn}(...,{s['raw']}) in {s['resource']}{tag}")
            print(f"        -> {conv}")
        if not sets:
            print("  SET   (not found in scanned files — timer is set elsewhere; "
                  "dump that file or widen --prefix/--resource)")
        for c, count in dedupe(checks, ("resource", "where", "context")):
            neg = "NOT-expired " if c["negated"] else ""
            fn = ("Real" if c["real"] else "") + "GlobalTimer" + ("NotExpired" if c["negated"] else "Expired")
            tag = f" (x{count})" if count > 1 else ""
            print(f"  CHECK {neg}{fn} in {c['resource']}{tag}")
            if c["context"]:
                for line in c["context"][:400].splitlines():
                    print(f"        {line}")
    return 0


if __name__ == "__main__":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass
    raise SystemExit(main())
