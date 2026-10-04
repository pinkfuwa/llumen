#!/usr/bin/env python3
"""Validate a semantic commit message.

Usage:
  python3 check_commit_msg.py MSG_FILE     # e.g. as a commit-msg hook
  echo "feat: add x" | python3 check_commit_msg.py

Exit 0 if valid (warnings may still print), 1 on errors.
Override allowed types with env COMMIT_TYPES="feat,fix,...".
"""
import os
import re
import sys

DEFAULT_TYPES = "feat,fix,docs,style,refactor,test,chore,build,ci,perf,revert"
TYPES = [t.strip() for t in os.environ.get("COMMIT_TYPES", DEFAULT_TYPES).split(",") if t.strip()]
HEADER = re.compile(r"^(?P<type>[a-z]+)(\((?P<scope>[a-z0-9_./-]+)\))?(?P<bang>!)?: (?P<subject>.+)$")
PAST_TENSE = re.compile(r"^(added|fixed|removed|updated|changed|created|deleted|implemented|refactored|renamed|moved)\b", re.I)
THIRD_PERSON = re.compile(r"^(adds|fixes|removes|updates|changes|creates|deletes|implements|renames|moves)\b", re.I)
GERUND = re.compile(r"^[a-z]+ing\b", re.I)


def main() -> int:
    raw = open(sys.argv[1], encoding="utf-8").read() if len(sys.argv) > 1 else sys.stdin.read()
    lines = [l for l in raw.splitlines() if not l.startswith("#")]
    while lines and not lines[0].strip():
        lines.pop(0)
    if not lines:
        print("ERROR: empty commit message")
        return 1

    header = lines[0]
    if header.startswith(("Merge ", "Revert \"", "fixup! ", "squash! ")):
        return 0

    errors, warnings = [], []
    m = HEADER.match(header)
    if not m:
        errors.append(f"header must be '<type>(<scope>): <subject>', got: {header!r}")
    else:
        t, subj = m["type"], m["subject"]
        if t not in TYPES:
            errors.append(f"unknown type '{t}'; allowed: {', '.join(TYPES)}")
        if subj[0].isupper() and not subj.split()[0].isupper():
            warnings.append("start the subject lowercase")
        if subj.endswith("."):
            errors.append("no trailing period in the subject")
        first = subj.split()[0]
        if PAST_TENSE.match(first) or THIRD_PERSON.match(first) or (GERUND.match(first) and first.lower() not in {"bring", "string", "ring"}):
            warnings.append(f"use imperative present tense ('add', not '{first}')")
        if re.search(r"\band\b", subj):
            warnings.append("subject contains 'and' — is this more than one change? consider splitting")
    if len(header) > 72:
        errors.append(f"header is {len(header)} chars; keep it <= 72 (ideally <= 50)")
    elif len(header) > 50:
        warnings.append(f"header is {len(header)} chars; aim for <= 50")

    if len(lines) > 1 and lines[1].strip():
        errors.append("leave a blank line between header and body")
    for i, l in enumerate(lines[2:], start=3):
        if len(l) > 72 and not re.match(r"^\s*(https?://|\S+:\s*https?://)", l):
            warnings.append(f"line {i} is {len(l)} chars; wrap body at 72")
            break
    if m and m["bang"] is None and any(l.startswith("BREAKING CHANGE") for l in lines):
        warnings.append("BREAKING CHANGE footer present; consider '!' in the header too")

    for w in warnings:
        print(f"warning: {w}")
    for e in errors:
        print(f"ERROR: {e}")
    if not errors and not warnings:
        print("ok")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
