"""Validate PILGRIM Studio records with Python 3.10+ (standard library only)."""
from __future__ import annotations

import argparse
from collections import Counter
from datetime import datetime
import hashlib
import json
from pathlib import Path
import re
from urllib.parse import unquote

STATUSES = {'proposed', 'approved', 'implemented', 'verified', 'accepted',
            'blocked', 'rejected', 'superseded', 'cancelled'}
SECTIONS = ('Status history', 'Objective (as approved)', 'Scope', 'Constraints',
            'Acceptance criteria', 'Decisions and rationale', 'Affected files and commits',
            'Verification performed', 'Review findings and disposition',
            'Unresolved issues', 'Handoff log', 'Final status')
# Exact historical event predating task IDs, correlated to TASK-0001 in a later event.
# This exception does not exempt future events or edits to the original event.
LEGACY_EVENT_SHA256 = '6d773e7df5e2f10fbead6ec3a7f57a74330b18353e0ca25c3653b1fa0fc00369'


def fingerprint(event: dict) -> str:
    return hashlib.sha256(json.dumps(event, sort_keys=True, ensure_ascii=False,
                                    separators=(',', ':')).encode()).hexdigest()


def status(text: str) -> str:
    match = re.match(r'([a-z]+)\b', text.strip())
    return match[1] if match else ''


def sections(text: str) -> dict[str, str]:
    matches = list(re.finditer(r'^## (.+?)\s*$', text, re.M))
    return {m[1]: text[m.end():matches[i+1].start() if i+1 < len(matches) else len(text)].strip()
            for i, m in enumerate(matches)}


def cells(line: str) -> list[str]:
    return [part.strip() for part in line.strip().strip('|').split('|')]


