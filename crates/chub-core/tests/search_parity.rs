//! Tests for BM25 search parity with JS implementation.
//! Mirrors bm25.test.js search quality tests.

use chub_core::search::bm25::{build_index, search};
use chub_core::search::index::InvertedIndex;
use chub_core::search::tokenizer::tokenize;
use chub_core::types::{DocEntry, Entry, LanguageEntry, SkillEntry, VersionEntry};

fn make_doc(id: &str, name: &str, desc: &str, tags: &[&str]) -> DocEntry {
    DocEntry {
        id: id.to_string(),
        name: name.to_string(),
        description: desc.to_string(),
        source: "community".to_string(),
        tags: tags.iter().map(|t| t.to_string()).collect(),
        languages: vec![LanguageEntry {
            language: "javascript".to_string(),
            versions: vec![VersionEntry {
                version: "1.0.0".to_string(),
                path: id.to_string(),
                files: vec!["DOC.md".to_string()],
                size: 100,
                last_updated: "2025-01-01".to_string(),
            }],
            recommended_version: "1.0.0".to_string(),
        }],
    }
}

fn make_skill(id: &str, name: &str, desc: &str, tags: &[&str]) -> SkillEntry {
    SkillEntry {
        id: id.to_string(),
        name: name.to_string(),
        description: desc.to_string(),
        source: "community".to_string(),
        tags: tags.iter().map(|t| t.to_string()).collect(),
        path: id.to_string(),
        files: vec!["SKILL.md".to_string()],
        size: 100,
        last_updated: "2025-01-01".to_string(),
    }
}

/// The same 10-entry corpus used in the JS bm25.test.js quality report.
fn quality_corpus() -> (Vec<DocEntry>, Vec<SkillEntry>) {
    let docs = vec![
        make_doc(
            "stripe/api",
            "Stripe API",
            "Payment processing and billing API",
            &["payments", "billing", "fintech"],
        ),
        make_doc(
            "square/payments",
            "Square Payments",
            "In-person and online payment processing",
            &["payments", "pos", "commerce"],
        ),
        make_doc(
            "redis/cache",
            "Redis",
            "In-memory data store and cache",
            &["database", "cache", "nosql"],
        ),
        make_doc(
            "openai/chat",
            "OpenAI Chat",
            "Large language models for chat and completion",
            &["ai", "llm", "gpt"],
        ),
        make_doc(
            "auth0/auth",
            "Auth0 Authentication",
            "Authentication and authorization platform",
            &["auth", "security", "identity"],
        ),
        make_doc(
            "twilio/sms",
            "Twilio SMS",
            "Send and receive SMS messages programmatically",
            &["sms", "messaging", "communication"],
        ),
        make_doc(
            "sentry/errors",
            "Sentry Error Tracking",
            "Error monitoring and performance tracking",
            &["errors", "monitoring", "debugging"],
        ),
        make_doc(
            "cypress/testing",
            "Cypress",
            "End-to-end browser testing framework",
            &["testing", "browser", "e2e"],
        ),
    ];
    let skills = vec![
        make_skill(
            "deploy/ci",
            "CI/CD Deploy",
            "Continuous integration and deployment automation",
            &["deploy", "ci", "automation"],
        ),
        make_skill(
            "lint/code",
            "Code Linter",
            "Automated code quality and style checking",
            &["lint", "quality", "style"],
        ),
    ];
    (docs, skills)
}

// ===== TOKENIZER PARITY TESTS (from bm25.test.js) =====

#[test]
fn tokenize_lowercases_and_splits() {
    assert_eq!(tokenize("Hello World"), vec!["hello", "world"]);
}

#[test]
fn tokenize_removes_stop_words() {
    let tokens = tokenize("the quick and brown fox");
    assert_eq!(tokens, vec!["quick", "brown", "fox"]);
}

#[test]
fn tokenize_removes_punctuation() {
    let tokens = tokenize("hello, world! foo-bar");
    assert_eq!(tokens, vec!["hello", "world", "foo", "bar"]);
}

#[test]
fn tokenize_removes_single_chars() {
    let tokens = tokenize("a b cd ef");
    assert_eq!(tokens, vec!["cd", "ef"]);
}

#[test]
fn tokenize_empty_input() {
    assert!(tokenize("").is_empty());
}

// ===== BUILD INDEX TESTS (from bm25.test.js) =====

