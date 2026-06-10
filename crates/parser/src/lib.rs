use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRecord {
    pub source_file: String,
    pub source_dir: String,
    pub line: usize,
    pub import_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanOptions {
    pub include_extensions: Vec<String>,
    pub ignore: Vec<String>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            include_extensions: vec![
                "ts".to_string(),
                "tsx".to_string(),
                "js".to_string(),
                "jsx".to_string(),
            ],
            ignore: vec![
                "**/node_modules/**".to_string(),
                "**/dist/**".to_string(),
                "**/build/**".to_string(),
                "**/coverage/**".to_string(),
                "**/target/**".to_string(),
            ],
        }
    }
}

pub fn collect_imports(root: &Path) -> io::Result<Vec<ImportRecord>> {
    collect_imports_with_options(root, &ScanOptions::default())
}

pub fn collect_imports_with_options(
    root: &Path,
    options: &ScanOptions,
) -> io::Result<Vec<ImportRecord>> {
    let mut files = Vec::new();
    collect_ts_like_files(root, root, options, &mut files)?;

    let mut imports = Vec::new();
    for file in files {
        let content = fs::read_to_string(&file)?;
        let relative = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        let source_dir = Path::new(&relative)
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();

        for (idx, line) in content.lines().enumerate() {
            if let Some(import_path) = extract_import_path(line) {
                imports.push(ImportRecord {
                    source_file: relative.clone(),
                    source_dir: source_dir.clone(),
                    line: idx + 1,
                    import_path,
                });
            }
        }
    }

    Ok(imports)
}

fn collect_ts_like_files(
    root: &Path,
    dir: &Path,
    options: &ScanOptions,
    acc: &mut Vec<PathBuf>,
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        if should_ignore(&relative, &options.ignore) {
            continue;
        }

        if path.is_dir() {
            collect_ts_like_files(root, &path, options, acc)?;
            continue;
        }

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if options
                .include_extensions
                .iter()
                .any(|include_ext| include_ext == ext)
            {
                acc.push(path);
            }
        }
    }

    Ok(())
}

fn should_ignore(relative_path: &str, patterns: &[String]) -> bool {
    let normalized = relative_path.trim_matches('/');

    patterns
        .iter()
        .any(|pattern| matches_ignore_pattern(normalized, pattern))
}

fn matches_ignore_pattern(path: &str, pattern: &str) -> bool {
    let normalized_pattern = pattern.trim_matches('/');

    if normalized_pattern.starts_with("**/") && normalized_pattern.ends_with("/**") {
        let segment = normalized_pattern
            .trim_start_matches("**/")
            .trim_end_matches("/**");
        return path == segment
            || path.starts_with(&format!("{segment}/"))
            || path.contains(&format!("/{segment}/"));
    }

    if let Some(prefix) = normalized_pattern.strip_suffix("/**") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }

    path == normalized_pattern || path.starts_with(&format!("{normalized_pattern}/"))
}

fn extract_import_path(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if trimmed.starts_with("import ") || trimmed.starts_with("export ") {
        if let Some(path) = extract_path_after_keyword(trimmed, " from ") {
            return Some(path);
        }
        if let Some(path) = extract_first_quoted(trimmed) {
            return Some(path);
        }
    }

    if trimmed.contains("require(") {
        return extract_path_after_require(trimmed);
    }

    None
}

fn extract_path_after_keyword(line: &str, needle: &str) -> Option<String> {
    let (_, rest) = line.split_once(needle)?;
    extract_first_quoted(rest)
}

fn extract_path_after_require(line: &str) -> Option<String> {
    let (_, rest) = line.split_once("require(")?;
    extract_first_quoted(rest)
}

fn extract_first_quoted(input: &str) -> Option<String> {
    let start = input.find('\'').or_else(|| input.find('"'))?;
    let quote = input.as_bytes()[start] as char;
    let after = &input[start + 1..];
    let end = after.find(quote)?;
    Some(after[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::extract_import_path;

    #[test]
    fn extracts_from_import_from() {
        let line = "import { x } from './server/usecase'";
        assert_eq!(
            extract_import_path(line).as_deref(),
            Some("./server/usecase")
        );
    }

    #[test]
    fn extracts_from_side_effect_import() {
        let line = "import './polyfill'";
        assert_eq!(extract_import_path(line).as_deref(), Some("./polyfill"));
    }

    #[test]
    fn extracts_from_require() {
        let line = "const x = require('../server/x')";
        assert_eq!(extract_import_path(line).as_deref(), Some("../server/x"));
    }
}
