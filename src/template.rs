use crate::error::Result;
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum Directive {
    Glob(String),
    Regex(String),
}

#[derive(Debug, PartialEq)]
pub struct Template {
    lines: Vec<TemplateLine>,
}

#[derive(Debug, PartialEq)]
pub enum TemplateLine {
    Text(String),
    Directive(Directive),
}

impl Template {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self> {
        let mut lines = Vec::new();

        for (_i, line) in content.lines().enumerate() {
            let line = if let Some(parts) = line.strip_prefix('#') {
                let mut elements = parts.split_whitespace();

                if let Some(directive_name) = elements.next() {
                    let arguments = elements.collect::<Vec<_>>();

                    match directive_name {
                        "ef" => {
                            let argument = arguments.join(" ");
                            if argument.is_empty() {
                                TemplateLine::Text(line.to_string())
                            } else {
                                TemplateLine::Directive(Directive::Glob(argument))
                            }
                        }
                        "efr" => {
                            let argument = arguments.join(" ");
                            if argument.is_empty() {
                                TemplateLine::Text(line.to_string())
                            } else {
                                TemplateLine::Directive(Directive::Regex(argument))
                            }
                        }
                        _ => TemplateLine::Text(line.to_string()),
                    }
                } else {
                    // #だけの行の場合
                    TemplateLine::Text(line.to_string())
                }
            } else {
                // #で始まらない行の場合
                TemplateLine::Text(line.to_string())
            };
            lines.push(line);
        }

        Ok(Self { lines })
    }

    pub fn lines(&self) -> &[TemplateLine] {
        &self.lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_glob_directive() {
        let template = Template::parse("#ef src/*.rs").unwrap();
        assert_eq!(
            template.lines(),
            &[TemplateLine::Directive(Directive::Glob(
                "src/*.rs".to_string()
            ))]
        );
    }

    #[test]
    fn test_parse_regex_directive() {
        let template = Template::parse("#efr ^src/.*\\.rs$").unwrap();
        assert_eq!(
            template.lines(),
            &[TemplateLine::Directive(Directive::Regex(
                "^src/.*\\.rs$".to_string()
            ))]
        );
    }

    #[test]
    fn test_parse_mixed_content() {
        let content = "Here is the content:\n#ef src/*.rs\nMore text";
        let template = Template::parse(content).unwrap();
        assert_eq!(
            template.lines(),
            &[
                TemplateLine::Text("Here is the content:".to_string()),
                TemplateLine::Directive(Directive::Glob("src/*.rs".to_string())),
                TemplateLine::Text("More text".to_string()),
            ]
        );
    }

    #[test]
    fn test_unknown_directive() {
        let template = Template::parse("#unknown argument").unwrap();
        assert_eq!(
            template.lines(),
            &[TemplateLine::Text("#unknown argument".to_string())]
        );
    }

    #[test]
    fn test_empty_directive() {
        let template = Template::parse("#ef").unwrap();
        assert_eq!(template.lines(), &[TemplateLine::Text("#ef".to_string())]);
    }

    #[test]
    fn test_hash_only_line() {
        let template = Template::parse("#").unwrap();
        assert_eq!(template.lines(), &[TemplateLine::Text("#".to_string())]);
    }

    #[test]
    fn test_directive_with_multiple_arguments() {
        let template = Template::parse("#ef src/*.rs test/*.rs").unwrap();
        assert_eq!(
            template.lines(),
            &[TemplateLine::Directive(Directive::Glob(
                "src/*.rs test/*.rs".to_string()
            ))]
        );
    }
}