#[test]
fn build_index_correct_structure() {
    let (docs, skills) = quality_corpus();
    let entries: Vec<Entry> = docs
        .iter()
        .map(|d| Entry::Doc(d))
        .chain(skills.iter().map(|s| Entry::Skill(s)))
        .collect();
    let index = build_index(&entries);

    assert_eq!(index.version, "1.0.0");
    assert_eq!(index.algorithm, "bm25");
    assert_eq!(index.total_docs, 10);
    assert_eq!(index.params.k1, 1.5);
    assert_eq!(index.params.b, 0.75);
}

#[test]
fn build_index_tokenizes_all_fields() {
    let (docs, skills) = quality_corpus();
    let entries: Vec<Entry> = docs
        .iter()
        .map(|d| Entry::Doc(d))
        .chain(skills.iter().map(|s| Entry::Skill(s)))
        .collect();
    let index = build_index(&entries);

    let stripe = index
        .documents
        .iter()
        .find(|d| d.id == "stripe/api")
        .unwrap();
    assert!(!stripe.tokens.id.is_empty());
    assert!(!stripe.tokens.name.is_empty());
    assert!(!stripe.tokens.description.is_empty());
    assert!(!stripe.tokens.tags.is_empty());
}

#[test]
fn build_index_computes_idf() {
    let (docs, skills) = quality_corpus();
    let entries: Vec<Entry> = docs
        .iter()
        .map(|d| Entry::Doc(d))
        .chain(skills.iter().map(|s| Entry::Skill(s)))
        .collect();
    let index = build_index(&entries);

    assert!(!index.idf.is_empty());
    for (_term, idf) in &index.idf {
        assert!(*idf > 0.0, "All IDF values should be positive");
    }
}

#[test]
fn build_index_computes_avg_field_lengths() {
    let (docs, skills) = quality_corpus();
    let entries: Vec<Entry> = docs
        .iter()
        .map(|d| Entry::Doc(d))
        .chain(skills.iter().map(|s| Entry::Skill(s)))
        .collect();
    let index = build_index(&entries);

    assert!(index.avg_field_lengths.id > 0.0);
    assert!(index.avg_field_lengths.name > 0.0);
    assert!(index.avg_field_lengths.description > 0.0);
    assert!(index.avg_field_lengths.tags > 0.0);
}

#[test]
fn build_index_handles_empty() {
    let entries: Vec<Entry> = vec![];
    let index = build_index(&entries);
    assert_eq!(index.total_docs, 0);
    assert!(index.documents.is_empty());
}

// ===== SEARCH TESTS (from bm25.test.js) =====

#[test]
fn search_finds_by_keyword() {
    let (docs, skills) = quality_corpus();
    let entries: Vec<Entry> = docs
        .iter()
        .map(|d| Entry::Doc(d))
        .chain(skills.iter().map(|s| Entry::Skill(s)))
        .collect();
    let index = build_index(&entries);

    let results = search("payment", &index, None);
    let ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
    assert!(ids.contains(&"stripe/api"), "Should find stripe/api");
    assert!(
        ids.contains(&"square/payments"),
        "Should find square/payments"
    );
}

#[test]
fn search_ranks_exact_name_higher() {
    let (docs, skills) = quality_corpus();
    let entries: Vec<Entry> = docs
        .iter()
        .map(|d| Entry::Doc(d))
        .chain(skills.iter().map(|s| Entry::Skill(s)))
        .collect();
    let index = build_index(&entries);

    let results = search("payments", &index, None);
    assert!(!results.is_empty());
    // "payments" appears in name of square/payments, so it should rank high
    assert_eq!(results[0].id, "square/payments");
}

#[test]
fn search_finds_by_tag() {
    let (docs, skills) = quality_corpus();
    let entries: Vec<Entry> = docs
        .iter()
        .map(|d| Entry::Doc(d))
        .chain(skills.iter().map(|s| Entry::Skill(s)))
        .collect();
    let index = build_index(&entries);

    let results = search("database", &index, None);
    let ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
    assert!(ids.contains(&"redis/cache"));
}

#[test]
fn search_finds_by_description() {
    let (docs, skills) = quality_corpus();
    let entries: Vec<Entry> = docs
        .iter()
        .map(|d| Entry::Doc(d))
        .chain(skills.iter().map(|s| Entry::Skill(s)))
        .collect();
    let index = build_index(&entries);

    let results = search("language models", &index, None);
    let ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
    assert!(ids.contains(&"openai/chat"));
}

#[test]
fn search_empty_for_no_match() {
    let (docs, skills) = quality_corpus();
    let entries: Vec<Entry> = docs
        .iter()
        .map(|d| Entry::Doc(d))
        .chain(skills.iter().map(|s| Entry::Skill(s)))
        .collect();
    let index = build_index(&entries);

    let results = search("xyznonexistent123", &index, None);
    assert!(results.is_empty());
}

