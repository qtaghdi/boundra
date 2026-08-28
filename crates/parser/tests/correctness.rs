use std::fs;
use std::path::Path;

use boundra_parser::{collect_imports_with_report, ScanOptions};

#[test]
fn import_extraction_matches_correctness_corpus() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/correctness");
    let options = ScanOptions {
        include_extensions: vec!["ts".to_string(), "svelte".to_string()],
        ignore: Vec::new(),
    };
    let report =
        collect_imports_with_report(&root, &options).expect("correctness corpus should scan");
    assert_eq!(report.scanned_file_count, 3);

    let mut actual = report
        .imports
        .into_iter()
        .map(|record| (record.source_file, record.line, record.import_path))
        .collect::<Vec<_>>();
    actual.sort();

    let mut expected = fs::read_to_string(root.join("expected.tsv"))
        .expect("expected import oracle should exist")
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut fields = line.splitn(3, '\t');
            let file = fields.next().expect("expected file").to_string();
            let line = fields
                .next()
                .expect("expected line")
                .parse::<usize>()
                .expect("line should be numeric");
            let import_path = fields.next().expect("expected import").to_string();
            (file, line, import_path)
        })
        .collect::<Vec<_>>();
    expected.sort();

    assert_eq!(actual, expected);
}
