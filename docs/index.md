# cook-docs
![GitHub tag (latest SemVer)](https://img.shields.io/github/v/tag/nicholaswilde/cook-docs?style=for-the-badge)
[![Go Report Card](https://goreportcard.com/badge/github.com/nicholaswilde/cook-docs?style=for-the-badge)](https://goreportcard.com/report/github.com/nicholaswilde/cook-docs)
[![ci](https://img.shields.io/github/workflow/status/nicholaswilde/cook-docs/ci?label=ci&style=for-the-badge)](https://github.com/nicholaswilde/cook-docs/actions/workflows/ci.yaml)
[![pre-commit](https://img.shields.io/badge/pre--commit-enabled-brightgreen?logo=pre-commit&logoColor=white&style=for-the-badge)](https://pre-commit.com/)
[![task](https://img.shields.io/badge/task-enabled-brightgreen?logo=task&logoColor=white&style=for-the-badge)](https://taskfile.dev/#/)

A tool for automatically generating markdown documentation for [cooklang][1] recipes

## :rocket: TL;DR

### :floppy_disk: Installation

=== "brew"
    ```bash
    brew install nicholaswilde/tap/cook-docs
    ```

=== "scoop"
    ```bash
    scoop bucket add nicholaswilde https://github.com/nicholaswilde/scoop-bucket.git
    scoop install nicholaswilde/cook-docs
    ```

### :gear: Usage

!!! warning
      The mode of operation of `cook-docs` is to process all recipes in the
      working directory and sub folders. See [Mode of Operation][4] for
      details.

```bash title="Run binary directly"
cook-docs
# OR
cook-docs --dry-run # prints generated documentation to stdout rather than modifying markdown files.
```

## :star: Contributing

See [Contributing](./CONTRIBUTING/)

## :sparkles: Code of Conduct

See [Code of Conduct](./CODE_OF_CONDUCT)

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
[4]: ./about#mode-of-operation
