## ADDED Requirements

### Requirement: Formatting
The project SHALL use `rustfmt` for consistent style. All Rust source SHALL pass `cargo fmt -- --check`. A `rustfmt.toml` (or equivalent) at the repo root MAY be used to fix options such as `edition` and `max_width`.

#### Scenario: CI or local check enforces format
- **GIVEN** the project has a formatting configuration (e.g. `rustfmt.toml`)
- **WHEN** a developer or CI runs `cargo fmt -- --check`
- **THEN** the command succeeds with no formatting changes required (or the project documents an exception)

### Requirement: Linting
The project SHALL pass `cargo clippy` with no warnings that are treated as errors (e.g. `-D warnings` in CI). Specific lints MAY be allowed via crate-level or project configuration, provided the choice is documented.

#### Scenario: CI or local check enforces clippy
- **GIVEN** the project has Clippy enabled (default or configured)
- **WHEN** a developer or CI runs `cargo clippy --all-targets -- -D warnings`
- **THEN** the command succeeds with zero warnings, or the project explicitly allows certain lints and documents the reason

### Requirement: Root README
The project SHALL have a README at the repository root that describes the project purpose, how to run the main user-facing commands (e.g. `cargo xtask todo add/list/complete/delete`), where to find documentation (e.g. `docs/`), and how to run tests.

#### Scenario: New contributor can run and test
- **GIVEN** a new contributor opens the repository
- **WHEN** they read the root README
- **THEN** they can run the primary commands (e.g. xtask todo) and run tests (e.g. `cargo test`) without guessing

### Requirement: Continuous integration
The project SHALL have a CI pipeline (e.g. GitHub Actions, GitLab CI) that runs on push or pull request and executes at least: `cargo build`, `cargo test`, `cargo fmt -- --check`, and `cargo clippy` with warnings as errors. Optional steps MAY include `cargo doc --no-deps`.

#### Scenario: CI runs quality checks
- **GIVEN** the project has a CI configuration
- **WHEN** a push or PR triggers the pipeline
- **THEN** the pipeline runs build, test, format check, and clippy; the job fails if any of these fail
