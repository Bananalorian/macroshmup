# Quality and Evidence
Document test procedures, results, reproduction steps and regressions. Differentiate tests executed from tests suggested.

**Where results go:** in the task's record (`.studio/production/tasks/TASK-NNNN-*.md`), under
"Verification performed" and "Review findings and disposition". This folder holds the procedures,
not a second copy of results. Write a separate QA file here only for a procedure worth reusing,
and link it from the records that use it.

**A verification entry says:** what was checked; on which commit or working tree; the exact
command or method; the result (pass/fail, numbers); where the evidence lives; which acceptance
criteria it covers. A check that wasn't run goes under "Not performed", never in the table.

**Evidence that lasts:** a commit or tag; a CI run URL (`gh run view <id>`); a file committed to
the repo (`docs/screenshots/` for captures); or the numbers quoted in the record. `./shots/` is
gitignored and scratchpads are temporary, so quote what they showed or commit the image.

**PILGRIM's checks** (from `PROJECT_RULES.md`, definition of done):
- `cargo test`: the headless sim tests (CI runs them in debug, with overflow checks).
- Warning-free native and wasm builds: CI doesn't enforce this, so read the build output.
- Screenshots with `PG_SHOTS` (grayscale with `PG_GRAY=1`), bot runs with `PG_AUTO`, balance
  reports with `PG_SIM`.
- Re-check earlier sprints' "Done when" items for regressions.
- CI green on the push and the tag (Pages and Release): `gh run list`, `gh run view`.

**Review findings:** each one gets a disposition: fixed (re-verified as a new entry), accepted as
is (who decided, why), deferred (to TASK-NNNN), or rejected (why). Findings are never deleted.

## Studio record validation (TASK-0007)
- `python -B tools/check_studio_records.py`: checks current Studio record structure and references, returning nonzero on errors.
- `python -B -m unittest discover -s tools -p "test_studio_records.py" -v`: isolated regression fixtures, including deliberately malformed records.
Both run automatically in the existing CI test job. See the manual's Automatic record checks section for exact coverage and limits. No toolkit clone is required.
