use crate::cli;
use crate::eftemplate;
use crate::error;
use crate::path_resolver;
use crate::template;
use crate::warning;
use std::io::Write;

pub fn execute<W: Write>(
    cli: cli::Cli,
    writer: &mut W,
    warnings: &mut warning::Warnings,
) -> error::Result<()> {
    let template = template::Template::from_file(&cli.template_path)?;
    let mut resolver = path_resolver::PathResolver::new()?;
    let eftemplate = eftemplate::EfTemplate::find_and_load(&cli.template_path)?;

    for line in template.lines() {
        match line {
            template::TemplateLine::Text(text) => {
                writeln!(writer, "{}", text)?;
            }
            template::TemplateLine::Directive(directive) => match directive {
                template::Directive::Glob(pattern) => {
                    let paths = resolver.resolve_glob(&pattern)?;
                    for path in paths {
                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                let formatted = eftemplate.format(&path, &content);
                                writeln!(writer, "{}", formatted)?;
                            }
                            Err(e) => {
                                eprintln!("Failed to read file {:?}: {}", path, e);
                                warnings
                                    .push(warning::Warning::FileNotFound { path: path.clone() });
                            }
                        }
                    }
                }
                template::Directive::Regex(pattern) => {
                    let paths = resolver.resolve_regex(&pattern)?;
                    for path in paths {
                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                let formatted = eftemplate.format(&path, &content);
                                writeln!(writer, "{}", formatted)?;
                            }
                            Err(e) => {
                                warnings
                                    .push(warning::Warning::FileNotFound { path: path.clone() });
                                eprintln!("Failed to read file {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            },
        }
    }

    warnings.extend(resolver.take_warnings());

    Ok(())
}
