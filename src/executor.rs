use crate::cli;
use crate::eftemplate;
use crate::error;
use crate::path_resolver;
use crate::template;
use crate::warning;
use std::io::Write;

/// コマンドの実行ロジック
pub fn execute<W: Write>(
    cli: cli::Cli,
    writer: &mut W,
    warnings: &mut warning::Warnings,
) -> error::Result<()> {
    // テンプレートを読み込み
    let template = template::Template::from_file(&cli.template_path)?;
    let mut resolver = path_resolver::PathResolver::new(&cli.template_path);
    let eftemplate = eftemplate::EfTemplate::find_and_load(&cli.template_path)?;

    // テンプレートの各行を処理
    for line in template.lines() {
        match line {
            template::TemplateLine::Text(text) => {
                writeln!(writer, "{}", text)?;
            }
            template::TemplateLine::Directive(directive) => match directive {
                template::Directive::Glob(pattern) => {
                    let paths = resolver.resolve_glob(&pattern)?;
                    for path in paths {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let formatted = eftemplate.format(&path, &content);
                            write!(writer, "{}", formatted)?;
                        }
                    }
                }
                template::Directive::Regex(pattern) => {
                    let paths = resolver.resolve_regex(&pattern)?;
                    for path in paths {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let formatted = eftemplate.format(&path, &content);
                            write!(writer, "{}", formatted)?;
                        }
                    }
                }
            },
        }
    }

    // PathResolverからの警告を統合
    warnings.extend(resolver.take_warnings());

    Ok(())
}
