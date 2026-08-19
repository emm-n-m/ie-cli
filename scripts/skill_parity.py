#!/usr/bin/env python3
"""Keep the Claude Code skill tree in sync with the canonical Codex packages.

`skills/` is the single source of truth (see AGENTS.md). `.claude/skills/` is a
mirror of it, differing only in the script paths quoted inside SKILL.md, because
each agent invokes the helpers from its own tree.

    python scripts/skill_parity.py --check    # verify (CI)
    python scripts/skill_parity.py --sync     # regenerate the mirror

Codex-only metadata (`agents/`) is not mirrored.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CANONICAL = REPO / "skills"
MIRROR = REPO / ".claude" / "skills"

# Directories inside a canonical skill package that are product-specific and so
# are deliberately absent from the mirror.
CANONICAL_ONLY = {"agents"}

# A bare `skills/` path reference belongs to the canonical tree; the mirror must
# point at its own copy. The lookbehind keeps an already-qualified
# `.claude/skills/` from being rewritten twice.
_SKILLS_PATH = re.compile(r"(?<![\w./-])skills/")


def to_mirror_text(text: str) -> str:
    """Rewrite canonical SKILL.md prose for the mirror tree."""
    return _SKILLS_PATH.sub(".claude/skills/", text)


def relevant_files(root: Path) -> set[Path]:
    """Package files under `root`, relative to it, ignoring caches."""
    found = set()
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        rel = path.relative_to(root)
        if any(part == "__pycache__" for part in rel.parts):
            continue
        if rel.suffix in {".pyc", ".pyo"}:
            continue
        found.add(rel)
    return found


def expected_mirror() -> dict[Path, bytes]:
    """The exact byte content the mirror tree should have."""
    expected: dict[Path, bytes] = {}
    for rel in relevant_files(CANONICAL):
        if rel.parts[1:2] and rel.parts[1] in CANONICAL_ONLY:
            continue
        source = (CANONICAL / rel).read_bytes()
        if rel.name == "SKILL.md":
            source = to_mirror_text(source.decode("utf-8")).encode("utf-8")
        expected[rel] = source
    return expected


def check() -> int:
    expected = expected_mirror()
    actual = relevant_files(MIRROR) if MIRROR.is_dir() else set()

    missing = sorted(set(expected) - actual)
    extra = sorted(actual - set(expected))
    differing = sorted(
        rel
        for rel in sorted(set(expected) & actual)
        if (MIRROR / rel).read_bytes() != expected[rel]
    )

    for rel in missing:
        print(f"missing from .claude/skills: {rel.as_posix()}")
    for rel in extra:
        print(f"not in canonical skills/:    {rel.as_posix()}")
    for rel in differing:
        print(f"out of sync with skills/:    {rel.as_posix()}")

    if missing or extra or differing:
        print(
            "\nskills/ is the source of truth; regenerate the mirror with:\n"
            "    python scripts/skill_parity.py --sync",
            file=sys.stderr,
        )
        return 1

    print(f"skill trees in sync ({len(expected)} files)")
    return 0


def sync() -> int:
    expected = expected_mirror()

    changed = 0
    for rel, content in sorted(expected.items()):
        dest = MIRROR / rel
        if dest.is_file() and dest.read_bytes() == content:
            continue
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(content)
        print(f"wrote   {(Path('.claude/skills') / rel).as_posix()}")
        changed += 1

    if MIRROR.is_dir():
        for rel in sorted(relevant_files(MIRROR) - set(expected)):
            (MIRROR / rel).unlink()
            print(f"removed {(Path('.claude/skills') / rel).as_posix()}")
            changed += 1
        # Drop directories the removals left behind.
        for path in sorted(MIRROR.rglob("*"), key=lambda p: -len(p.parts)):
            if path.is_dir() and not any(path.iterdir()):
                path.rmdir()

    print("mirror already up to date" if not changed else f"synced {changed} file(s)")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="verify the mirror matches")
    mode.add_argument("--sync", action="store_true", help="regenerate the mirror")
    args = parser.parse_args()

    if not CANONICAL.is_dir():
        print(f"canonical skills tree not found at {CANONICAL}", file=sys.stderr)
        return 2

    return check() if args.check else sync()


if __name__ == "__main__":
    raise SystemExit(main())
