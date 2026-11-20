# Specification: Implement Output Directory Configuration

## User Story
As a cook-docs CLI user,
I want to specify a custom target directory for all generated recipe markdown files,
So that I can separate my raw Cooklang source files from my build/documentation output folder without manual copying.

## Requirements
1. **CLI Flag & Binding**:
   - Add a new CLI string flag `--output-dir` (shorthand `-o`) to the main command.
   - Bind the flag using Viper under the name `output-dir` (alias `outputDir`).
   - Add the corresponding field `OutputDir string` to the `types.Config` struct.

2. **Output Path Generation**:
   - If `OutputDir` is configured:
     - The output markdown file should be written to the `OutputDir`.
     - The path inside `OutputDir` should match the recipe's relative path from `RecipeSearchRoot`. For example:
       - If `RecipeSearchRoot` is `recipes/` and `OutputDir` is `build/`.
       - A recipe at `recipes/desserts/cake.cook` should produce documentation at `build/desserts/cake.md`.
   - If `OutputDir` is empty, fallback to the current behavior (writing the markdown file in the same directory as the input `.cook` file).

3. **Directory Creation**:
   - Ensure the target output directory and any required subdirectories are created automatically (e.g., using `os.MkdirAll`) before creating the target markdown file.
