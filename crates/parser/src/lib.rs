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

        for (line, import_path) in extract_imports_from_content(&content) {
            imports.push(ImportRecord {
                source_file: relative.clone(),
                source_dir: source_dir.clone(),
                line,
                import_path,
            });
        }
    }

    Ok(imports)
}

fn extract_imports_from_content(content: &str) -> Vec<(usize, String)> {
    let mut imports = Vec::new();
    let mut pending_statement: Option<PendingStatement> = None;

    for (idx, line) in content.lines().enumerate() {
        let line_number = idx + 1;
        let trimmed = line.trim();

        if let Some(pending) = pending_statement.as_mut() {
            pending.statement.push(' ');
            pending.statement.push_str(trimmed);

            if let Some(import_path) = pending.extract_import_path() {
                imports.push((pending.start_line, import_path));
                pending_statement = None;
            }
            continue;
        }

        if starts_static_import_or_export(trimmed) {
            if let Some(import_path) = extract_static_import_path(trimmed) {
                imports.push((line_number, import_path));
            } else {
                pending_statement = Some(PendingStatement::new(
                    line_number,
                    trimmed.to_string(),
                    PendingStatementKind::Static,
                ));
            }
            continue;
        }

        if trimmed.contains("require(") {
            if let Some(import_path) = extract_require_path(trimmed) {
                imports.push((line_number, import_path));
            } else {
                pending_statement = Some(PendingStatement::new(
                    line_number,
                    trimmed.to_string(),
                    PendingStatementKind::RequireCall,
                ));
                continue;
            }
        }

        if trimmed.contains("import(") {
            if let Some(import_path) = extract_dynamic_import_path(trimmed) {
                imports.push((line_number, import_path));
            } else {
                pending_statement = Some(PendingStatement::new(
                    line_number,
                    trimmed.to_string(),
                    PendingStatementKind::DynamicImportCall,
                ));
            }
        }
    }

    imports
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingStatement {
    start_line: usize,
    statement: String,
    kind: PendingStatementKind,
}

impl PendingStatement {
    fn new(start_line: usize, statement: String, kind: PendingStatementKind) -> Self {
        Self {
            start_line,
            statement,
            kind,
        }
    }

    fn extract_import_path(&self) -> Option<String> {
        match self.kind {
            PendingStatementKind::Static => extract_static_import_path(&self.statement),
            PendingStatementKind::RequireCall => extract_require_path(&self.statement),
            PendingStatementKind::DynamicImportCall => extract_dynamic_import_path(&self.statement),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingStatementKind {
    Static,
    RequireCall,
    DynamicImportCall,
}

fn starts_static_import_or_export(line: &str) -> bool {
    line.starts_with("import ")
        || (line.starts_with("export ")
            && (line.contains(" from ")
                || line.starts_with("export {")
                || line.starts_with("export type {")
                || line.starts_with("export *")))
}

fn extract_static_import_path(statement: &str) -> Option<String> {
    let trimmed = statement.trim();

    if starts_static_import_or_export(trimmed) {
        if let Some(path) = extract_path_after_keyword(trimmed, " from ") {
            return Some(path);
        }
        if trimmed.starts_with("import ") {
            return extract_first_quoted(trimmed);
        }
    }

    None
}

fn extract_require_path(line: &str) -> Option<String> {
    extract_path_after_call(line, "require(")
}

fn extract_dynamic_import_path(line: &str) -> Option<String> {
    extract_path_after_call(line, "import(")
}

fn extract_path_after_call(line: &str, needle: &str) -> Option<String> {
    let (_, rest) = line.split_once(needle)?;
    extract_first_quoted(rest)
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

fn extract_path_after_keyword(line: &str, needle: &str) -> Option<String> {
    let (_, rest) = line.split_once(needle)?;
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
    use super::extract_imports_from_content;

    fn import_paths(content: &str) -> Vec<String> {
        extract_imports_from_content(content)
            .into_iter()
            .map(|(_, import_path)| import_path)
            .collect()
    }

    #[test]
    fn extracts_from_import_from() {
        let line = "import { x } from './server/usecase'";
        assert_eq!(import_paths(line), vec!["./server/usecase"]);
    }

    #[test]
    fn extracts_from_side_effect_import() {
        let line = "import './polyfill'";
        assert_eq!(import_paths(line), vec!["./polyfill"]);
    }

    #[test]
    fn extracts_from_require() {
        let line = "const x = require('../server/x')";
        assert_eq!(import_paths(line), vec!["../server/x"]);
    }

    #[test]
    fn extracts_from_multiline_import_from() {
        let content = r#"import {
  checkout
} from '../server/checkout';
"#;

        let imports = extract_imports_from_content(content);
        assert_eq!(imports, vec![(1, "../server/checkout".to_string())]);
    }

    #[test]
    fn extracts_from_multiline_export_from() {
        let content = r#"export {
  checkout
} from '../server/checkout';
"#;

        let imports = extract_imports_from_content(content);
        assert_eq!(imports, vec![(1, "../server/checkout".to_string())]);
    }

    #[test]
    fn extracts_from_dynamic_import() {
        let content = "const mod = await import('../server/checkout');";
        assert_eq!(import_paths(content), vec!["../server/checkout"]);
    }

    #[test]
    fn extracts_from_multiline_dynamic_import() {
        let content = r#"const mod = await import(
  '../server/checkout'
);
"#;

        let imports = extract_imports_from_content(content);
        assert_eq!(imports, vec![(1, "../server/checkout".to_string())]);
    }
}
