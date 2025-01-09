use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;

const LINGUIST_URL: &str = "https://raw.githubusercontent.com/github-linguist/linguist/refs/heads/main/lib/linguist/languages.yml";

#[derive(Debug, Deserialize)]
struct LanguageInfo {
    extensions: Option<Vec<String>>,
    #[serde(flatten)]
    _other: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Serialize)]
struct ExtensionMappings(BTreeMap<String, String>);

fn fetch_languages_yaml() -> Result<String> {
    println!("{}", "Fetching languages.yml from GitHub...".cyan());
    let response = reqwest::blocking::get(LINGUIST_URL).context("Failed to fetch languages.yml")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch languages.yml: HTTP {}", response.status());
    }

    response.text().context("Failed to read response body")
}

fn generate_mappings(yaml_content: &str) -> Result<ExtensionMappings> {
    println!("{}", "\nGenerating extension mappings...".cyan());
    let languages: HashMap<String, LanguageInfo> =
        serde_yaml::from_str(yaml_content).context("Failed to parse languages.yml")?;

    let mut mappings = BTreeMap::new();

    for (name, info) in languages {
        if let Some(extensions) = info.extensions {
            let syntax_name = name.to_lowercase();
            for ext in extensions {
                let ext = ext.trim_start_matches('.');
                mappings.insert(ext.to_string(), syntax_name.clone());
            }
        }
    }

    // NOTE: 普通に変なマッピング紛れ込むので、手動で修正 意味ない！
    mappings.insert("txt".to_string(), "plaintext".to_string());
    mappings.insert("tsx".to_string(), "typescript".to_string());
    mappings.insert("md".to_string(), "markdown".to_string());
    mappings.insert("json".to_string(), "json".to_string());
    mappings.insert("yml".to_string(), "yaml".to_string());
    mappings.insert("rs".to_string(), "rust".to_string());
    mappings.insert("php".to_string(), "php".to_string());

    println!("{} {}", "Total mappings generated:".green(), mappings.len());
    Ok(ExtensionMappings(mappings))
}

fn save_ron_file(mappings: &ExtensionMappings) -> Result<()> {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assets_dir = project_root.join("assets");

    // Create assets directory if it doesn't exist
    fs::create_dir_all(&assets_dir).context("Failed to create assets directory")?;

    let ron_path = assets_dir.join("language_mappings.ron");
    println!("{} {}", "\nSaving RON file to:".cyan(), ron_path.display());

    let ron_data = ron::ser::to_string_pretty(
        mappings,
        ron::ser::PrettyConfig::new()
            .enumerate_arrays(true)
            .new_line("\n".to_string()),
    )
    .context("Failed to serialize to RON")?;

    fs::write(&ron_path, ron_data).context("Failed to write RON file")?;

    println!("{}", "RON file generated successfully!".green());
    Ok(())
}

fn main() -> Result<()> {
    let yaml_content = fetch_languages_yaml()?;
    let mappings = generate_mappings(&yaml_content)?;
    save_ron_file(&mappings)?;
    Ok(())
}
