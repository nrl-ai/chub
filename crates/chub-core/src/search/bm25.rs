use std::collections::HashMap;

use super::tokenizer::{tokenize, tokenize_identifier};
use crate::types::{AvgFieldLengths, Bm25Params, Entry, SearchDocument, SearchIndex, SearchTokens};

/// Default BM25 parameters.
pub(crate) const DEFAULT_K1: f64 = 1.5;
pub(crate) const DEFAULT_B: f64 = 0.75;

/// Field weights for multi-field scoring (id > name > tags > description).
pub(crate) const FIELD_WEIGHT_ID: f64 = 4.0;
pub(crate) const FIELD_WEIGHT_NAME: f64 = 3.0;
pub(crate) const FIELD_WEIGHT_TAGS: f64 = 2.0;
pub(crate) const FIELD_WEIGHT_DESCRIPTION: f64 = 1.0;

/// Build an inverted index from documents: term → list of doc indexes.
fn build_inverted_index(documents: &[SearchDocument]) -> HashMap<String, Vec<usize>> {
    let mut inverted: HashMap<String, Vec<usize>> = HashMap::new();

    for (doc_idx, doc) in documents.iter().enumerate() {
        let mut all_terms: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for t in &doc.tokens.id {
            all_terms.insert(t);
        }
        for t in &doc.tokens.name {
            all_terms.insert(t);
        }
        for t in &doc.tokens.description {
            all_terms.insert(t);
        }
        for t in &doc.tokens.tags {
            all_terms.insert(t);
        }
        for term in all_terms {
            inverted.entry(term.to_string()).or_default().push(doc_idx);
        }
    }

    inverted
}

/// Build a search index from pre-tokenized documents.
/// Used for runtime index merging/rebuilding.
pub fn build_index_from_documents(
    documents: Vec<SearchDocument>,
    params: Bm25Params,
) -> SearchIndex {
    let mut df_map: HashMap<String, usize> = HashMap::new();
    let mut id_lengths = Vec::with_capacity(documents.len());
    let mut name_lengths = Vec::with_capacity(documents.len());
    let mut desc_lengths = Vec::with_capacity(documents.len());
    let mut tags_lengths = Vec::with_capacity(documents.len());

    for doc in &documents {
        id_lengths.push(doc.tokens.id.len());
        name_lengths.push(doc.tokens.name.len());
        desc_lengths.push(doc.tokens.description.len());
        tags_lengths.push(doc.tokens.tags.len());

        let mut all_terms: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for t in &doc.tokens.id {
            all_terms.insert(t);
        }
        for t in &doc.tokens.name {
            all_terms.insert(t);
        }
        for t in &doc.tokens.description {
            all_terms.insert(t);
        }
        for t in &doc.tokens.tags {
            all_terms.insert(t);
        }
        for term in &all_terms {
            *df_map.entry(term.to_string()).or_insert(0) += 1;
        }
    }

    let n = documents.len();
    let idf: HashMap<String, f64> = df_map
        .iter()
        .map(|(term, &df)| {
            let val = ((n as f64 - df as f64 + 0.5) / (df as f64 + 0.5) + 1.0).ln();
            (term.clone(), val)
        })
        .collect();

    let avg = |lens: &[usize]| -> f64 {
        if lens.is_empty() {
            0.0
        } else {
            lens.iter().sum::<usize>() as f64 / lens.len() as f64
        }
    };

    let inverted_index = build_inverted_index(&documents);

    SearchIndex {
        version: "1.0.0".to_string(),
        algorithm: "bm25".to_string(),
        params,
        total_docs: n,
        avg_field_lengths: AvgFieldLengths {
            id: avg(&id_lengths),
            name: avg(&name_lengths),
            description: avg(&desc_lengths),
            tags: avg(&tags_lengths),
        },
        idf,
        documents,
        inverted_index: Some(inverted_index),
    }
}

