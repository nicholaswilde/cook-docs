# cook-docs

![GitHub tag (latest SemVer)](https://img.shields.io/github/v/tag/nicholaswilde/cook-docs?style=for-the-badge)
[![ci](https://img.shields.io/github/actions/workflow/status/nicholaswilde/cook-docs/ci.yaml?label=ci&style=for-the-badge&branch=main)](https://github.com/nicholaswilde/cook-docs/actions/workflows/ci.yaml)
[![Coveralls](https://img.shields.io/coveralls/github/nicholaswilde/cook-docs/main?style=for-the-badge&logo=coveralls)](https://coveralls.io/github/nicholaswilde/cook-docs?branch=main)
[![pre-commit](https://img.shields.io/badge/pre--commit-enabled-brightgreen?logo=pre-commit&logoColor=white&style=for-the-badge)](https://pre-commit.com/)
[![task](https://img.shields.io/badge/task-enabled-brightgreen?logo=task&logoColor=white&style=for-the-badge)](https://taskfile.dev/#/)

A fast Rust CLI for automatically generating Markdown documentation for [Cooklang][1] recipes using Go-compatible templates.

---

## :rocket:&nbsp; TL;DR

### :floppy_disk:&nbsp; Installation

```bash
# Homebrew
brew install nicholaswilde/tap/cook-docs
```

```powershell
# Scoop
scoop bucket add nicholaswilde https://github.com/nicholaswilde/scoop-bucket.git
scoop install nicholaswilde/cook-docs
```

```bash
# Cargo (from source)
cargo install --git https://github.com/nicholaswilde/cook-docs
```

Debian/Ubuntu `.deb` packages are also published via [`nicholaswilde/apt`](https://nicholaswilde.io/apt) — see [Installation](https://nicholaswilde.io/cook-docs/installation/).

### :gear:&nbsp; Usage

> :warning: Warning: The mode of operation of `cook-docs` is to process all
> recipes in the working directory and sub folders. See [Mode of Operation][2]
> for details.

```bash
cook-docs
# Preview generated Markdown on stdout without writing files
cook-docs --dry-run
# Watch recipes & templates and re-render automatically on save
cook-docs watch
# Validate config, templates, and output permissions
cook-docs doctor
# Check for or install the latest GitHub release
cook-docs self-update --check
```

---

## :book:&nbsp; Documentation

Documentation can be found on the [cook-docs documentation site](https://nicholaswilde.io/cook-docs).

---

## :star:&nbsp; Contributing

See [Contributing](https://nicholaswilde.io/cook-docs/CONTRIBUTING/)

---

## :sparkles:&nbsp; Code of Conduct

See [Code of Conduct](https://nicholaswilde.io/cook-docs/CODE_OF_CONDUCT/)

---

## :bulb:&nbsp; Inspiration

Inspiration for this repository has been taken from [helm-docs](https://github.com/norwoodj/helm-docs).

---

## :balance_scale:&nbsp; License

[Apache License 2.0](../LICENSE)

---

## :pencil:&nbsp; Author

This project was started in 2022 by [Nicholas Wilde](https://github.com/nicholaswilde/).

[1]: https://cooklang.org/
[2]: https://nicholaswilde.io/cook-docs/about#mode-of-operation
