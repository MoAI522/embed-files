mod cli;
mod eftemplate;
mod error;
mod executor;
mod language_mapping;
mod path_resolver;
mod template;
mod warning;

use cli::parse_cli;
use std::error::Error;

fn main() {
    let cli = parse_cli();
    let mut warnings = warning::Warnings::new();
    let debug = cli.debug;

    if let Err(err) = run(cli, &mut warnings) {
        eprintln!("Error: {}", err);
        if debug {
            if let Some(source) = err.source() {
                eprintln!("\nCaused by:");
                let mut source = source;
                while let Some(next) = source.source() {
                    eprintln!("    {}", source);
                    source = next;
                }
                eprintln!("    {}", source);
            }
        }
        warnings.print_all();
        std::process::exit(1);
    }

    warnings.print_all();
}

fn run(cli: cli::Cli, warnings: &mut warning::Warnings) -> error::Result<()> {
    executor::execute(cli, &mut std::io::stdout(), warnings)
}