/// Build a BM25 search index from registry entries.
/// Called during `chub build`. Produces a JSON-compatible SearchIndex.
pub fn build_index(entries: &[Entry<'_>]) -> SearchIndex {
    let mut documents = Vec::with_capacity(entries.len());

    for entry in entries {
        let id_tokens = tokenize_identifier(entry.id());
        let name_tokens = tokenize(entry.name());
        let desc_tokens = tokenize(entry.description());
        let tag_tokens: Vec<String> = entry.tags().iter().flat_map(|t| tokenize(t)).collect();

        documents.push(SearchDocument {
            id: entry.id().to_string(),
            tokens: SearchTokens {
                id: id_tokens,
                name: name_tokens,
                description: desc_tokens,
                tags: tag_tokens,
            },
        });
    }

    let params = Bm25Params {
        k1: DEFAULT_K1,
        b: DEFAULT_B,
    };
    build_index_from_documents(documents, params)
}

/// Search result from BM25 scoring.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
}

/// Score a single field using BM25.
fn score_field(
    query_terms: &[String],
    field_tokens: &[String],
    idf: &HashMap<String, f64>,
    avg_field_len: f64,
    k1: f64,
    b: f64,
) -> f64 {
    if field_tokens.is_empty() {
        return 0.0;
    }

    // Build term frequency map
    let mut tf: HashMap<&str, f64> = HashMap::new();
    for t in field_tokens {
        *tf.entry(t.as_str()).or_insert(0.0) += 1.0;
    }

    let dl = field_tokens.len() as f64;
    let avg_fl = if avg_field_len == 0.0 {
        1.0
    } else {
        avg_field_len
    };
    let mut score = 0.0;

    for term in query_terms {
        let term_freq = tf.get(term.as_str()).copied().unwrap_or(0.0);
        if term_freq == 0.0 {
            continue;
        }

        let term_idf = idf.get(term.as_str()).copied().unwrap_or(0.0);
        let numerator = term_freq * (k1 + 1.0);
        let denominator = term_freq + k1 * (1.0 - b + b * (dl / avg_fl));
        score += term_idf * (numerator / denominator);
    }

    score
}

/// Get candidate document indexes using the inverted index.
/// Falls back to all documents if no inverted index is available.
fn get_candidate_doc_indexes(query_terms: &[String], index: &SearchIndex) -> Vec<usize> {
    match &index.inverted_index {
        Some(inv) => {
            let mut candidates: std::collections::HashSet<usize> = std::collections::HashSet::new();
            for term in query_terms {
                if let Some(postings) = inv.get(term.as_str()) {
                    for &idx in postings {
                        candidates.insert(idx);
                    }
                }
            }
            candidates.into_iter().collect()
        }
        None => (0..index.documents.len()).collect(),
    }
}

