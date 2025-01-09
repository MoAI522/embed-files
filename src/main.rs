use cli::parse_cli;
use std::error::Error;

mod cli;
mod error;
mod template;
mod warning;

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
    let template = template::Template::from_file(&cli.template_path)?;

    // デバッグ出力（後で削除）
    for line in template.lines() {
        println!("{:?}", line);
    }

    Ok(())
}
