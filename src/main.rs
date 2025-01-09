use cli::parse_cli;
use std::error::Error;

mod cli;
mod error;
mod path_resolver;
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
    let mut resolver = path_resolver::PathResolver::new(&cli.template_path);

    // テンプレートの各行を処理
    for line in template.lines() {
        match line {
            template::TemplateLine::Text(text) => {
                println!("{}", text);
            }
            template::TemplateLine::Directive(directive) => {
                match directive {
                    template::Directive::Glob(pattern) => {
                        let paths = resolver.resolve_glob(&pattern)?;
                        for path in paths {
                            // TODO: .eftemplateの処理を実装したら、
                            // ここでファイル内容を適切なフォーマットで出力する
                            if let Some(path_str) = path.to_str() {
                                println!("Found file: {}", path_str);
                            }
                        }
                    }
                    template::Directive::Regex(pattern) => {
                        let paths = resolver.resolve_regex(&pattern)?;
                        for path in paths {
                            // TODO: .eftemplateの処理を実装したら、
                            // ここでファイル内容を適切なフォーマットで出力する
                            if let Some(path_str) = path.to_str() {
                                println!("Found file: {}", path_str);
                            }
                        }
                    }
                }
            }
        }
    }

    // PathResolverからの警告を統合
    warnings.extend(resolver.take_warnings());

    Ok(())
}
