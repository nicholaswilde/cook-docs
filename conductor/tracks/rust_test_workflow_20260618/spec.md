# Specification: Update GitHub Test Workflow to test Rust

## Overview
Update `.github/workflows/test.yaml` to run Rust tests using Cargo instead of Go tests.

## Requirements
1. Remove Go setup (`actions/setup-go`) and `go test` steps.
2. Add Rust toolchain setup.
3. Run `cargo test` on push and pull requests.
4. Retain YAML lint step.