/// Search the BM25 index with a query string.
/// Returns results sorted by score descending.
pub fn search(query: &str, index: &SearchIndex, limit: Option<usize>) -> Vec<SearchResult> {
    let query_terms = tokenize(query);
    if query_terms.is_empty() {
        return Vec::new();
    }

    let k1 = index.params.k1;
    let b = index.params.b;
    let mut results = Vec::new();

    let candidates = get_candidate_doc_indexes(&query_terms, index);

    for doc_idx in candidates {
        let doc = &index.documents[doc_idx];

        let id_score = score_field(
            &query_terms,
            &doc.tokens.id,
            &index.idf,
            index.avg_field_lengths.id,
            k1,
            b,
        );
        let name_score = score_field(
            &query_terms,
            &doc.tokens.name,
            &index.idf,
            index.avg_field_lengths.name,
            k1,
            b,
        );
        let desc_score = score_field(
            &query_terms,
            &doc.tokens.description,
            &index.idf,
            index.avg_field_lengths.description,
            k1,
            b,
        );
        let tags_score = score_field(
            &query_terms,
            &doc.tokens.tags,
            &index.idf,
            index.avg_field_lengths.tags,
            k1,
            b,
        );

        let total = id_score * FIELD_WEIGHT_ID
            + name_score * FIELD_WEIGHT_NAME
            + tags_score * FIELD_WEIGHT_TAGS
            + desc_score * FIELD_WEIGHT_DESCRIPTION;

        if total > 0.0 {
            results.push(SearchResult {
                id: doc.id.clone(),
                score: total,
            });
        }
    }

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if let Some(limit) = limit {
        results.truncate(limit);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DocEntry, LanguageEntry, SkillEntry, VersionEntry};

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

    #[test]
    fn test_build_index_structure() {
        let doc = make_doc(
            "test/lib",
            "Test Library",
            "A test library for testing",
            &["test"],
        );
        let entries = vec![Entry::Doc(&doc)];
        let index = build_index(&entries);

        assert_eq!(index.version, "1.0.0");
        assert_eq!(index.algorithm, "bm25");
        assert_eq!(index.params.k1, 1.5);
        assert_eq!(index.params.b, 0.75);
        assert_eq!(index.total_docs, 1);
        assert_eq!(index.documents.len(), 1);
        assert_eq!(index.documents[0].id, "test/lib");
        assert!(index.inverted_index.is_some());
    }

    #[test]
    fn test_build_index_has_id_tokens() {
        let doc = make_doc("node-fetch/http", "Node Fetch", "HTTP client", &[]);
        let entries = vec![Entry::Doc(&doc)];
        let index = build_index(&entries);

        let id_tokens = &index.documents[0].tokens.id;
        assert!(
            id_tokens.contains(&"nodefetch".to_string()) || id_tokens.contains(&"node".to_string())
        );
    }

    #[test]
    fn test_search_basic() {
        let doc1 = make_doc(
            "stripe/stripe",
            "Stripe",
            "Payment processing API",
            &["payments"],
        );
        let doc2 = make_doc(
            "react/react",
            "React",
            "UI component library",
            &["frontend", "ui"],
        );
        let entries = vec![Entry::Doc(&doc1), Entry::Doc(&doc2)];
        let index = build_index(&entries);

        let results = search("stripe payment", &index, None);
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "stripe/stripe");
    }

    #[test]
    fn test_search_with_limit() {
        let doc1 = make_doc("a/a", "Alpha API", "Alpha description", &["api"]);
        let doc2 = make_doc("b/b", "Beta API", "Beta description", &["api"]);
        let doc3 = make_doc("c/c", "Gamma API", "Gamma description", &["api"]);
        let entries = vec![Entry::Doc(&doc1), Entry::Doc(&doc2), Entry::Doc(&doc3)];
        let index = build_index(&entries);

        let results = search("api", &index, Some(2));
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_empty_query() {
        let doc = make_doc("test/lib", "Test", "Desc", &[]);
        let entries = vec![Entry::Doc(&doc)];
        let index = build_index(&entries);

        let results = search("", &index, None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_no_match() {
        let doc = make_doc("test/lib", "Test Library", "A library", &["test"]);
        let entries = vec![Entry::Doc(&doc)];
        let index = build_index(&entries);

        let results = search("zzzznonexistent", &index, None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_name_field_weighted_higher() {
        let doc1 = make_doc("a/stripe", "Stripe", "Some generic description", &[]);
        let doc2 = make_doc("b/other", "Other Library", "Stripe integration helper", &[]);
        let entries = vec![Entry::Doc(&doc1), Entry::Doc(&doc2)];
        let index = build_index(&entries);

        let results = search("stripe", &index, None);
        assert!(results.len() >= 2);
        // "stripe" in name should rank higher than "stripe" in description
        assert_eq!(results[0].id, "a/stripe");
    }

    #[test]
    fn test_skills_indexed() {
        let skill = make_skill(
            "test/deploy",
            "deploy",
            "Deployment automation",
            &["ci", "deploy"],
        );
        let entries = vec![Entry::Skill(&skill)];
        let index = build_index(&entries);

        let results = search("deploy", &index, None);
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "test/deploy");
    }
}
