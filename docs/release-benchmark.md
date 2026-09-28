# Release benchmark baseline

Run the repository baseline command from the workspace root:

```sh
cargo xtask release-baseline --runs 5
```

The default output is `docs/benchmarks/release-baseline.json`. Use `--output`
to write a temporary measurement elsewhere, and use `--runs 1` for a quick
smoke measurement. The command refuses to overwrite an existing output file.

The JSON schema is versioned by `schema_version`. Binary targets are discovered
from Cargo metadata and sorted by target name and package name. Each target
records its release file size and independent `--help` subprocess timings in
microseconds (`samples`, `min`, `p50`, and `p95`). The metadata also records the
commit, whether the working tree was dirty, Rust/Cargo versions, host/target
triples, profile, build arguments, and run count.

Values are measurements of the current machine and checkout, not external
benchmarks. Compare measurements only when the platform, target, toolchain,
build profile, and command arguments are comparable; cross-platform numbers are
not directly interchangeable.

## Release profile optimization

Story 2.2 evaluated `opt-level=z`, `lto=thin`, `codegen-units=1`, and
`strip="symbols"` together against the Story 2.1 baseline. On the Linux host
used for this measurement, the combination reduced all four binary sizes by
roughly 36%–37%. Cold-start p50 increased in this run, so this is a size-first
trade-off rather than a claim of universal startup improvement. The checked-in
before/after JSON files and the report record the exact commits and dirty-state
metadata.

The release profile intentionally strips symbols from distributable binaries.
Use debug builds and the unstripped build artifacts available during local
development for diagnosis; the profile change does not alter debug or test
profiles.
