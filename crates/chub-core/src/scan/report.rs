//! Report generation — JSON, SARIF, CSV output formats.

use std::io::Write;

use super::finding::Finding;

/// Output format for scan reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Json,
    Sarif,
    Csv,
}

impl ReportFormat {
    /// Parse from string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "sarif" => Some(Self::Sarif),
            "csv" => Some(Self::Csv),
            _ => None,
        }
    }

    /// Infer from file extension.
    pub fn from_extension(path: &str) -> Option<Self> {
        if path.ends_with(".json") {
            Some(Self::Json)
        } else if path.ends_with(".sarif") {
            Some(Self::Sarif)
        } else if path.ends_with(".csv") {
            Some(Self::Csv)
        } else {
            None
        }
    }
}

/// Write findings as JSON.
pub fn write_json<W: Write>(findings: &[Finding], writer: &mut W) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(findings).map_err(std::io::Error::other)?;
    writer.write_all(json.as_bytes())?;
    writer.write_all(b"\n")
}

/// Write findings as CSV.
pub fn write_csv<W: Write>(findings: &[Finding], writer: &mut W) -> std::io::Result<()> {
    // Header
    writeln!(
        writer,
        "RuleID,Commit,File,Secret,Match,StartLine,EndLine,StartColumn,EndColumn,Author,Date,Fingerprint"
    )?;
    for f in findings {
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            csv_escape(&f.rule_id),
            csv_escape(&f.commit),
            csv_escape(&f.file),
            csv_escape(&f.secret),
            csv_escape(&f.match_text),
            f.start_line,
            f.end_line,
            f.start_column,
            f.end_column,
            csv_escape(&f.author),
            csv_escape(&f.date),
            csv_escape(&f.fingerprint),
        )?;
    }
    Ok(())
}

/// Write findings as SARIF (Static Analysis Results Interchange Format).
pub fn write_sarif<W: Write>(findings: &[Finding], writer: &mut W) -> std::io::Result<()> {
    let sarif = build_sarif(findings);
    let json = serde_json::to_string_pretty(&sarif).map_err(std::io::Error::other)?;
    writer.write_all(json.as_bytes())?;
    writer.write_all(b"\n")
}

/// Write findings in the specified format.
pub fn write_report<W: Write>(
    findings: &[Finding],
    writer: &mut W,
    format: ReportFormat,
) -> std::io::Result<()> {
    match format {
        ReportFormat::Json => write_json(findings, writer),
        ReportFormat::Sarif => write_sarif(findings, writer),
        ReportFormat::Csv => write_csv(findings, writer),
    }
}

/// Escape a field for CSV output.
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Build a SARIF 2.1.0 document from findings.
fn build_sarif(findings: &[Finding]) -> serde_json::Value {
    use serde_json::json;

    // Collect unique rules
    let mut rule_ids: Vec<String> = Vec::new();
    let mut rule_index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for f in findings {
        if !rule_index.contains_key(&f.rule_id) {
            let idx = rule_ids.len();
            rule_ids.push(f.rule_id.clone());
            rule_index.insert(f.rule_id.clone(), idx);
        }
    }

    let rules: Vec<serde_json::Value> = rule_ids
        .iter()
        .map(|id| {
            let desc = findings
                .iter()
                .find(|f| &f.rule_id == id)
                .map(|f| f.description.as_str())
                .unwrap_or("Secret detected");
            json!({
                "id": id,
                "shortDescription": { "text": desc },
            })
        })
        .collect();

    let results: Vec<serde_json::Value> = findings
        .iter()
        .map(|f| {
            let idx = rule_index.get(&f.rule_id).copied().unwrap_or(0);
            json!({
                "ruleId": f.rule_id,
                "ruleIndex": idx,
                "message": { "text": f.description },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": f.file },
                        "region": {
                            "startLine": f.start_line,
                            "startColumn": f.start_column,
                            "endLine": f.end_line,
                            "endColumn": f.end_column,
                        }
                    }
                }],
                "fingerprints": {
                    "chubFingerprint/v1": f.fingerprint,
                },
                "partialFingerprints": {
                    "commitSha": f.commit,
                }
            })
        })
        .collect();

    json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "chub",
                    "informationUri": "https://github.com/AiChub/chub",
                    "rules": rules,
                }
            },
            "results": results,
        }]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_finding() -> Finding {
        Finding {
            rule_id: "aws-access-token".to_string(),
            description: "AWS Access Token detected".to_string(),
            start_line: 3,
            end_line: 3,
            start_column: 5,
            end_column: 25,
            match_text: "AKIAIOSFODNN7EXAMPLE".to_string(),
            secret: "AKIAIOSFODNN7EXAMPLE".to_string(),
            file: "test.env".to_string(),
            symlink_file: String::new(),
            commit: "abc123".to_string(),
            entropy: 3.5,
            author: "Test".to_string(),
            email: "test@test.com".to_string(),
            date: "2024-01-01".to_string(),
            message: "add key".to_string(),
            tags: vec![],
            fingerprint: "abc123def456".to_string(),
        }
    }

    #[test]
    fn json_output() {
        let findings = vec![sample_finding()];
        let mut buf = Vec::new();
        write_json(&findings, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("aws-access-token"));
        assert!(output.contains("AKIAIOSFODNN7EXAMPLE"));
        // Verify it's valid JSON
        let parsed: Vec<Finding> = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn csv_output() {
        let findings = vec![sample_finding()];
        let mut buf = Vec::new();
        write_csv(&findings, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.starts_with("RuleID,"));
        assert!(output.contains("aws-access-token"));
        assert!(output.contains("test.env"));
    }

    #[test]
    fn sarif_output() {
        let findings = vec![sample_finding()];
        let mut buf = Vec::new();
        write_sarif(&findings, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("sarif-schema-2.1.0"));
        assert!(output.contains("aws-access-token"));
        // Verify valid JSON
        let _: serde_json::Value = serde_json::from_str(&output).unwrap();
    }

    #[test]
    fn empty_findings_json() {
        let mut buf = Vec::new();
        write_json(&[], &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(output.trim(), "[]");
    }

    #[test]
    fn format_from_str() {
        assert_eq!(ReportFormat::parse("json"), Some(ReportFormat::Json));
        assert_eq!(ReportFormat::parse("JSON"), Some(ReportFormat::Json));
        assert_eq!(ReportFormat::parse("sarif"), Some(ReportFormat::Sarif));
        assert_eq!(ReportFormat::parse("csv"), Some(ReportFormat::Csv));
        assert_eq!(ReportFormat::parse("xml"), None);
    }

    #[test]
    fn format_from_extension() {
        assert_eq!(
            ReportFormat::from_extension("report.json"),
            Some(ReportFormat::Json)
        );
        assert_eq!(
            ReportFormat::from_extension("report.sarif"),
            Some(ReportFormat::Sarif)
        );
        assert_eq!(
            ReportFormat::from_extension("report.csv"),
            Some(ReportFormat::Csv)
        );
        assert_eq!(ReportFormat::from_extension("report.txt"), None);
    }

    #[test]
    fn csv_escape_with_comma() {
        assert_eq!(csv_escape("hello,world"), "\"hello,world\"");
    }

    #[test]
    fn csv_escape_plain() {
        assert_eq!(csv_escape("hello"), "hello");
    }
}
