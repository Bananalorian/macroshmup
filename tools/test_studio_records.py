"""Regression tests use isolated fixtures; never mutate the project's records."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_studio_records as check


class RecordTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.task = '.studio/production/tasks/TASK-0001-example.md'
        self.put('.studio/production/backlog.md', 'Next free ID: TASK-0003\n'
                 '| TASK-0001 | Example | verified | Codex | [record](tasks/TASK-0001-example.md) | Director |\n'
                 '| TASK-0002 | Future | proposed | — | not yet | Plan |\n')
        body = '# TASK-0001 — Example\n| ID | TASK-0001 |\n| Status | verified |\n| Owner | Codex |\n| Requested by | Director |\n'
        for section in check.SECTIONS:
            body += f'\n## {section}\n'
            body += {
                'Status history': '- 2026-10-16 — approved — Director: do it\n- 2026-10-16 — verified — checks below\n',
                'Verification performed': '| V1 | command | working tree | passed | retained output | 1 |\n',
                'Review findings and disposition': 'No findings (reviewed by Codex).\n',
            }.get(section, 'Recorded details.\n')
        self.put(self.task, body)
        self.event = dict(at='2026-10-16T13:00:00Z', agent='codex', type='review', message='Checked', task='TASK-0001')
        self.put('.studio/events.jsonl', json.dumps(self.event)+'\n')
        for name in ['LEADERSHIP_HANDOFF.md', 'IMPLEMENTATION_REPORT.md']:
            self.put('.studio/'+name, 'Task: TASK-0001\n')
        for name in ['AGENTS.md', 'CLAUDE.md', 'PROJECT_RULES.md']:
            self.put(name, 'Project instructions.\n')

    def put(self, path, text):
        dest = self.root/path
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(text, encoding='utf-8')

    def change(self, path, old, new):
        self.put(path, (self.root/path).read_text(encoding='utf-8').replace(old,new))

    def fails(self, fragment):
        self.assertTrue(any(fragment in error for error in check.validate(self.root)), check.validate(self.root))

    def test_valid_and_proposed_without_record(self):
        self.assertEqual(check.validate(self.root), [])

    def test_missing_required_section(self):
        self.change(self.task, '## Decisions and rationale', '## Other')
        self.fails('missing or empty section: Decisions and rationale')

    def test_empty_required_section(self):
        self.change(self.task, '## Constraints\nRecorded details.', '## Constraints\n')
        self.fails('missing or empty section: Constraints')

    def test_duplicate_backlog_id(self):
        self.change('.studio/production/backlog.md', 'TASK-0002', 'TASK-0001')
        self.fails('duplicate ID')

    def test_duplicate_record_id(self):
        self.put('.studio/production/tasks/TASK-0001-other.md', (self.root/self.task).read_text(encoding='utf-8'))
        self.fails('duplicate record files')

    def test_unknown_event_task(self):
        self.change('.studio/events.jsonl', 'TASK-0001', 'TASK-0099')
        self.fails('unknown task record')

    def test_event_cannot_reference_unstarted_proposal(self):
        self.change('.studio/events.jsonl', 'TASK-0001', 'TASK-0002')
        self.fails('unknown task record')

    def test_invalid_json_and_nonobject(self):
        for content in ['{broken}\n', '[]\n']:
            with self.subTest(content=content):
                self.put('.studio/events.jsonl', content)
                self.fails('events line 1')

    def test_missing_task_id(self):
        del self.event['task']
        self.put('.studio/events.jsonl', json.dumps(self.event)+'\n')
        self.fails('missing task ID')

    def test_explicit_session_event(self):
        del self.event['task']
        self.event['scope']='session'
        self.put('.studio/events.jsonl', json.dumps(self.event)+'\n')
        self.assertEqual(check.validate(self.root), [])

    def test_bad_timestamp(self):
        self.change('.studio/events.jsonl', '2026-10-16T13:00:00Z', 'yesterday')
        self.fails('invalid UTC timestamp')

    def test_wrong_metadata_id(self):
        self.change(self.task, '| ID | TASK-0001 |', '| ID | TASK-0002 |')
        self.fails('metadata ID mismatch')

    def test_status_mismatch(self):
        self.change('.studio/production/backlog.md', '| verified |', '| approved |')
        self.fails('backlog status differs')

    def test_missing_history(self):
        self.change(self.task, '— verified —', '— started —')
        self.fails('latest dated history')

    def test_missing_evidence(self):
        self.change(self.task, '| passed | retained output |', '| passed | |')
        self.fails('complete verification rows')

    def test_missing_review(self):
        self.change(self.task, 'No findings (reviewed by Codex).', 'Pending review.')
        self.fails('review findings/disposition')

    def test_next_id_collision(self):
        self.change('.studio/production/backlog.md', 'Next free ID: TASK-0003', 'Next free ID: TASK-0002')
        self.fails('overlapping Next free ID')

    def test_active_task_needs_record(self):
        self.change('.studio/production/backlog.md', '| proposed |', '| approved |')
        self.fails('non-proposed task has no record')

    def test_broken_link(self):
        self.put('.studio/STATE.md', '[record](production/tasks/missing.md)\n')
        self.fails('broken file link')

    def test_encoded_link(self):
        self.put('.studio/qa/my evidence.md', 'Evidence')
        self.put('.studio/STATE.md', '[evidence](qa/my%20evidence.md)\n')
        self.assertEqual(check.validate(self.root), [])

    def test_current_report_unknown_task(self):
        self.put('.studio/IMPLEMENTATION_REPORT.md', 'Task: TASK-0099\n')
        self.fails('current summary names an unknown task')

    def test_cli_nonzero_on_bad_records(self):
        self.put('.studio/events.jsonl', '{}\n')
        result=subprocess.run([sys.executable, str(Path(check.__file__)), '--root', str(self.root)], capture_output=True,text=True)
        self.assertEqual(result.returncode, 1)
        self.assertIn('ERROR:', result.stdout)

    def test_exact_legacy_exception_cannot_exempt_modified_event(self):
        del self.event['task']
        with patch.object(check, 'LEGACY_EVENT_SHA256', check.fingerprint(self.event)):
            self.put('.studio/events.jsonl', json.dumps(self.event)+'\n')
            self.assertEqual(check.validate(self.root), [])
            self.event['message']='Changed legacy event'
            self.put('.studio/events.jsonl', json.dumps(self.event)+'\n')
            self.fails('missing task ID')

    def test_missing_review_disposition(self):
        self.change(self.task, 'No findings (reviewed by Codex).', '| R1 | Bug | Codex | |')
        self.fails('review row needs')

    def test_duplicate_required_heading(self):
        self.change(self.task, '## Scope', '## Constraints\nOther constraints.\n\n## Scope')
        self.fails('duplicate required section')

    def test_malformed_scope_is_reported_not_crash(self):
        del self.event['task']
        self.event['scope']=[]
        self.put('.studio/events.jsonl', json.dumps(self.event)+'\n')
        self.fails('missing task ID')


if __name__ == '__main__':
    unittest.main()
