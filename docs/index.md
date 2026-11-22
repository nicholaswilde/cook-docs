# cook-docs

![GitHub tag (latest SemVer)](https://img.shields.io/github/v/tag/nicholaswilde/cook-docs?style=for-the-badge)
[![ci](https://img.shields.io/github/actions/workflow/status/nicholaswilde/cook-docs/ci.yaml?label=ci&style=for-the-badge&branch=main)](https://github.com/nicholaswilde/cook-docs/actions/workflows/ci.yaml)
[![Coveralls](https://img.shields.io/coveralls/github/nicholaswilde/cook-docs/main?style=for-the-badge&logo=coveralls)](https://coveralls.io/github/nicholaswilde/cook-docs?branch=main)
[![pre-commit](https://img.shields.io/badge/pre--commit-enabled-brightgreen?logo=pre-commit&logoColor=white&style=for-the-badge)](https://pre-commit.com/)
[![task](https://img.shields.io/badge/task-enabled-brightgreen?logo=task&logoColor=white&style=for-the-badge)](https://taskfile.dev/#/)

A fast Rust CLI for automatically generating Markdown documentation from [Cooklang][1] (`.cook`) recipes using Go-compatible templates.

## :rocket: TL;DR

### :floppy_disk: Installation

=== "Homebrew"
    ```bash
    brew install nicholaswilde/tap/cook-docs
    ```

=== "APT"
    ```bash
    # After adding https://nicholaswilde.io/apt (see Installation page)
    sudo apt update && sudo apt install cook-docs
    ```

=== "Scoop"
    ```powershell
    scoop bucket add nicholaswilde https://github.com/nicholaswilde/scoop-bucket.git
    scoop install nicholaswilde/cook-docs
    ```

=== "Cargo"
    ```bash
    cargo install --git https://github.com/nicholaswilde/cook-docs
    ```

### :gear: Usage

!!! warning
    The mode of operation of `cook-docs` is to process all recipes in the
    working directory and sub folders. See [Mode of Operation][4] for
    details.

```bash title="Run binary directly"
cook-docs
# Render a single recipe into a mirrored output dir, copy companion images, and preserve manual edits
cook-docs -c ./cook -o ./docs --assets-dir ./docs/assets/images --merge ./cook/breads/Bagels.cook
# Enrich ingredients with emojis, gram weights, and internal recipe cross-links
cook-docs -c ./cook -o ./docs --emoji-file ./emoji.yaml --weights-file ./weights.yaml --link-dirs sauces,ingredients
# Preview generated Markdown on stdout without writing files
cook-docs --dry-run
# Inspect parsed recipes (including aggregated Ingredients/Cookware) as clean JSON on stdout
cook-docs --jsonify
# Watch recipes & templates and re-render automatically on save
cook-docs watch --merge
# Validate config, templates, output permissions, and recipe syntax/images/metadata/emojis
cook-docs doctor --check-recipes
```

## :star: Contributing

See [Contributing](./CONTRIBUTING.md)

## :sparkles: Code of Conduct

See [Code of Conduct](./CODE_OF_CONDUCT.md)

## :bulb: Inspiration

Inspiration for this repository has been taken from [helm-docs][2].

## :scales: License

[Apache License 2.0][3]

## :pencil: Author

This project was started in 2022 by [Nicholas Wilde].

[Nicholas Wilde]: https://github.com/nicholaswilde/
[1]: https://cooklang.org/
[2]: https://github.com/norwoodj/helm-docs
[3]: https://github.com/nicholaswilde/cook-docs/blob/main/LICENSE
[4]: ./about.md#mode-of-operation
