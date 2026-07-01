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
    let content = strip_comments_preserving_strings(content);

    for (idx, line) in content.lines().enumerate() {
        let line_number = idx + 1;
        let trimmed = line.trim();

        // 여러 줄 import/export/require/import()는 한 줄씩 이어 붙인 뒤
        // 실제 import path를 찾을 수 있을 때 한 번만 기록한다.
        if let Some(pending) = pending_statement.as_mut() {
            pending.statement.push(' ');
            pending.statement.push_str(trimmed);

            if let Some(import_path) = pending.extract_import_path() {
                imports.push((pending.start_line, import_path));
                pending_statement = None;
            }
            continue;
        }

        // 정적 import/export는 가장 흔한 형태라 먼저 처리한다.
        // 예: import { x } from './x', export { x } from './x'
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

        // CommonJS require도 boundary 위반을 만들 수 있어서 import처럼 수집한다.
        if contains_code_call(trimmed, "require(") {
            if let Some(import_path) = extract_require_path(trimmed) {
                imports.push((line_number, import_path));
            } else if call_may_continue(trimmed, "require(") {
                pending_statement = Some(PendingStatement::new(
                    line_number,
                    trimmed.to_string(),
                    PendingStatementKind::RequireCall,
                ));
                continue;
            }
        }

        // dynamic import('../x')도 client/server 경계를 우회할 수 있으므로 검사 대상이다.
        if contains_code_call(trimmed, "import(") {
            if let Some(import_path) = extract_dynamic_import_path(trimmed) {
                imports.push((line_number, import_path));
            } else if call_may_continue(trimmed, "import(") {
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
    // export async function 같은 일반 export는 import path가 없으므로 제외한다.
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
    let index = find_code_needle(line, needle)?;
    let rest = &line[index + needle.len()..];
    extract_first_quoted(rest)
}

fn contains_code_call(line: &str, needle: &str) -> bool {
    find_code_needle(line, needle).is_some()
}

fn call_may_continue(line: &str, needle: &str) -> bool {
    let Some(index) = find_code_needle(line, needle) else {
        return false;
    };
    !line[index + needle.len()..].contains(')')
}

fn find_code_needle(line: &str, needle: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut index = 0;
    let mut quote: Option<u8> = None;
    let mut escaped = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == active_quote {
                quote = None;
            }
            index += 1;
            continue;
        }

        if matches!(byte, b'\'' | b'"' | b'`') {
            quote = Some(byte);
            index += 1;
            continue;
        }
        if bytes[index..].starts_with(needle.as_bytes()) {
            return Some(index);
        }
        index += 1;
    }

    None
}

fn strip_comments_preserving_strings(content: &str) -> String {
    let bytes = content.as_bytes();
    let mut output = String::with_capacity(content.len());
    let mut index = 0;
    let mut quote: Option<u8> = None;
    let mut escaped = false;
    let mut block_comment = false;

    while index < bytes.len() {
        let byte = bytes[index];
        let next = bytes.get(index + 1).copied();

        if block_comment {
            if byte == b'*' && next == Some(b'/') {
                output.push_str("  ");
                index += 2;
                block_comment = false;
            } else {
                output.push(if byte == b'\n' { '\n' } else { ' ' });
                index += 1;
            }
            continue;
        }

        if let Some(active_quote) = quote {
            output.push(byte as char);
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == active_quote {
                quote = None;
            }
            index += 1;
            continue;
        }

        if matches!(byte, b'\'' | b'"' | b'`') {
            quote = Some(byte);
            output.push(byte as char);
            index += 1;
            continue;
        }
        if byte == b'/' && next == Some(b'/') {
            output.push_str("  ");
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                output.push(' ');
                index += 1;
            }
            continue;
        }
        if byte == b'/' && next == Some(b'*') {
            output.push_str("  ");
            index += 2;
            block_comment = true;
            continue;
        }

        output.push(byte as char);
        index += 1;
    }

    output
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
    let start = input.find(['\'', '"', '`'])?;
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

    #[test]
    fn ignores_imports_inside_comments() {
        let content = r#"// import '../server/commented';
/*
import { hidden } from '../server/hidden';
const alsoHidden = require('../server/hidden-require');
*/
import { visible } from '../shared/visible';
"#;

        assert_eq!(import_paths(content), vec!["../shared/visible"]);
    }

    #[test]
    fn ignores_import_calls_inside_strings() {
        let content = r#"const first = "require('../server/not-real')";
const second = `import('../server/not-real-either')`;
const actual = require('../shared/actual');
"#;

        assert_eq!(import_paths(content), vec!["../shared/actual"]);
    }

    #[test]
    fn ignores_non_literal_dynamic_import_without_swallowing_next_statement() {
        let content = r#"const selected = import(moduleName);
import { visible } from '../shared/visible';
"#;

        assert_eq!(import_paths(content), vec!["../shared/visible"]);
    }

    #[test]
    fn extracts_static_template_literal_dynamic_import() {
        let content = "const selected = import(`../shared/visible`);";
        assert_eq!(import_paths(content), vec!["../shared/visible"]);
    }
}
