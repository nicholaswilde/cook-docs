
use builtin;
use str;

set edit:completion:arg-completer[cook-docs] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'cook-docs'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'cook-docs'= {
            cand -C 'path to configuration file (TOML, YAML, or JSON)'
            cand --config 'path to configuration file (TOML, YAML, or JSON)'
            cand -i 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")'
            cand --ignore-file 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")'
            cand -c 'directory to search recursively within for recipes (default ".")'
            cand --recipe-search-root 'directory to search recursively within for recipes (default ".")'
            cand -l 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")'
            cand --log-level 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")'
            cand -t 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")'
            cand --template-files 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")'
            cand -w 'word wrap line length for recipe steps section (default 120)'
            cand --word-wrap 'word wrap line length for recipe steps section (default 120)'
            cand -o 'custom output directory to write generated markdown documentation'
            cand --output-dir 'custom output directory to write generated markdown documentation'
            cand --debounce 'debounce delay in milliseconds for watch mode (default 300)'
            cand -v 'version for cook-docs'
            cand --version 'version for cook-docs'
            cand -d 'don''t actually render any markdown files just print to stdout passed'
            cand --dry-run 'don''t actually render any markdown files just print to stdout passed'
            cand -j 'parse the recipe and display it in json format'
            cand --jsonify 'parse the recipe and display it in json format'
            cand --watch 'watch recipes and templates and re-render automatically on changes'
            cand -h 'Print help'
            cand --help 'Print help'
            cand watch 'Watch recipes and templates and re-render automatically on changes'
            cand completions 'Generate shell autocompletions script'
            cand man 'Generate UNIX roff man page'
            cand doctor 'Validate configuration, inputs, templates, and output permissions'
            cand self-update 'Check for and install the latest cook-docs release from GitHub'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'cook-docs;watch'= {
            cand --debounce 'debounce delay in milliseconds for watch mode (default 300)'
            cand -C 'path to configuration file (TOML, YAML, or JSON)'
            cand --config 'path to configuration file (TOML, YAML, or JSON)'
            cand -i 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")'
            cand --ignore-file 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")'
            cand -c 'directory to search recursively within for recipes (default ".")'
            cand --recipe-search-root 'directory to search recursively within for recipes (default ".")'
            cand -l 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")'
            cand --log-level 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")'
            cand -t 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")'
            cand --template-files 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")'
            cand -w 'word wrap line length for recipe steps section (default 120)'
            cand --word-wrap 'word wrap line length for recipe steps section (default 120)'
            cand -o 'custom output directory to write generated markdown documentation'
            cand --output-dir 'custom output directory to write generated markdown documentation'
            cand -d 'don''t actually render any markdown files just print to stdout passed'
            cand --dry-run 'don''t actually render any markdown files just print to stdout passed'
            cand -j 'parse the recipe and display it in json format'
            cand --jsonify 'parse the recipe and display it in json format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'cook-docs;completions'= {
            cand -C 'path to configuration file (TOML, YAML, or JSON)'
            cand --config 'path to configuration file (TOML, YAML, or JSON)'
            cand -i 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")'
            cand --ignore-file 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")'
            cand -c 'directory to search recursively within for recipes (default ".")'
            cand --recipe-search-root 'directory to search recursively within for recipes (default ".")'
            cand -l 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")'
            cand --log-level 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")'
            cand -t 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")'
            cand --template-files 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")'
            cand -w 'word wrap line length for recipe steps section (default 120)'
            cand --word-wrap 'word wrap line length for recipe steps section (default 120)'
            cand -o 'custom output directory to write generated markdown documentation'
            cand --output-dir 'custom output directory to write generated markdown documentation'
            cand -d 'don''t actually render any markdown files just print to stdout passed'
            cand --dry-run 'don''t actually render any markdown files just print to stdout passed'
            cand -j 'parse the recipe and display it in json format'
            cand --jsonify 'parse the recipe and display it in json format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'cook-docs;man'= {
            cand -C 'path to configuration file (TOML, YAML, or JSON)'
            cand --config 'path to configuration file (TOML, YAML, or JSON)'
            cand -i 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")'
            cand --ignore-file 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")'
            cand -c 'directory to search recursively within for recipes (default ".")'
            cand --recipe-search-root 'directory to search recursively within for recipes (default ".")'
            cand -l 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")'
            cand --log-level 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")'
            cand -t 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")'
            cand --template-files 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")'
            cand -w 'word wrap line length for recipe steps section (default 120)'
            cand --word-wrap 'word wrap line length for recipe steps section (default 120)'
            cand -o 'custom output directory to write generated markdown documentation'
            cand --output-dir 'custom output directory to write generated markdown documentation'
            cand -d 'don''t actually render any markdown files just print to stdout passed'
            cand --dry-run 'don''t actually render any markdown files just print to stdout passed'
            cand -j 'parse the recipe and display it in json format'
            cand --jsonify 'parse the recipe and display it in json format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'cook-docs;doctor'= {
            cand -C 'path to configuration file (TOML, YAML, or JSON)'
            cand --config 'path to configuration file (TOML, YAML, or JSON)'
            cand -i 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")'
            cand --ignore-file 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")'
            cand -c 'directory to search recursively within for recipes (default ".")'
            cand --recipe-search-root 'directory to search recursively within for recipes (default ".")'
            cand -l 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")'
            cand --log-level 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")'
            cand -t 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")'
            cand --template-files 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")'
            cand -w 'word wrap line length for recipe steps section (default 120)'
            cand --word-wrap 'word wrap line length for recipe steps section (default 120)'
            cand -o 'custom output directory to write generated markdown documentation'
            cand --output-dir 'custom output directory to write generated markdown documentation'
            cand --json 'Output doctor report in JSON format'
            cand -d 'don''t actually render any markdown files just print to stdout passed'
            cand --dry-run 'don''t actually render any markdown files just print to stdout passed'
            cand -j 'parse the recipe and display it in json format'
            cand --jsonify 'parse the recipe and display it in json format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'cook-docs;self-update'= {
            cand --version 'Target a specific release version or tag (e.g. 0.9.1 or v0.9.1)'
            cand -C 'path to configuration file (TOML, YAML, or JSON)'
            cand --config 'path to configuration file (TOML, YAML, or JSON)'
            cand -i 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")'
            cand --ignore-file 'filename to use as an ignore file to exclude recipe directories (default ".cookdocsignore")'
            cand -c 'directory to search recursively within for recipes (default ".")'
            cand --recipe-search-root 'directory to search recursively within for recipes (default ".")'
            cand -l 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")'
            cand --log-level 'level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default "info")'
            cand -t 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")'
            cand --template-files 'gotemplate file paths relative to each recipe directory from which documentation will be generated (default "recipe.md.gotmpl")'
            cand -w 'word wrap line length for recipe steps section (default 120)'
            cand --word-wrap 'word wrap line length for recipe steps section (default 120)'
            cand -o 'custom output directory to write generated markdown documentation'
            cand --output-dir 'custom output directory to write generated markdown documentation'
            cand --check 'Check for available updates without installing'
            cand -y 'Skip confirmation prompt before installing'
            cand --yes 'Skip confirmation prompt before installing'
            cand -d 'don''t actually render any markdown files just print to stdout passed'
            cand --dry-run 'don''t actually render any markdown files just print to stdout passed'
            cand -j 'parse the recipe and display it in json format'
            cand --jsonify 'parse the recipe and display it in json format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'cook-docs;help'= {
            cand watch 'Watch recipes and templates and re-render automatically on changes'
            cand completions 'Generate shell autocompletions script'
            cand man 'Generate UNIX roff man page'
            cand doctor 'Validate configuration, inputs, templates, and output permissions'
            cand self-update 'Check for and install the latest cook-docs release from GitHub'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'cook-docs;help;watch'= {
        }
        &'cook-docs;help;completions'= {
        }
        &'cook-docs;help;man'= {
        }
        &'cook-docs;help;doctor'= {
        }
        &'cook-docs;help;self-update'= {
        }
        &'cook-docs;help;help'= {
        }
    ]
    $completions[$command]
}
