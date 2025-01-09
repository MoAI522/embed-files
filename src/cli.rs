use clap::Parser;

/// プロンプトテンプレート内のファイル内容を展開し、LLMへの入力に適した形式に変換するCLIツール
#[derive(Parser)]
#[command(name = "ef")]
#[command(author = "moai")]
#[command(version = "0.1.0")]
#[command(about = "Expand files in prompt templates for LLM input")]
#[command(
    long_about = "This tool expands files and directories specified in prompt templates into a format suitable for LLM input. It supports both glob patterns (#ef) and regex patterns (#efr) for file selection. Output formatting can be customized using .eftemplate files."
)]
pub struct Cli {
    /// Path to the prompt template file
    ///
    /// You can expand the contents of all matching files in place using the following directives in the template:
    ///
    /// - #ef <glob_pattern>: File selection using glob patterns
    ///
    /// - #efr <regex_pattern>: File selection using regular expressions
    ///
    /// File path resolution follows these rules:
    ///
    /// - Relative paths: Resolved relative to the template file's directory
    ///
    /// - Absolute paths: Used as specified
    ///
    /// - Binary files are excluded (files containing NULL bytes in the first 1024 bytes)
    ///
    /// In the output, file paths are displayed relative to the directory where the command is executed.
    #[arg(value_name = "TEMPLATE")]
    #[arg(help_heading = "Arguments")]
    pub template_path: String,

    /// Display debug information
    /// Shows detailed cause information when errors occur
    #[arg(short, long)]
    #[arg(help_heading = "Options")]
    pub debug: bool,
}
pub fn parse_cli() -> Cli {
    Cli::parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use similar_asserts::assert_eq;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    #[test]
    fn test_cli_debug_flag() {
        let args = vec!["ef", "template.txt", "--debug"];
        let cli = Cli::parse_from(args);
        assert!(cli.debug);
        assert_eq!(cli.template_path, "template.txt");
    }

    #[test]
    fn test_cli_without_debug() {
        let args = vec!["ef", "template.txt"];
        let cli = Cli::parse_from(args);
        assert!(!cli.debug);
        assert_eq!(cli.template_path, "template.txt");
    }
}
