use ron::de::from_str;
use serde::Deserialize;
use std::collections::HashMap;

// assets/language_mappings.ronをコンパイル時に埋め込み
const LANGUAGE_MAPPINGS_RON: &str = include_str!("../assets/language_mappings.ron");

#[derive(Debug, Deserialize)]
struct LanguageMappings(HashMap<String, String>);

// コンパイル時に変換できないためコンパイル後に一度だけ実行
static MAPPINGS: once_cell::sync::Lazy<HashMap<String, String>> =
    once_cell::sync::Lazy::new(|| {
        let LanguageMappings(mappings) =
            from_str(LANGUAGE_MAPPINGS_RON).expect("Failed to parse embedded language mappings");
        mappings
    });

pub fn get_language_for_extension(extension: &str) -> Option<String> {
    MAPPINGS.get(extension).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_mappings() {
        // 組み込まれたマッピングが正しく読み込めることを確認
        assert!(MAPPINGS.contains_key("rs")); // rustファイルの拡張子が含まれていることを確認
    }

    #[test]
    fn test_get_language() {
        assert_eq!(get_language_for_extension("rs").unwrap(), "rust");

        assert_eq!(get_language_for_extension("unknown"), None);
    }
}
