# About

`cook-docs` was created to help automate the generation of markdown files from
cooklang recipes. I am using [`cooklang`][1] to help collect my recipes from
all over the internet and I wanted to publish them on my website using
[mkdocs-material][2] but I got tired of manually creating the markdown files.
I knew from my experience with [k8s-at-home helm charts][3] that [helm-docs][4]
existed as a tool that automatically generates markdown files from template and
value files and I thought that I could make something similar for `cooklang`.

## :ocean: Workflow

My ideal workflow consists of me using [cook-import][5] to parse a website from
a recipe, commit it to my [recipes repo][6], and have GitHub Actions generate
the markdown files using `cook-docs` and publish them to my
[recipes mkdocs-material website][7].

## :tram: Mode of Operation

The way that `cook-docs` works is similar to `helm-docs` where it crawls through
the working directory and its sub folders and looks for any `*.cook` files and
`recipe.md.gotmpl` template files to process. This mode of operation is preferred
over specifying each recipe file to process to help with automation. However,
this requires the user to be diligent in how, when, and where they are using
`cook-docs`. See [ignoring recipe directories][8] for how to ignore directories
and files.

[1]: https://cooklang.org/
[2]: https://squidfunk.github.io/mkdocs-material/
[3]: https://docs.k8s-at-home.com/
[4]: https://github.com/norwoodj/helm-docs
[5]: https://github.com/cooklang/cook-import
[6]: https://github.com/nicholaswilde/recipes/
[7]: https://nicholaswilde.io/recipes/
[8]: ../usage#ignoring-recipe-directories
