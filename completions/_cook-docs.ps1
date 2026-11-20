
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
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'version for cook-docs')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'version for cook-docs')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'don''t actually render any markdown files just print to stdout passed')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'don''t actually render any markdown files just print to stdout passed')
            [CompletionResult]::new('-j', '-j', [CompletionResultType]::ParameterName, 'parse the recipe and display it in json format')
            [CompletionResult]::new('--jsonify', '--jsonify', [CompletionResultType]::ParameterName, 'parse the recipe and display it in json format')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell autocompletions script')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'cook-docs;completions' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'cook-docs;help' {
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell autocompletions script')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'cook-docs;help;completions' {
            break
        }
        'cook-docs;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
