# cook-docs
![GitHub tag (latest SemVer)](https://img.shields.io/github/v/tag/nicholaswilde/cook-docs?style=for-the-badge)
[![Go Report Card](https://goreportcard.com/badge/github.com/nicholaswilde/cook-docs?style=for-the-badge)](https://goreportcard.com/report/github.com/nicholaswilde/cook-docs)
[![ci](https://img.shields.io/github/workflow/status/nicholaswilde/cook-docs/ci?label=ci&style=for-the-badge)](https://github.com/nicholaswilde/cook-docs/actions/workflows/ci.yaml)
[![pre-commit](https://img.shields.io/badge/pre--commit-enabled-brightgreen?logo=pre-commit&logoColor=white&style=for-the-badge)](https://pre-commit.com/)
[![task](https://img.shields.io/badge/task-enabled-brightgreen?logo=task&logoColor=white&style=for-the-badge)](https://taskfile.dev/#/)

A tool for automatically generating markdown documentation for [cooklang][1] recipes.

---

## :rocket:&nbsp; TL;DR

### :floppy_disk:&nbsp; Installation

```
# brew
brew install nicholaswilde/tap/cook-docs
```

```
# scoop
scoop bucket add nicholaswilde https://github.com/nicholaswilde/scoop-bucket.git
scoop install nicholaswilde/cook-docs
```

### :gear:&nbsp; Usage

> :warning: Warning: The mode of operation of cook-docs is to process all
> recipes in the working directory and sub folders. See [Mode of Operation][2]
> for details.

```
cook-docs
# OR
cook-docs --dry-run # prints generated documentation to stdout rather than modifying markdown files.
```

---

## :book:&nbsp; Documentation

Documentation can be found [here](http://nicholaswilde.io/cook-docs).

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

## ​:balance_scale:​&nbsp;​ License

​[​Apache License 2.0](../LICENSE)

---

## ​:pencil:​&nbsp;​ Author

​This project was started in 2022 by [​Nicholas Wilde​](https://github.com/nicholaswilde/).

[1]: https://cooklang.org/
[2]: https://nicholaswilde.io/cook-docs/about#mode-of-operation
