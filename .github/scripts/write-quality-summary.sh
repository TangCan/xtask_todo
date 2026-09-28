#!/usr/bin/env bash
set -euo pipefail

mkdir -p quality
{
  echo "# CI quality summary"
  echo
  echo "- commit: ${GITHUB_SHA:-local}"
  echo "- workflow: ${GITHUB_WORKFLOW:-local}"
  echo "- runner: ${RUNNER_OS:-unknown}/${RUNNER_ARCH:-unknown}"
  echo "- ref: ${GITHUB_REF_NAME:-local}"
  echo
  echo "## Gates"
  echo
  echo "The job attempted the following gates before this summary was uploaded; the workflow job status is authoritative for pass/fail:"
  echo
  echo "- cargo fmt --all -- --check"
  echo "- cargo build --workspace"
  echo "- cargo nextest run --workspace --all-features"
  echo "- cargo test --doc --workspace --all-features"
  echo "- cargo doc --workspace --no-deps"
  echo "- cargo clippy --workspace --all-targets --all-features -- -D warnings"
  echo "- cargo llvm-cov coverage gates"
  echo "- cargo deny check"
  echo "- cargo audit"
} > quality/quality-summary.md
