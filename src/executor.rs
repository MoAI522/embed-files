use crate::cli;
use crate::eftemplate;
use crate::error;
use crate::error::Error;
use crate::path_resolver;
use crate::template;
use crate::warning;
use std::io::Write;
use std::path::PathBuf;

pub fn execute<W: Write>(
    cli: cli::Cli,
    writer: &mut W,
    warnings: &mut warning::Warnings,
) -> error::Result<()> {
    let template_path = PathBuf::from(&cli.template_path)
        .canonicalize()
        .map_err(|e| Error::IoError(e))?;

    let template = template::Template::from_file(&template_path)?;
    let mut resolver = path_resolver::PathResolver::new()?;
    let eftemplate = eftemplate::EfTemplate::find_and_load(&template_path)?;

    for line in template.lines() {
        match line {
            template::TemplateLine::Text(text) => {
                writeln!(writer, "{}", text)?;
            }
            template::TemplateLine::Directive(directive) => {
                let paths = match &directive {
                    template::Directive::Glob(pattern) => resolver.resolve_glob(pattern)?,
                    template::Directive::Regex(pattern) => resolver.resolve_regex(pattern)?,
                };

                for path in paths {
                    match std::fs::read_to_string(&path) {
                        Ok(content) => {
                            let formatted = eftemplate.format(&path, &content);
                            writeln!(writer, "{}", formatted)?;
                        }
                        Err(e) => {
                            warnings.push(warning::Warning::FileNotFound { path: path.clone() });
                            eprintln!("Failed to read file {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
    }

    warnings.extend(resolver.take_warnings());
    Ok(())
}
