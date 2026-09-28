---
stepsCompleted:
  - step-01-preflight-and-context
  - step-02-generation-mode
  - step-03-test-strategy
  - step-04c-aggregate
  - step-05-validate-and-complete
lastStep: step-05-validate-and-complete
lastSaved: '2026-09-28'
storyId: '1.1'
storyKey: '1-1-test-runner-and-testing-toolchain-upgrade'
storyFile: '_agile-output/implementation-artifacts/1-1-test-runner-and-testing-toolchain-upgrade.md'
atddChecklistPath: '_bmad-output/test-artifacts/atdd-checklist-1-1-test-runner-and-testing-toolchain-upgrade.md'
generatedTestFiles: []
inputDocuments:
  - '_agile-output/implementation-artifacts/1-1-test-runner-and-testing-toolchain-upgrade.md'
  - '_agile-output/planning-artifacts/epics-hardening.md'
  - '_agile-output/project-context.md'
  - 'Cargo.toml'
  - 'xtask/Cargo.toml'
  - 'crates/todo/Cargo.toml'
  - 'xtask/tests/'
  - 'crates/todo/src/'
acceptanceCriteria:
  - id: AC-1
    idSource: generated
    text: 'cargo nextest run passes all non-doctest tests without changing existing contracts.'
  - id: AC-2
    idSource: generated
    text: 'The declared test-only dependencies are available to the intended crates.'
  - id: AC-3
    idSource: generated
    text: 'cargo test --doc passes independently.'
  - id: AC-4
    idSource: generated
    text: 'Parallel nextest execution does not introduce cwd, temporary-directory, or mutable-state races.'
---

# ATDD Checklist: 测试运行器与测试工具链升级

## Preflight

- Stack detected: backend/Rust workspace.
- Generation mode: AI/sequential. Browser recording and API/E2E scaffolds are not applicable because this Story changes test infrastructure rather than a user-facing API.
- Existing framework: Cargo unit, integration, and doctest layout; no Playwright/Cypress/Pact artifacts.
- Risk: P1. This Story changes the execution layer used by every later quality gate.

## TDD Red Phase

No executable red-phase API/E2E files were generated. The acceptance criteria are command/toolchain contracts, not application endpoints. Generating placeholder `test.skip()` tests would create false coverage and violate the repository's Rust test conventions.

The red-phase evidence is the pre-implementation baseline:

| Criterion | Primary red-phase probe | Level | Priority | Green-phase evidence |
|---|---|---|---|---|
| AC-1 | Confirm nextest is not yet configured/available, then run the eventual command after implementation | CI/toolchain integration | P1 | `cargo nextest run` full workspace pass |
| AC-2 | Compile a minimal dev-dependency consumer after manifest changes | Build integration | P1 | `cargo check --workspace --all-targets` |
| AC-3 | Run doctests independently before and after migration | CI/toolchain integration | P1 | `cargo test --doc` pass |
| AC-4 | Run existing cwd/temp-dir tests under nextest parallelism and inspect failures for shared-state races | Test-runner integration | P1 | repeated nextest pass with no isolation regressions |

## Test Strategy

- Prefer existing unit and integration tests over new duplicate tests.
- Do not add browser, HTTP, or contract test scaffolds for this infrastructure-only Story.
- Use the smallest direct evidence first: dependency resolution, nextest discovery, doctest execution, then workspace execution.
- Preserve existing `cwd_test_lock`, `RestoreCwd`, temporary-directory cleanup, and environment-variable cleanup unless a concrete nextest failure proves a change is required.
- Do not hide failures by globally forcing serial execution; isolate only tests with an evidenced shared resource.

## Green-Phase Activation

1. Add the dev dependencies and run `cargo check --workspace --all-targets`.
2. Run `cargo nextest run`; activate no skipped tests because this Story has no generated red test files.
3. Run `cargo test --doc` separately.
4. Repeat the full nextest run after any isolation fix.
5. Record command output and any changed test isolation in the Story's Dev Agent Record.

## Validation

- [x] Story metadata and handoff path captured.
- [x] All four acceptance criteria mapped exactly once to a primary probe.
- [x] No API/E2E scaffolds emitted for a non-API infrastructure Story.
- [x] No browser sessions or temporary CLI recording artifacts created.
- [ ] Green-phase command results pending implementation.

## Next Step

Proceed to `bmad-build` for Story 1.1. After implementation, run code review, test automation expansion, and the requested regression command sequence.
