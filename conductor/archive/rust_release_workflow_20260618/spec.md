# Specification: Update GitHub Release Workflow to release Rust binary

## Overview
Update `.github/workflows/release.yaml` to build and package Rust release binaries instead of using GoReleaser.

## Requirements
1. Remove Go setup and GoReleaser action.
2. Implement standard GitHub Action workflow for building Rust binaries and generating GitHub releases with artifacts.
