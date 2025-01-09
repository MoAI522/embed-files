use clap::Parser;

#[derive(Parser)]
#[command(name = "ef")]
#[command(author = "moai")]
#[command(version = "0.1.0")]
#[command(about = "Expands file contents in prompt templates for LLM input", long_about = None)]
pub struct Cli {
    /// Path to prompt template file
    #[arg(value_name = "TEMPLATE")]
    pub template_path: String,

    /// Show debug information
    #[arg(short, long)]
    pub debug: bool,
}

pub fn parse_cli() -> Cli {
    Cli::parse()
}

#[cfg(test)]
mod tests {
    use super::*;

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
