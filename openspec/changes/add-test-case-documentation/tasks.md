# Tasks: Add test case documentation

## 1. Documentation
- [ ] 1.1 Add `docs/test-cases.md` with a short purpose and structure (test case id, requirement ref, description, steps, verification, expected result).
- [ ] 1.2 Populate test cases for Todo domain (US-T1–US-T4): one or more test cases per acceptance criterion, with requirement ref (e.g. US-T1), verification method (e.g. unit/integration test or command), and expected result.
- [ ] 1.3 Populate test cases for Xtask workflow (US-X1–US-X3): e.g. `cargo xtask --help`, `cargo xtask run`, and optional new subcommand check.
- [ ] 1.4 Add a short “Maintenance” section stating that new requirements or acceptance criteria SHALL be reflected in this document.

## 2. References
- [ ] 2.1 From `docs/acceptance.md` or README, add a pointer to `docs/test-cases.md` (e.g. one line in acceptance doc or project.md).

## 3. Validation
- [ ] 3.1 Confirm every acceptance criterion in `docs/acceptance.md` has at least one corresponding test case in `docs/test-cases.md`.
- [ ] 3.2 Confirm `openspec validate add-test-case-documentation --strict` passes.
