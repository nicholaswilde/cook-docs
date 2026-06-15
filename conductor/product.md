# Product Guide - cook-docs

## Vision & Description
`cook-docs` is a command-line tool written in Go that automatically generates clean, standardized markdown documentation for recipes written in the Cooklang markup language. By utilizing Go templates, it enables recipe authors, developers, and food bloggers to maintain version-controlled recipe source files while automatically rendering beautiful, human-readable markdown docs.

## Target Audience
- **Tech-savvy Home Cooks**: Users who maintain recipe collections in Git repositories.
- **Recipe Website Maintainers**: Developers who build static sites (like Hugo, Jekyll, or MkDocs) using markdown and want to parse Cooklang files automatically.
- **Technical Writers & Food Bloggers**: Content creators looking for automated recipe formatting and publishing workflows.

## Key Features
1. **Automated Recipe Search & Parsing**: Recursively scans directories to find Cooklang recipes and parses them using the `cooklang-go` library.
2. **Flexible Template Rendering**: Renders recipes using custom Go html/text templates (`recipe.md.gotmpl` by default), allowing complete control over document layout.
3. **Dry-run Execution**: Supports a dry-run flag to print rendered markdown outputs to `stdout` before writing any files.
4. **JSON Serialization**: Can serialize parsed Cooklang recipes into JSON format for integration with frontend apps or external APIs.
5. **Configurable Behavior**: Uses Viper and Cobra to support options via YAML config files (`.cookdocs`), environment variables, or CLI flags.
6. **Smart Exclusions**: Supports an ignore file (`.cookdocsignore`) to skip specified directories during recursive scans.
7. **Word Wrapping**: Provides configurable line length wrapping for step-by-step cooking instructions.
8. **Custom Output Directory**: Supports outputting generated markdown documentation to a custom folder via `--output-dir` / `-o` (or `output-dir` config setting), automatically reproducing relative subdirectory structures.

## Success Criteria
- **Ease of Use**: A single CLI command converts a directory of recipes.
- **Customization**: Authors can customize the output design entirely via template overrides.
- **Accuracy**: Perfect parsing of Cooklang-spec ingredients, cookware, timers, and metadata.
