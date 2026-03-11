# Change: Add test case documentation

## Why
The project has requirements, design, tasks, and an acceptance checklist, but no dedicated test case document. A structured test case document improves traceability (requirement → test case → verification), onboarding, and regression clarity.

## What Changes
- Introduce a single test case document (e.g. `docs/test-cases.md`) that lists concrete test cases with id, requirement reference, steps, verification method, and expected result.
- Define the capability "test-documentation" in OpenSpec so the project SHALL maintain this document and keep it aligned with requirements and acceptance.

## Impact
- Affected specs: new capability `test-documentation`
- Affected code/docs: new file `docs/test-cases.md`; optional updates to `docs/acceptance.md` or README to reference it
