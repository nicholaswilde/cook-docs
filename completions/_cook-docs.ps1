
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'cook-docs' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'cook-docs'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'cook-docs' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('-i', '-i', [CompletionResultType]::ParameterName, 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")')
            [CompletionResult]::new('--ignore-file', '--ignore-file', [CompletionResultType]::ParameterName, 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'directory to search recursively within for recipes (default ".")')
            [CompletionResult]::new('--recipe-search-root', '--recipe-search-root', [CompletionResultType]::ParameterName, 'directory to search recursively within for recipes (default ".")')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")')
            [CompletionResult]::new('--log-level', '--log-level', [CompletionResultType]::ParameterName, 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")')
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")')
            [CompletionResult]::new('--template-files', '--template-files', [CompletionResultType]::ParameterName, 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")')
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'word wrap line length for recipe steps section (default 120)')
            [CompletionResult]::new('--word-wrap', '--word-wrap', [CompletionResultType]::ParameterName, 'word wrap line length for recipe steps section (default 120)')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'custom output directory to write generated markdown documentation')
            [CompletionResult]::new('--output-dir', '--output-dir', [CompletionResultType]::ParameterName, 'custom output directory to write generated markdown documentation')
            [CompletionResult]::new('--assets-dir', '--assets-dir', [CompletionResultType]::ParameterName, 'optional directory to copy companion recipe images into (slug-named)')
            [CompletionResult]::new('--image-ext', '--image-ext', [CompletionResultType]::ParameterName, 'optional image extension override for Info.ImageSlug (e.g. webp)')
            [CompletionResult]::new('--image-ext-map', '--image-ext-map', [CompletionResultType]::ParameterName, 'comma-separated source=target image extension mappings for Info.ImageSlug (e.g. jpg=webp,jpeg=webp)')
            [CompletionResult]::new('--convert-images', '--convert-images', [CompletionResultType]::ParameterName, 'convert matching companion images when copying to --assets-dir (supports ''webp'' via cwebp)')
            [CompletionResult]::new('--image-quality', '--image-quality', [CompletionResultType]::ParameterName, 'output image quality (1-100) for --convert-images (default 80)')
            [CompletionResult]::new('--emoji-file', '--emoji-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON file mapping ingredients and cookware to emojis')
            [CompletionResult]::new('--weights-file', '--weights-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON ingredient density file for toGrams volume-to-weight conversion')
            [CompletionResult]::new('--link-dirs', '--link-dirs', [CompletionResultType]::ParameterName, 'comma-separated recipe subdirectories eligible for linkRecipe cross-linking')
            [CompletionResult]::new('--required-metadata', '--required-metadata', [CompletionResultType]::ParameterName, 'comma-separated metadata keys required by doctor --check-recipes (e.g. source,serves)')
            [CompletionResult]::new('--ignored-ingredients-file', '--ignored-ingredients-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON list of ingredients that should not be tagged with ''@'' (checked by doctor --check-recipes)')
            [CompletionResult]::new('--nav-file', '--nav-file', [CompletionResultType]::ParameterName, 'optional Zensical (zensical.toml) or MkDocs (mkdocs.yml) nav file to sync rendered recipes into')
            [CompletionResult]::new('--debounce', '--debounce', [CompletionResultType]::ParameterName, 'debounce delay in milliseconds for watch mode (default 300)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'version for cook-docs')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'version for cook-docs')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'don''t actually render any markdown files just print to stdout passed')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'don''t actually render any markdown files just print to stdout passed')
            [CompletionResult]::new('-j', '-j', [CompletionResultType]::ParameterName, 'parse the recipe and display it in json format')
            [CompletionResult]::new('--jsonify', '--jsonify', [CompletionResultType]::ParameterName, 'parse the recipe and display it in json format')
            [CompletionResult]::new('--link-in-steps', '--link-in-steps', [CompletionResultType]::ParameterName, 'cross-link matching @ingredient references inside step directions')
            [CompletionResult]::new('--merge', '--merge', [CompletionResultType]::ParameterName, 'merge generated content into existing Markdown files without clobbering manual edits')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'force full overwrite of existing Markdown files even when --merge / merge is enabled')
            [CompletionResult]::new('--watch', '--watch', [CompletionResultType]::ParameterName, 'watch recipes and templates and re-render automatically on changes')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('watch', 'watch', [CompletionResultType]::ParameterValue, 'Watch recipes and templates and re-render automatically on changes')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell autocompletions script')
            [CompletionResult]::new('man', 'man', [CompletionResultType]::ParameterValue, 'Generate UNIX roff man page')
            [CompletionResult]::new('doctor', 'doctor', [CompletionResultType]::ParameterValue, 'Validate configuration, inputs, templates, and output permissions')
            [CompletionResult]::new('self-update', 'self-update', [CompletionResultType]::ParameterValue, 'Check for and install the latest cook-docs release from GitHub')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'cook-docs;watch' {
            [CompletionResult]::new('--debounce', '--debounce', [CompletionResultType]::ParameterName, 'debounce delay in milliseconds for watch mode (default 300)')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('-i', '-i', [CompletionResultType]::ParameterName, 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")')
            [CompletionResult]::new('--ignore-file', '--ignore-file', [CompletionResultType]::ParameterName, 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'directory to search recursively within for recipes (default ".")')
            [CompletionResult]::new('--recipe-search-root', '--recipe-search-root', [CompletionResultType]::ParameterName, 'directory to search recursively within for recipes (default ".")')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")')
            [CompletionResult]::new('--log-level', '--log-level', [CompletionResultType]::ParameterName, 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")')
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")')
            [CompletionResult]::new('--template-files', '--template-files', [CompletionResultType]::ParameterName, 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")')
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'word wrap line length for recipe steps section (default 120)')
            [CompletionResult]::new('--word-wrap', '--word-wrap', [CompletionResultType]::ParameterName, 'word wrap line length for recipe steps section (default 120)')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'custom output directory to write generated markdown documentation')
            [CompletionResult]::new('--output-dir', '--output-dir', [CompletionResultType]::ParameterName, 'custom output directory to write generated markdown documentation')
            [CompletionResult]::new('--assets-dir', '--assets-dir', [CompletionResultType]::ParameterName, 'optional directory to copy companion recipe images into (slug-named)')
            [CompletionResult]::new('--image-ext', '--image-ext', [CompletionResultType]::ParameterName, 'optional image extension override for Info.ImageSlug (e.g. webp)')
            [CompletionResult]::new('--image-ext-map', '--image-ext-map', [CompletionResultType]::ParameterName, 'comma-separated source=target image extension mappings for Info.ImageSlug (e.g. jpg=webp,jpeg=webp)')
            [CompletionResult]::new('--convert-images', '--convert-images', [CompletionResultType]::ParameterName, 'convert matching companion images when copying to --assets-dir (supports ''webp'' via cwebp)')
            [CompletionResult]::new('--image-quality', '--image-quality', [CompletionResultType]::ParameterName, 'output image quality (1-100) for --convert-images (default 80)')
            [CompletionResult]::new('--emoji-file', '--emoji-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON file mapping ingredients and cookware to emojis')
            [CompletionResult]::new('--weights-file', '--weights-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON ingredient density file for toGrams volume-to-weight conversion')
            [CompletionResult]::new('--link-dirs', '--link-dirs', [CompletionResultType]::ParameterName, 'comma-separated recipe subdirectories eligible for linkRecipe cross-linking')
            [CompletionResult]::new('--required-metadata', '--required-metadata', [CompletionResultType]::ParameterName, 'comma-separated metadata keys required by doctor --check-recipes (e.g. source,serves)')
            [CompletionResult]::new('--ignored-ingredients-file', '--ignored-ingredients-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON list of ingredients that should not be tagged with ''@'' (checked by doctor --check-recipes)')
            [CompletionResult]::new('--nav-file', '--nav-file', [CompletionResultType]::ParameterName, 'optional Zensical (zensical.toml) or MkDocs (mkdocs.yml) nav file to sync rendered recipes into')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'don''t actually render any markdown files just print to stdout passed')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'don''t actually render any markdown files just print to stdout passed')
            [CompletionResult]::new('-j', '-j', [CompletionResultType]::ParameterName, 'parse the recipe and display it in json format')
            [CompletionResult]::new('--jsonify', '--jsonify', [CompletionResultType]::ParameterName, 'parse the recipe and display it in json format')
            [CompletionResult]::new('--link-in-steps', '--link-in-steps', [CompletionResultType]::ParameterName, 'cross-link matching @ingredient references inside step directions')
            [CompletionResult]::new('--merge', '--merge', [CompletionResultType]::ParameterName, 'merge generated content into existing Markdown files without clobbering manual edits')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'force full overwrite of existing Markdown files even when --merge / merge is enabled')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'cook-docs;completions' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('-i', '-i', [CompletionResultType]::ParameterName, 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")')
            [CompletionResult]::new('--ignore-file', '--ignore-file', [CompletionResultType]::ParameterName, 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'directory to search recursively within for recipes (default ".")')
            [CompletionResult]::new('--recipe-search-root', '--recipe-search-root', [CompletionResultType]::ParameterName, 'directory to search recursively within for recipes (default ".")')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")')
            [CompletionResult]::new('--log-level', '--log-level', [CompletionResultType]::ParameterName, 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")')
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")')
            [CompletionResult]::new('--template-files', '--template-files', [CompletionResultType]::ParameterName, 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")')
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'word wrap line length for recipe steps section (default 120)')
            [CompletionResult]::new('--word-wrap', '--word-wrap', [CompletionResultType]::ParameterName, 'word wrap line length for recipe steps section (default 120)')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'custom output directory to write generated markdown documentation')
            [CompletionResult]::new('--output-dir', '--output-dir', [CompletionResultType]::ParameterName, 'custom output directory to write generated markdown documentation')
            [CompletionResult]::new('--assets-dir', '--assets-dir', [CompletionResultType]::ParameterName, 'optional directory to copy companion recipe images into (slug-named)')
            [CompletionResult]::new('--image-ext', '--image-ext', [CompletionResultType]::ParameterName, 'optional image extension override for Info.ImageSlug (e.g. webp)')
            [CompletionResult]::new('--image-ext-map', '--image-ext-map', [CompletionResultType]::ParameterName, 'comma-separated source=target image extension mappings for Info.ImageSlug (e.g. jpg=webp,jpeg=webp)')
            [CompletionResult]::new('--convert-images', '--convert-images', [CompletionResultType]::ParameterName, 'convert matching companion images when copying to --assets-dir (supports ''webp'' via cwebp)')
            [CompletionResult]::new('--image-quality', '--image-quality', [CompletionResultType]::ParameterName, 'output image quality (1-100) for --convert-images (default 80)')
            [CompletionResult]::new('--emoji-file', '--emoji-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON file mapping ingredients and cookware to emojis')
            [CompletionResult]::new('--weights-file', '--weights-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON ingredient density file for toGrams volume-to-weight conversion')
            [CompletionResult]::new('--link-dirs', '--link-dirs', [CompletionResultType]::ParameterName, 'comma-separated recipe subdirectories eligible for linkRecipe cross-linking')
            [CompletionResult]::new('--required-metadata', '--required-metadata', [CompletionResultType]::ParameterName, 'comma-separated metadata keys required by doctor --check-recipes (e.g. source,serves)')
            [CompletionResult]::new('--ignored-ingredients-file', '--ignored-ingredients-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON list of ingredients that should not be tagged with ''@'' (checked by doctor --check-recipes)')
            [CompletionResult]::new('--nav-file', '--nav-file', [CompletionResultType]::ParameterName, 'optional Zensical (zensical.toml) or MkDocs (mkdocs.yml) nav file to sync rendered recipes into')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'don''t actually render any markdown files just print to stdout passed')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'don''t actually render any markdown files just print to stdout passed')
            [CompletionResult]::new('-j', '-j', [CompletionResultType]::ParameterName, 'parse the recipe and display it in json format')
            [CompletionResult]::new('--jsonify', '--jsonify', [CompletionResultType]::ParameterName, 'parse the recipe and display it in json format')
            [CompletionResult]::new('--link-in-steps', '--link-in-steps', [CompletionResultType]::ParameterName, 'cross-link matching @ingredient references inside step directions')
            [CompletionResult]::new('--merge', '--merge', [CompletionResultType]::ParameterName, 'merge generated content into existing Markdown files without clobbering manual edits')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'force full overwrite of existing Markdown files even when --merge / merge is enabled')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'cook-docs;man' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('-i', '-i', [CompletionResultType]::ParameterName, 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")')
            [CompletionResult]::new('--ignore-file', '--ignore-file', [CompletionResultType]::ParameterName, 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'directory to search recursively within for recipes (default ".")')
            [CompletionResult]::new('--recipe-search-root', '--recipe-search-root', [CompletionResultType]::ParameterName, 'directory to search recursively within for recipes (default ".")')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")')
            [CompletionResult]::new('--log-level', '--log-level', [CompletionResultType]::ParameterName, 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")')
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")')
            [CompletionResult]::new('--template-files', '--template-files', [CompletionResultType]::ParameterName, 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")')
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'word wrap line length for recipe steps section (default 120)')
            [CompletionResult]::new('--word-wrap', '--word-wrap', [CompletionResultType]::ParameterName, 'word wrap line length for recipe steps section (default 120)')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'custom output directory to write generated markdown documentation')
            [CompletionResult]::new('--output-dir', '--output-dir', [CompletionResultType]::ParameterName, 'custom output directory to write generated markdown documentation')
            [CompletionResult]::new('--assets-dir', '--assets-dir', [CompletionResultType]::ParameterName, 'optional directory to copy companion recipe images into (slug-named)')
            [CompletionResult]::new('--image-ext', '--image-ext', [CompletionResultType]::ParameterName, 'optional image extension override for Info.ImageSlug (e.g. webp)')
            [CompletionResult]::new('--image-ext-map', '--image-ext-map', [CompletionResultType]::ParameterName, 'comma-separated source=target image extension mappings for Info.ImageSlug (e.g. jpg=webp,jpeg=webp)')
            [CompletionResult]::new('--convert-images', '--convert-images', [CompletionResultType]::ParameterName, 'convert matching companion images when copying to --assets-dir (supports ''webp'' via cwebp)')
            [CompletionResult]::new('--image-quality', '--image-quality', [CompletionResultType]::ParameterName, 'output image quality (1-100) for --convert-images (default 80)')
            [CompletionResult]::new('--emoji-file', '--emoji-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON file mapping ingredients and cookware to emojis')
            [CompletionResult]::new('--weights-file', '--weights-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON ingredient density file for toGrams volume-to-weight conversion')
            [CompletionResult]::new('--link-dirs', '--link-dirs', [CompletionResultType]::ParameterName, 'comma-separated recipe subdirectories eligible for linkRecipe cross-linking')
            [CompletionResult]::new('--required-metadata', '--required-metadata', [CompletionResultType]::ParameterName, 'comma-separated metadata keys required by doctor --check-recipes (e.g. source,serves)')
            [CompletionResult]::new('--ignored-ingredients-file', '--ignored-ingredients-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON list of ingredients that should not be tagged with ''@'' (checked by doctor --check-recipes)')
            [CompletionResult]::new('--nav-file', '--nav-file', [CompletionResultType]::ParameterName, 'optional Zensical (zensical.toml) or MkDocs (mkdocs.yml) nav file to sync rendered recipes into')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'don''t actually render any markdown files just print to stdout passed')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'don''t actually render any markdown files just print to stdout passed')
            [CompletionResult]::new('-j', '-j', [CompletionResultType]::ParameterName, 'parse the recipe and display it in json format')
            [CompletionResult]::new('--jsonify', '--jsonify', [CompletionResultType]::ParameterName, 'parse the recipe and display it in json format')
            [CompletionResult]::new('--link-in-steps', '--link-in-steps', [CompletionResultType]::ParameterName, 'cross-link matching @ingredient references inside step directions')
            [CompletionResult]::new('--merge', '--merge', [CompletionResultType]::ParameterName, 'merge generated content into existing Markdown files without clobbering manual edits')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'force full overwrite of existing Markdown files even when --merge / merge is enabled')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'cook-docs;doctor' {
            [CompletionResult]::new('--only', '--only', [CompletionResultType]::ParameterName, 'Comma-separated recipe check categories to run (syntax,images,metadata,emojis,ignored)')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('-i', '-i', [CompletionResultType]::ParameterName, 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")')
            [CompletionResult]::new('--ignore-file', '--ignore-file', [CompletionResultType]::ParameterName, 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'directory to search recursively within for recipes (default ".")')
            [CompletionResult]::new('--recipe-search-root', '--recipe-search-root', [CompletionResultType]::ParameterName, 'directory to search recursively within for recipes (default ".")')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")')
            [CompletionResult]::new('--log-level', '--log-level', [CompletionResultType]::ParameterName, 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")')
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")')
            [CompletionResult]::new('--template-files', '--template-files', [CompletionResultType]::ParameterName, 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")')
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'word wrap line length for recipe steps section (default 120)')
            [CompletionResult]::new('--word-wrap', '--word-wrap', [CompletionResultType]::ParameterName, 'word wrap line length for recipe steps section (default 120)')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'custom output directory to write generated markdown documentation')
            [CompletionResult]::new('--output-dir', '--output-dir', [CompletionResultType]::ParameterName, 'custom output directory to write generated markdown documentation')
            [CompletionResult]::new('--assets-dir', '--assets-dir', [CompletionResultType]::ParameterName, 'optional directory to copy companion recipe images into (slug-named)')
            [CompletionResult]::new('--image-ext', '--image-ext', [CompletionResultType]::ParameterName, 'optional image extension override for Info.ImageSlug (e.g. webp)')
            [CompletionResult]::new('--image-ext-map', '--image-ext-map', [CompletionResultType]::ParameterName, 'comma-separated source=target image extension mappings for Info.ImageSlug (e.g. jpg=webp,jpeg=webp)')
            [CompletionResult]::new('--convert-images', '--convert-images', [CompletionResultType]::ParameterName, 'convert matching companion images when copying to --assets-dir (supports ''webp'' via cwebp)')
            [CompletionResult]::new('--image-quality', '--image-quality', [CompletionResultType]::ParameterName, 'output image quality (1-100) for --convert-images (default 80)')
            [CompletionResult]::new('--emoji-file', '--emoji-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON file mapping ingredients and cookware to emojis')
            [CompletionResult]::new('--weights-file', '--weights-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON ingredient density file for toGrams volume-to-weight conversion')
            [CompletionResult]::new('--link-dirs', '--link-dirs', [CompletionResultType]::ParameterName, 'comma-separated recipe subdirectories eligible for linkRecipe cross-linking')
            [CompletionResult]::new('--required-metadata', '--required-metadata', [CompletionResultType]::ParameterName, 'comma-separated metadata keys required by doctor --check-recipes (e.g. source,serves)')
            [CompletionResult]::new('--ignored-ingredients-file', '--ignored-ingredients-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON list of ingredients that should not be tagged with ''@'' (checked by doctor --check-recipes)')
            [CompletionResult]::new('--nav-file', '--nav-file', [CompletionResultType]::ParameterName, 'optional Zensical (zensical.toml) or MkDocs (mkdocs.yml) nav file to sync rendered recipes into')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output doctor report in JSON format')
            [CompletionResult]::new('--check-recipes', '--check-recipes', [CompletionResultType]::ParameterName, 'Validate recipe syntax, companion images, required metadata, emoji mappings, and ignored ingredients')
            [CompletionResult]::new('--strict', '--strict', [CompletionResultType]::ParameterName, 'Treat warnings as unhealthy and exit non-zero')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'don''t actually render any markdown files just print to stdout passed')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'don''t actually render any markdown files just print to stdout passed')
            [CompletionResult]::new('-j', '-j', [CompletionResultType]::ParameterName, 'parse the recipe and display it in json format')
            [CompletionResult]::new('--jsonify', '--jsonify', [CompletionResultType]::ParameterName, 'parse the recipe and display it in json format')
            [CompletionResult]::new('--link-in-steps', '--link-in-steps', [CompletionResultType]::ParameterName, 'cross-link matching @ingredient references inside step directions')
            [CompletionResult]::new('--merge', '--merge', [CompletionResultType]::ParameterName, 'merge generated content into existing Markdown files without clobbering manual edits')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'force full overwrite of existing Markdown files even when --merge / merge is enabled')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'cook-docs;self-update' {
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Target a specific release version or tag (e.g. 0.9.1 or v0.9.1)')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('-i', '-i', [CompletionResultType]::ParameterName, 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")')
            [CompletionResult]::new('--ignore-file', '--ignore-file', [CompletionResultType]::ParameterName, 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'directory to search recursively within for recipes (default ".")')
            [CompletionResult]::new('--recipe-search-root', '--recipe-search-root', [CompletionResultType]::ParameterName, 'directory to search recursively within for recipes (default ".")')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")')
            [CompletionResult]::new('--log-level', '--log-level', [CompletionResultType]::ParameterName, 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")')
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")')
            [CompletionResult]::new('--template-files', '--template-files', [CompletionResultType]::ParameterName, 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")')
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'word wrap line length for recipe steps section (default 120)')
            [CompletionResult]::new('--word-wrap', '--word-wrap', [CompletionResultType]::ParameterName, 'word wrap line length for recipe steps section (default 120)')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'custom output directory to write generated markdown documentation')
            [CompletionResult]::new('--output-dir', '--output-dir', [CompletionResultType]::ParameterName, 'custom output directory to write generated markdown documentation')
            [CompletionResult]::new('--assets-dir', '--assets-dir', [CompletionResultType]::ParameterName, 'optional directory to copy companion recipe images into (slug-named)')
            [CompletionResult]::new('--image-ext', '--image-ext', [CompletionResultType]::ParameterName, 'optional image extension override for Info.ImageSlug (e.g. webp)')
            [CompletionResult]::new('--image-ext-map', '--image-ext-map', [CompletionResultType]::ParameterName, 'comma-separated source=target image extension mappings for Info.ImageSlug (e.g. jpg=webp,jpeg=webp)')
            [CompletionResult]::new('--convert-images', '--convert-images', [CompletionResultType]::ParameterName, 'convert matching companion images when copying to --assets-dir (supports ''webp'' via cwebp)')
            [CompletionResult]::new('--image-quality', '--image-quality', [CompletionResultType]::ParameterName, 'output image quality (1-100) for --convert-images (default 80)')
            [CompletionResult]::new('--emoji-file', '--emoji-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON file mapping ingredients and cookware to emojis')
            [CompletionResult]::new('--weights-file', '--weights-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON ingredient density file for toGrams volume-to-weight conversion')
            [CompletionResult]::new('--link-dirs', '--link-dirs', [CompletionResultType]::ParameterName, 'comma-separated recipe subdirectories eligible for linkRecipe cross-linking')
            [CompletionResult]::new('--required-metadata', '--required-metadata', [CompletionResultType]::ParameterName, 'comma-separated metadata keys required by doctor --check-recipes (e.g. source,serves)')
            [CompletionResult]::new('--ignored-ingredients-file', '--ignored-ingredients-file', [CompletionResultType]::ParameterName, 'optional YAML/TOML/JSON list of ingredients that should not be tagged with ''@'' (checked by doctor --check-recipes)')
            [CompletionResult]::new('--nav-file', '--nav-file', [CompletionResultType]::ParameterName, 'optional Zensical (zensical.toml) or MkDocs (mkdocs.yml) nav file to sync rendered recipes into')
            [CompletionResult]::new('--check', '--check', [CompletionResultType]::ParameterName, 'Check for available updates without installing')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Skip confirmation prompt before installing')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Skip confirmation prompt before installing')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'don''t actually render any markdown files just print to stdout passed')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'don''t actually render any markdown files just print to stdout passed')
            [CompletionResult]::new('-j', '-j', [CompletionResultType]::ParameterName, 'parse the recipe and display it in json format')
            [CompletionResult]::new('--jsonify', '--jsonify', [CompletionResultType]::ParameterName, 'parse the recipe and display it in json format')
            [CompletionResult]::new('--link-in-steps', '--link-in-steps', [CompletionResultType]::ParameterName, 'cross-link matching @ingredient references inside step directions')
            [CompletionResult]::new('--merge', '--merge', [CompletionResultType]::ParameterName, 'merge generated content into existing Markdown files without clobbering manual edits')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'force full overwrite of existing Markdown files even when --merge / merge is enabled')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'cook-docs;help' {
            [CompletionResult]::new('watch', 'watch', [CompletionResultType]::ParameterValue, 'Watch recipes and templates and re-render automatically on changes')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell autocompletions script')
            [CompletionResult]::new('man', 'man', [CompletionResultType]::ParameterValue, 'Generate UNIX roff man page')
            [CompletionResult]::new('doctor', 'doctor', [CompletionResultType]::ParameterValue, 'Validate configuration, inputs, templates, and output permissions')
            [CompletionResult]::new('self-update', 'self-update', [CompletionResultType]::ParameterValue, 'Check for and install the latest cook-docs release from GitHub')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'cook-docs;help;watch' {
            break
        }
        'cook-docs;help;completions' {
            break
        }
        'cook-docs;help;man' {
            break
        }
        'cook-docs;help;doctor' {
            break
        }
        'cook-docs;help;self-update' {
            break
        }
        'cook-docs;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