def validate(root: Path) -> list[str]:
    """Read only. Return actionable errors; an empty list means structural checks passed."""
    root = root.resolve()
    errors: list[str] = []

    def read(path: Path) -> str:
        try:
            return path.read_text(encoding='utf-8-sig')
        except (OSError, UnicodeError) as exc:
            errors.append(f'{path.relative_to(root)}: cannot read: {exc}')
            return ''

    backlog_path = root/'.studio/production/backlog.md'
    backlog = read(backlog_path)
    rows = [cells(line) for line in backlog.splitlines()
            if re.match(r'^\| TASK-\d{4} \|', line)]
    counts = Counter(row[0] for row in rows)
    for task, count in counts.items():
        if count > 1:
            errors.append(f'backlog: duplicate ID {task}')
    next_match = re.search(r'Next free ID:\s*TASK-(\d{4})', backlog)
    if not next_match or any(int(task[5:]) >= int(next_match[1]) for task in counts):
        errors.append('backlog: missing or overlapping Next free ID')
    if not rows:
        errors.append('backlog: no task rows found')
    row_by_id = {row[0]: row for row in rows}
    record_dir = root/'.studio/production/tasks'
    paths = sorted(record_dir.glob('TASK-*.md'))
    records: dict[str, Path] = {}
    for path in paths:
        match = re.fullmatch(r'(TASK-\d{4})-[a-z0-9-]+\.md', path.name)
        if not match:
            errors.append(f'{path.name}: invalid task record filename')
            continue
        task = match[1]
        if task in records:
            errors.append(f'{task}: duplicate record files')
        records[task] = path

    for task, path in records.items():
        text = read(path)
        parts = sections(text)
        for section in SECTIONS:
            if section not in parts or not parts[section]:
                errors.append(f'{task}: missing or empty section: {section}')
            if len(re.findall(r'^## '+re.escape(section)+r'\s*$', text, re.M)) > 1:
                errors.append(f'{task}: duplicate required section: {section}')
        if not re.match(rf'^# {task}\s+[—-]\s+\S', text):
            errors.append(f'{task}: heading ID/title mismatch')
        fields = {}
        for line in text.split('## Status history', 1)[0].splitlines():
            if line.startswith('|') and len(cells(line)) == 2:
                key, value = cells(line)
                fields[key] = value
        if fields.get('ID') != task:
            errors.append(f'{task}: metadata ID mismatch')
        for key in ['Status', 'Owner', 'Requested by']:
            if not fields.get(key) or fields[key] in {'TBD', '—'}:
                errors.append(f'{task}: missing metadata {key}')
        current = status(fields.get('Status', ''))
        if current not in STATUSES:
            errors.append(f'{task}: invalid status {current!r}')
        history = re.findall(r'^- .+? [—-] (' + '|'.join(sorted(STATUSES)) + r')\b',
                             parts.get('Status history', ''), re.M)
        if not history or history[-1] != current:
            errors.append(f'{task}: current status differs from latest dated history')
        if current in {'approved', 'implemented', 'verified', 'accepted'} and 'approved' not in history:
            errors.append(f'{task}: no recorded approval in status history')
        if current in {'verified', 'accepted'}:
            verification = [cells(line) for line in parts.get('Verification performed', '').splitlines()
                            if re.match(r'^\| V\d+ \|', line)]
            if not verification or any(len(row) < 6 or any(not c or c == 'TBD' for c in row[:6])
                                       for row in verification):
                errors.append(f'{task}: verified task needs complete verification rows')
            review = parts.get('Review findings and disposition', '')
            findings = [cells(line) for line in review.splitlines()
                        if re.match(r'^\| (?:R|A)\d+ \|', line)]
            if not findings and 'No findings' not in review:
                errors.append(f'{task}: verified task needs review findings/disposition or explicit No findings')
            elif any(len(row) < 4 or any(not c or c == 'TBD' for c in row[:4]) for row in findings):
                errors.append(f'{task}: review row needs finding, reviewer and disposition')
        row = row_by_id.get(task)
        if not row:
            errors.append(f'{task}: record has no backlog row')
        elif len(row) < 6:
            errors.append(f'{task}: incomplete backlog row')
        else:
            if status(row[2]) != current:
                errors.append(f'{task}: backlog status differs from record')
            target = re.search(r'\]\(([^)]+)\)', row[4])
            if not target or (backlog_path.parent/unquote(target[1])).resolve() != path.resolve():
                errors.append(f'{task}: backlog must link to its record')

    for row in rows:
        if len(row) < 6:
            errors.append(f'{row[0]}: incomplete backlog row')
        elif status(row[2]) not in STATUSES:
            errors.append(f'{row[0]}: invalid backlog status')
        elif row[0] not in records and status(row[2]) != 'proposed':
            errors.append(f'{row[0]}: non-proposed task has no record')

    events_path = root/'.studio/events.jsonl'
    for number, line in enumerate(read(events_path).splitlines(), 1):
        if not line.strip():
            errors.append(f'events line {number}: blank line')
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError as exc:
            errors.append(f'events line {number}: invalid JSON: {exc.msg}')
            continue
        if not isinstance(event, dict):
            errors.append(f'events line {number}: expected object')
            continue
        for key in ['at', 'agent', 'type', 'message']:
            if not isinstance(event.get(key), str) or not event[key].strip():
                errors.append(f'events line {number}: missing or invalid {key}')
        try:
            moment = datetime.fromisoformat(str(event.get('at', '')).replace('Z', '+00:00'))
            if moment.utcoffset() is None or moment.utcoffset().total_seconds() != 0:
                raise ValueError('timestamp must be UTC')
        except ValueError:
            errors.append(f'events line {number}: invalid UTC timestamp')
        task = event.get('task')
        if task is not None:
            if not isinstance(task, str) or task not in records:
                errors.append(f'events line {number}: unknown task record {task!r}')
        elif fingerprint(event) != LEGACY_EVENT_SHA256:
            # Session/installation events must explicitly declare that they are not task events.
            if not isinstance(event.get('scope'), str) or event['scope'] not in {'session', 'installation'}:
                errors.append(f'events line {number}: missing task ID (or explicit non-task scope)')

    # Validate destination files, including URL-encoded spaces. Anchor contents and remote
    # URLs are outside this structural check. Ignore fenced code examples.
    docs = list((root/'.studio').rglob('*.md')) + list((root/'.claude').rglob('*.md'))
    docs += [root/name for name in ['AGENTS.md', 'CLAUDE.md', 'PROJECT_RULES.md']]
    for path in docs:
        text = re.sub(r'^```.*?^```[^\n]*$', '', read(path), flags=re.M | re.S)
        for target in re.findall(r'\]\(([^)]+)\)', text):
            if re.match(r'[a-zA-Z][a-zA-Z0-9+.-]*:', target):
                continue
            target = unquote(target.split('#', 1)[0].strip('<>'))
            if not target:
                continue
            destination = root/target.lstrip('/') if target.startswith('/') else path.parent/target
            if not destination.exists():
                errors.append(f'{path.relative_to(root)}: broken file link {target}')

    for name in ['LEADERSHIP_HANDOFF.md', 'IMPLEMENTATION_REPORT.md']:
        path = root/'.studio'/name
        text = read(path)
        match = re.search(r'^Task:\s*(TASK-\d{4})', text, re.M)
        if match and match[1] not in records:
            errors.append(f'{name}: current summary names an unknown task')
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    errors = validate(args.root)
    for error in errors:
        print(f'ERROR: {error}')
    print(f'Studio record check: {len(errors)} error(s).')
    return 1 if errors else 0


if __name__ == '__main__':
    raise SystemExit(main())
