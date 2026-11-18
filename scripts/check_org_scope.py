#!/usr/bin/env python3
"""M16.4: every SQL statement on a tenant table must mention `org_id`, or be in the reviewed
baseline (statements keyed by a primary key the caller already scoped, system jobs, …).

  python3 scripts/check_org_scope.py            # CI: fail on statements not in the baseline
  python3 scripts/check_org_scope.py --update   # after review: rewrite the baseline
"""
import hashlib, pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASELINE = ROOT / "scripts" / "org_scope_baseline.txt"
TABLES = ["users", "roles", "audit_logs", "notifications", "files", "api_tokens", "email_outbox"]
TABLE_RE = re.compile(r"\b(FROM|JOIN|UPDATE|INTO)\s+(" + "|".join(TABLES) + r")\b", re.I)
SQL_RE = re.compile(r'query(?:_as|_scalar)?!\(\s*(?:[A-Za-z_:]+\s*,\s*)?(r#"[\s\S]*?"#|"(?:[^"\\]|\\.)*")', re.M)


def statements():
    for path in sorted((ROOT / "src" / "server").rglob("*.rs")):
        text = path.read_text()
        # Tests build their own fixtures.
        text = text.split("#[cfg(test)]")[0]
        for m in SQL_RE.finditer(text):
            sql = m.group(1)
            if TABLE_RE.search(sql) and "org_id" not in sql:
                norm = " ".join(sql.split())
                key = hashlib.sha1(norm.encode()).hexdigest()[:12]
                yield f"{path.relative_to(ROOT)} {key} {norm[:110]}"


found = sorted(set(statements()))
if "--update" in sys.argv:
    BASELINE.write_text("\n".join(found) + "\n")
    print(f"baseline: {len(found)} reviewed statements")
    sys.exit(0)
known = {" ".join(l.split()[:2]) for l in BASELINE.read_text().splitlines() if l.strip()} if BASELINE.exists() else set()
new = [f for f in found if " ".join(f.split()[:2]) not in known]
if new:
    print("SQL on tenant tables without org_id (add the filter, or review and run --update):")
    print("\n".join("  " + n for n in new))
    sys.exit(1)
print(f"org scope ok ({len(found)} reviewed statements)")
