#!/usr/bin/env python3
"""SPEC-02 v3 §3 acceptance gate: docs/PORT_MANIFEST.md must list every file of
reference/shadcn-admin exactly once, with a valid status.

    python3 scripts/manifest_check.py          # completeness + validity, prints progress
    python3 scripts/manifest_check.py final    # additionally fails on any todo / in-progress row
"""
import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REF = ROOT / "reference" / "shadcn-admin"
MANIFEST = ROOT / "docs" / "PORT_MANIFEST.md"
STATUSES = {"todo", "in-progress", "done", "n/a"}
ROW = re.compile(r"^\|\s*`([^`]+)`\s*\|([^|]*)\|([^|]*)\|([^|]*)\|(.*)\|\s*$")


def main() -> int:
    final = len(sys.argv) > 1 and sys.argv[1] == "final"
    if not REF.is_dir():
        print(f"missing {REF.relative_to(ROOT)}: clone it first (see SPEC-02 §2)")
        return 2
    on_disk = {p.relative_to(REF).as_posix() for p in REF.rglob("*") if p.is_file()}

    rows, problems = {}, []
    for n, line in enumerate(MANIFEST.read_text().splitlines(), 1):
        m = ROW.match(line)
        if not m:
            continue
        path, kind, target, status, notes = (g.strip() for g in m.groups())
        if path in rows:
            problems.append(f"line {n}: duplicate row for {path}")
        rows[path] = status
        if status not in STATUSES:
            problems.append(f"line {n}: invalid status '{status}' for {path}")
        if status == "n/a" and not notes:
            problems.append(f"line {n}: n/a without a reason for {path}")
        if status != "n/a" and target in ("", "—", "?"):
            problems.append(f"line {n}: no target for {path}")

    for path in sorted(on_disk - rows.keys()):
        problems.append(f"missing from manifest: {path}")
    for path in sorted(rows.keys() - on_disk):
        problems.append(f"row for a file that does not exist: {path}")

    count = Counter(rows.values())
    open_rows = count["todo"] + count["in-progress"]
    total = len(rows) - count["n/a"]
    print(f"{len(on_disk)} reference files, {len(rows)} rows: "
          f"done {count['done']}/{total}, in-progress {count['in-progress']}, "
          f"todo {count['todo']}, n/a {count['n/a']}")
    if final and open_rows:
        problems.append(f"final check: {open_rows} rows still todo / in-progress")

    for p in problems:
        print("  ✗", p)
    print("manifest-check:", "FAILED" if problems else "ok")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
