# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_cook_docs_global_optspecs
    string join \n v/version C/config= d/dry-run j/jsonify i/ignore-file= c/recipe-search-root= l/log-level= t/template-files= w/word-wrap= o/output-dir= assets-dir= image-ext= emoji-file= weights-file= link-dirs= required-metadata= merge force watch debounce= h/help
end

function __fish_cook_docs_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_cook_docs_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_cook_docs_using_subcommand
    set -l cmd (__fish_cook_docs_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c cook-docs -n "__fish_cook_docs_needs_command" -s C -l config -d 'path to configuration file (TOML, YAML, or JSON)' -r
complete -c cook-docs -n "__fish_cook_docs_needs_command" -s i -l ignore-file -d 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")' -r
complete -c cook-docs -n "__fish_cook_docs_needs_command" -s c -l recipe-search-root -d 'directory to search recursively within for recipes (default ".")' -r
complete -c cook-docs -n "__fish_cook_docs_needs_command" -s l -l log-level -d 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")' -r
complete -c cook-docs -n "__fish_cook_docs_needs_command" -s t -l template-files -d 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")' -r
complete -c cook-docs -n "__fish_cook_docs_needs_command" -s w -l word-wrap -d 'word wrap line length for recipe steps section (default 120)' -r
complete -c cook-docs -n "__fish_cook_docs_needs_command" -s o -l output-dir -d 'custom output directory to write generated markdown documentation' -r
complete -c cook-docs -n "__fish_cook_docs_needs_command" -l assets-dir -d 'optional directory to copy companion recipe images into (slug-named)' -r
complete -c cook-docs -n "__fish_cook_docs_needs_command" -l image-ext -d 'optional image extension override for Info.ImageSlug (e.g. webp)' -r
complete -c cook-docs -n "__fish_cook_docs_needs_command" -l emoji-file -d 'optional YAML/TOML/JSON file mapping ingredients and cookware to emojis' -r
complete -c cook-docs -n "__fish_cook_docs_needs_command" -l weights-file -d 'optional YAML/TOML/JSON ingredient density file for toGrams volume-to-weight conversion' -r
complete -c cook-docs -n "__fish_cook_docs_needs_command" -l link-dirs -d 'comma-separated recipe subdirectories eligible for linkRecipe cross-linking' -r
complete -c cook-docs -n "__fish_cook_docs_needs_command" -l required-metadata -d 'comma-separated metadata keys required by doctor --check-recipes (e.g. source,serves)' -r
complete -c cook-docs -n "__fish_cook_docs_needs_command" -l debounce -d 'debounce delay in milliseconds for watch mode (default 300)' -r
complete -c cook-docs -n "__fish_cook_docs_needs_command" -s v -l version -d 'version for cook-docs'
complete -c cook-docs -n "__fish_cook_docs_needs_command" -s d -l dry-run -d 'don\'t actually render any markdown files just print to stdout passed'
complete -c cook-docs -n "__fish_cook_docs_needs_command" -s j -l jsonify -d 'parse the recipe and display it in json format'
complete -c cook-docs -n "__fish_cook_docs_needs_command" -l merge -d 'merge generated content into existing Markdown files without clobbering manual edits'
complete -c cook-docs -n "__fish_cook_docs_needs_command" -l force -d 'force full overwrite of existing Markdown files even when --merge / merge is enabled'
complete -c cook-docs -n "__fish_cook_docs_needs_command" -l watch -d 'watch recipes and templates and re-render automatically on changes'
complete -c cook-docs -n "__fish_cook_docs_needs_command" -s h -l help -d 'Print help'
complete -c cook-docs -n "__fish_cook_docs_needs_command" -a "watch" -d 'Watch recipes and templates and re-render automatically on changes'
complete -c cook-docs -n "__fish_cook_docs_needs_command" -a "completions" -d 'Generate shell autocompletions script'
complete -c cook-docs -n "__fish_cook_docs_needs_command" -a "man" -d 'Generate UNIX roff man page'
complete -c cook-docs -n "__fish_cook_docs_needs_command" -a "doctor" -d 'Validate configuration, inputs, templates, and output permissions'
complete -c cook-docs -n "__fish_cook_docs_needs_command" -a "self-update" -d 'Check for and install the latest cook-docs release from GitHub'
complete -c cook-docs -n "__fish_cook_docs_needs_command" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -l debounce -d 'debounce delay in milliseconds for watch mode (default 300)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -s C -l config -d 'path to configuration file (TOML, YAML, or JSON)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -s i -l ignore-file -d 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -s c -l recipe-search-root -d 'directory to search recursively within for recipes (default ".")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -s l -l log-level -d 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -s t -l template-files -d 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -s w -l word-wrap -d 'word wrap line length for recipe steps section (default 120)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -s o -l output-dir -d 'custom output directory to write generated markdown documentation' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -l assets-dir -d 'optional directory to copy companion recipe images into (slug-named)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -l image-ext -d 'optional image extension override for Info.ImageSlug (e.g. webp)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -l emoji-file -d 'optional YAML/TOML/JSON file mapping ingredients and cookware to emojis' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -l weights-file -d 'optional YAML/TOML/JSON ingredient density file for toGrams volume-to-weight conversion' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -l link-dirs -d 'comma-separated recipe subdirectories eligible for linkRecipe cross-linking' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -l required-metadata -d 'comma-separated metadata keys required by doctor --check-recipes (e.g. source,serves)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -s d -l dry-run -d 'don\'t actually render any markdown files just print to stdout passed'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -s j -l jsonify -d 'parse the recipe and display it in json format'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -l merge -d 'merge generated content into existing Markdown files without clobbering manual edits'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -l force -d 'force full overwrite of existing Markdown files even when --merge / merge is enabled'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand watch" -s h -l help -d 'Print help'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -s C -l config -d 'path to configuration file (TOML, YAML, or JSON)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -s i -l ignore-file -d 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -s c -l recipe-search-root -d 'directory to search recursively within for recipes (default ".")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -s l -l log-level -d 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -s t -l template-files -d 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -s w -l word-wrap -d 'word wrap line length for recipe steps section (default 120)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -s o -l output-dir -d 'custom output directory to write generated markdown documentation' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -l assets-dir -d 'optional directory to copy companion recipe images into (slug-named)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -l image-ext -d 'optional image extension override for Info.ImageSlug (e.g. webp)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -l emoji-file -d 'optional YAML/TOML/JSON file mapping ingredients and cookware to emojis' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -l weights-file -d 'optional YAML/TOML/JSON ingredient density file for toGrams volume-to-weight conversion' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -l link-dirs -d 'comma-separated recipe subdirectories eligible for linkRecipe cross-linking' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -l required-metadata -d 'comma-separated metadata keys required by doctor --check-recipes (e.g. source,serves)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -s d -l dry-run -d 'don\'t actually render any markdown files just print to stdout passed'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -s j -l jsonify -d 'parse the recipe and display it in json format'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -l merge -d 'merge generated content into existing Markdown files without clobbering manual edits'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -l force -d 'force full overwrite of existing Markdown files even when --merge / merge is enabled'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand completions" -s h -l help -d 'Print help'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -s C -l config -d 'path to configuration file (TOML, YAML, or JSON)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -s i -l ignore-file -d 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -s c -l recipe-search-root -d 'directory to search recursively within for recipes (default ".")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -s l -l log-level -d 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -s t -l template-files -d 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -s w -l word-wrap -d 'word wrap line length for recipe steps section (default 120)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -s o -l output-dir -d 'custom output directory to write generated markdown documentation' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -l assets-dir -d 'optional directory to copy companion recipe images into (slug-named)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -l image-ext -d 'optional image extension override for Info.ImageSlug (e.g. webp)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -l emoji-file -d 'optional YAML/TOML/JSON file mapping ingredients and cookware to emojis' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -l weights-file -d 'optional YAML/TOML/JSON ingredient density file for toGrams volume-to-weight conversion' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -l link-dirs -d 'comma-separated recipe subdirectories eligible for linkRecipe cross-linking' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -l required-metadata -d 'comma-separated metadata keys required by doctor --check-recipes (e.g. source,serves)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -s d -l dry-run -d 'don\'t actually render any markdown files just print to stdout passed'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -s j -l jsonify -d 'parse the recipe and display it in json format'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -l merge -d 'merge generated content into existing Markdown files without clobbering manual edits'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -l force -d 'force full overwrite of existing Markdown files even when --merge / merge is enabled'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand man" -s h -l help -d 'Print help'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -s C -l config -d 'path to configuration file (TOML, YAML, or JSON)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -s i -l ignore-file -d 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -s c -l recipe-search-root -d 'directory to search recursively within for recipes (default ".")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -s l -l log-level -d 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -s t -l template-files -d 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -s w -l word-wrap -d 'word wrap line length for recipe steps section (default 120)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -s o -l output-dir -d 'custom output directory to write generated markdown documentation' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -l assets-dir -d 'optional directory to copy companion recipe images into (slug-named)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -l image-ext -d 'optional image extension override for Info.ImageSlug (e.g. webp)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -l emoji-file -d 'optional YAML/TOML/JSON file mapping ingredients and cookware to emojis' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -l weights-file -d 'optional YAML/TOML/JSON ingredient density file for toGrams volume-to-weight conversion' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -l link-dirs -d 'comma-separated recipe subdirectories eligible for linkRecipe cross-linking' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -l required-metadata -d 'comma-separated metadata keys required by doctor --check-recipes (e.g. source,serves)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -l json -d 'Output doctor report in JSON format'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -l check-recipes -d 'Validate recipe syntax, companion images, required metadata, and emoji mappings'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -s d -l dry-run -d 'don\'t actually render any markdown files just print to stdout passed'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -s j -l jsonify -d 'parse the recipe and display it in json format'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -l merge -d 'merge generated content into existing Markdown files without clobbering manual edits'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -l force -d 'force full overwrite of existing Markdown files even when --merge / merge is enabled'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand doctor" -s h -l help -d 'Print help'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -l version -d 'Target a specific release version or tag (e.g. 0.9.1 or v0.9.1)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -s C -l config -d 'path to configuration file (TOML, YAML, or JSON)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -s i -l ignore-file -d 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -s c -l recipe-search-root -d 'directory to search recursively within for recipes (default ".")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -s l -l log-level -d 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -s t -l template-files -d 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -s w -l word-wrap -d 'word wrap line length for recipe steps section (default 120)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -s o -l output-dir -d 'custom output directory to write generated markdown documentation' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -l assets-dir -d 'optional directory to copy companion recipe images into (slug-named)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -l image-ext -d 'optional image extension override for Info.ImageSlug (e.g. webp)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -l emoji-file -d 'optional YAML/TOML/JSON file mapping ingredients and cookware to emojis' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -l weights-file -d 'optional YAML/TOML/JSON ingredient density file for toGrams volume-to-weight conversion' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -l link-dirs -d 'comma-separated recipe subdirectories eligible for linkRecipe cross-linking' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -l required-metadata -d 'comma-separated metadata keys required by doctor --check-recipes (e.g. source,serves)' -r
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -l check -d 'Check for available updates without installing'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -s y -l yes -d 'Skip confirmation prompt before installing'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -s d -l dry-run -d 'don\'t actually render any markdown files just print to stdout passed'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -s j -l jsonify -d 'parse the recipe and display it in json format'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -l merge -d 'merge generated content into existing Markdown files without clobbering manual edits'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -l force -d 'force full overwrite of existing Markdown files even when --merge / merge is enabled'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand self-update" -s h -l help -d 'Print help'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand help; and not __fish_seen_subcommand_from watch completions man doctor self-update help" -f -a "watch" -d 'Watch recipes and templates and re-render automatically on changes'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand help; and not __fish_seen_subcommand_from watch completions man doctor self-update help" -f -a "completions" -d 'Generate shell autocompletions script'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand help; and not __fish_seen_subcommand_from watch completions man doctor self-update help" -f -a "man" -d 'Generate UNIX roff man page'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand help; and not __fish_seen_subcommand_from watch completions man doctor self-update help" -f -a "doctor" -d 'Validate configuration, inputs, templates, and output permissions'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand help; and not __fish_seen_subcommand_from watch completions man doctor self-update help" -f -a "self-update" -d 'Check for and install the latest cook-docs release from GitHub'
complete -c cook-docs -n "__fish_cook_docs_using_subcommand help; and not __fish_seen_subcommand_from watch completions man doctor self-update help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