#[test]
fn search_multi_word_queries() {
    let (docs, skills) = quality_corpus();
    let entries: Vec<Entry> = docs
        .iter()
        .map(|d| Entry::Doc(d))
        .chain(skills.iter().map(|s| Entry::Skill(s)))
        .collect();
    let index = build_index(&entries);

    let results = search("payment processing", &index, None);
    let ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
    assert!(ids.contains(&"stripe/api"));
    assert!(ids.contains(&"square/payments"));
}

#[test]
fn search_respects_limit() {
    let (docs, skills) = quality_corpus();
    let entries: Vec<Entry> = docs
        .iter()
        .map(|d| Entry::Doc(d))
        .chain(skills.iter().map(|s| Entry::Skill(s)))
        .collect();
    let index = build_index(&entries);

    let results = search("payment", &index, Some(1));
    assert_eq!(results.len(), 1);
}

#[test]
fn search_empty_for_stop_words_only() {
    let (docs, skills) = quality_corpus();
    let entries: Vec<Entry> = docs
        .iter()
        .map(|d| Entry::Doc(d))
        .chain(skills.iter().map(|s| Entry::Skill(s)))
        .collect();
    let index = build_index(&entries);

    let results = search("the and is", &index, None);
    assert!(results.is_empty());
}

// ===== INVERTED INDEX PARITY =====

#[test]
fn inverted_index_matches_linear_for_all_queries() {
    let (docs, skills) = quality_corpus();
    let entries: Vec<Entry> = docs
        .iter()
        .map(|d| Entry::Doc(d))
        .chain(skills.iter().map(|s| Entry::Skill(s)))
        .collect();
    let index = build_index(&entries);
    let inv = InvertedIndex::new(&index);

    let queries = [
        "payment processing",
        "database",
        "authentication login",
        "browser testing",
        "error monitoring",
        "send SMS",
        "AI language model",
        "deploy",
        "stripe",
        "redis cache",
    ];

    for query in &queries {
        let linear = search(query, &index, None);
        let inverted = inv.search(query, None);

        assert_eq!(
            linear.len(),
            inverted.len(),
            "Result count mismatch for query '{}'",
            query
        );

        // Compare result sets (same IDs with close scores).
        // Due to floating-point accumulation order differences between
        // term-at-a-time (inverted) and document-at-a-time (linear),
        // results with very close scores may swap order.
        let mut linear_ids: Vec<&str> = linear.iter().map(|r| r.id.as_str()).collect();
        let mut inverted_ids: Vec<&str> = inverted.iter().map(|r| r.id.as_str()).collect();
        linear_ids.sort();
        inverted_ids.sort();
        assert_eq!(
            linear_ids, inverted_ids,
            "ID set mismatch for query '{}'",
            query
        );

        // Verify scores are close (find matching IDs)
        for l in &linear {
            let matching = inverted.iter().find(|i| i.id == l.id).unwrap();
            assert!(
                (l.score - matching.score).abs() < 1e-6,
                "Score mismatch for query '{}', id '{}': linear={} inverted={}",
                query,
                l.id,
                l.score,
                matching.score
            );
        }
    }
}

// ===== SEARCH INDEX SERIALIZATION ROUNDTRIP =====

#[test]
fn search_index_json_roundtrip() {
    let (docs, skills) = quality_corpus();
    let entries: Vec<Entry> = docs
        .iter()
        .map(|d| Entry::Doc(d))
        .chain(skills.iter().map(|s| Entry::Skill(s)))
        .collect();
    let index = build_index(&entries);

    // Serialize to JSON
    let json = serde_json::to_string(&index).unwrap();
    // Deserialize back
    let deserialized: chub_core::types::SearchIndex = serde_json::from_str(&json).unwrap();

    assert_eq!(index.version, deserialized.version);
    assert_eq!(index.total_docs, deserialized.total_docs);
    assert_eq!(index.documents.len(), deserialized.documents.len());

    // Search on deserialized index should produce same results
    let original_results = search("payment", &index, None);
    let deser_results = search("payment", &deserialized, None);
    assert_eq!(original_results.len(), deser_results.len());
    // Compare sets of IDs (order may vary for equal-score entries due to HashMap iteration)
    let mut orig_ids: Vec<&str> = original_results.iter().map(|r| r.id.as_str()).collect();
    let mut deser_ids: Vec<&str> = deser_results.iter().map(|r| r.id.as_str()).collect();
    orig_ids.sort();
    deser_ids.sort();
    assert_eq!(orig_ids, deser_ids);
}
