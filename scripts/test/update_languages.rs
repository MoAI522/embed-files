use super::*;
use similar_asserts::assert_eq;
use std::fs;
use tempfile::TempDir;

// テスト用のYAMLデータ
const TEST_YAML: &str = r#"
Rust:
    extensions:
        - ".rs"
        - ".rs.in"
TypeScript:
    extensions:
        - ".ts"
        - ".tsx"
Python:
    extensions:
        - ".py"
        - ".pyi"
NoExtension:
    type: programming
"#;

#[test]
fn test_generate_mappings_basic() {
    let mappings = generate_mappings(TEST_YAML).unwrap();

    let expected_mappings = {
        let mut map = BTreeMap::new();
        map.insert("rs".to_string(), "rust".to_string());
        map.insert("rs.in".to_string(), "rust".to_string());
        map.insert("ts".to_string(), "typescript".to_string());
        map.insert("tsx".to_string(), "typescript".to_string());
        map.insert("py".to_string(), "python".to_string());
        map.insert("pyi".to_string(), "python".to_string());
        ExtensionMappings(map)
    };

    assert_eq!(mappings.0, expected_mappings.0);
}

#[test]
fn test_generate_mappings_empty_extensions() {
    let yaml = r#"
Language1:
    extensions: []
Language2:
    type: programming
"#;
    let mappings = generate_mappings(yaml).unwrap();
    assert!(mappings.0.is_empty());
}

#[test]
fn test_generate_mappings_invalid_yaml() {
    let yaml = "invalid: - yaml: content";
    assert!(generate_mappings(yaml).is_err());
}

#[test]
fn test_save_ron_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let assets_dir = temp_dir.path().join("assets");

    // テスト用のマッピングデータ
    let mut mappings = BTreeMap::new();
    mappings.insert("rs".to_string(), "rust".to_string());
    mappings.insert("py".to_string(), "python".to_string());
    let mappings = ExtensionMappings(mappings);

    // 環境変数を一時的に設定
    temp_env::with_var(
        "CARGO_MANIFEST_DIR",
        Some(temp_dir.path().to_str().unwrap()),
        || save_ron_file(&mappings),
    )?;

    // 生成されたRONファイルを検証
    let ron_content = fs::read_to_string(assets_dir.join("language_mappings.ron"))?;
    let parsed_mappings: ExtensionMappings = ron::from_str(&ron_content)?;

    assert_eq!(parsed_mappings.0, mappings.0);
    Ok(())
}

#[test]
fn test_extension_trimming() {
    let yaml = r#"
Language:
    extensions:
        - ".ext"
        - "ext2"
"#;
    let mappings = generate_mappings(yaml).unwrap();

    assert_eq!(mappings.0.get("ext"), Some(&"language".to_string()));
    assert_eq!(mappings.0.get("ext2"), Some(&"language".to_string()));
}

// モックを使用したHTTPリクエストのテスト
#[cfg(test)]
mod http_tests {
    use super::*;
    use mockito::{mock, Server};

    #[test]
    fn test_fetch_languages_yaml() {
        let mut server = Server::new();
        let mock = mock("GET", "/linguist/languages.yml")
            .with_status(200)
            .with_body(TEST_YAML)
            .create_async()
            .await;

        let original_url = LINGUIST_URL;
        let test_url = format!("{}/linguist/languages.yml", server.url());

        // URLを一時的に差し替え
        temp_env::with_var("LINGUIST_URL", Some(&test_url), || {
            let result = fetch_languages_yaml();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TEST_YAML);
        });

        mock.assert();
    }

    #[test]
    fn test_fetch_languages_yaml_error() {
        let mut server = Server::new();
        let mock = mock("GET", "/linguist/languages.yml")
            .with_status(404)
            .create_async()
            .await;

        let test_url = format!("{}/linguist/languages.yml", server.url());

        temp_env::with_var("LINGUIST_URL", Some(&test_url), || {
            let result = fetch_languages_yaml();
            assert!(result.is_err());
        });

        mock.assert();
    }
}
