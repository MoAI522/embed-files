use clap::Parser;

pub mod cli;
pub mod eftemplate;
pub mod error;
mod executor;
pub mod path_resolver;
pub mod template;
pub mod warning;

#[doc(hidden)]
pub fn run_with_args(args: Vec<String>) -> error::Result<String> {
    let mut output = Vec::new();
    let mut warnings = warning::Warnings::new();
    let cli = cli::Cli::parse_from(args);

    executor::execute(cli, &mut output, &mut warnings)?;

    for warning in warnings.into_iter() {
        eprintln!("Warning: {}", warning);
    }

    Ok(String::from_utf8(output)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?)
}
