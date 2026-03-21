//! Inverted index for fast BM25 search.
//! This is a Rust-side optimization — instead of scanning all documents,
//! we only score documents that contain at least one query term.

use std::collections::HashMap;

use super::bm25::SearchResult;
use super::tokenizer::tokenize;
use crate::types::SearchIndex;

/// A posting in the inverted index: document index + per-field term frequencies.
#[derive(Debug, Clone)]
struct Posting {
    doc_idx: usize,
    tf_id: f64,
    tf_name: f64,
    tf_description: f64,
    tf_tags: f64,
}

/// An inverted index built from a SearchIndex for fast lookup.
pub struct InvertedIndex<'a> {
    index: &'a SearchIndex,
    postings: HashMap<String, Vec<Posting>>,
}

impl<'a> InvertedIndex<'a> {
    /// Build an inverted index from a SearchIndex.
    pub fn new(index: &'a SearchIndex) -> Self {
        let mut postings: HashMap<String, Vec<Posting>> = HashMap::new();

        for (doc_idx, doc) in index.documents.iter().enumerate() {
            // Compute TF maps for each field
            let tf_id = term_freq_map(&doc.tokens.id);
            let tf_name = term_freq_map(&doc.tokens.name);
            let tf_desc = term_freq_map(&doc.tokens.description);
            let tf_tags = term_freq_map(&doc.tokens.tags);

            // Collect all unique terms
            let mut all_terms: std::collections::HashSet<&str> = std::collections::HashSet::new();
            for t in tf_id.keys() {
                all_terms.insert(t.as_str());
            }
            for t in tf_name.keys() {
                all_terms.insert(t.as_str());
            }
            for t in tf_desc.keys() {
                all_terms.insert(t.as_str());
            }
            for t in tf_tags.keys() {
                all_terms.insert(t.as_str());
            }

            for term in all_terms {
                postings.entry(term.to_string()).or_default().push(Posting {
                    doc_idx,
                    tf_id: tf_id.get(term).copied().unwrap_or(0.0),
                    tf_name: tf_name.get(term).copied().unwrap_or(0.0),
                    tf_description: tf_desc.get(term).copied().unwrap_or(0.0),
                    tf_tags: tf_tags.get(term).copied().unwrap_or(0.0),
                });
            }
        }

        InvertedIndex { index, postings }
    }

    /// Search using the inverted index — only scores documents containing query terms.
    pub fn search(&self, query: &str, limit: Option<usize>) -> Vec<SearchResult> {
        let query_terms = tokenize(query);
        if query_terms.is_empty() {
            return Vec::new();
        }

        let k1 = self.index.params.k1;
        let b = self.index.params.b;

        // Collect candidate documents (those containing at least one query term)
        let mut scores: HashMap<usize, f64> = HashMap::new();

        for term in &query_terms {
            let term_idf = self.index.idf.get(term.as_str()).copied().unwrap_or(0.0);
            if term_idf == 0.0 {
                continue;
            }

            if let Some(postings) = self.postings.get(term.as_str()) {
                for posting in postings {
                    let doc = &self.index.documents[posting.doc_idx];

                    use super::bm25::{
                        FIELD_WEIGHT_DESCRIPTION, FIELD_WEIGHT_ID, FIELD_WEIGHT_NAME,
                        FIELD_WEIGHT_TAGS,
                    };

                    let id_score = bm25_tf_score(
                        posting.tf_id,
                        doc.tokens.id.len() as f64,
                        self.index.avg_field_lengths.id,
                        k1,
                        b,
                    ) * term_idf
                        * FIELD_WEIGHT_ID;

                    let name_score = bm25_tf_score(
                        posting.tf_name,
                        doc.tokens.name.len() as f64,
                        self.index.avg_field_lengths.name,
                        k1,
                        b,
                    ) * term_idf
                        * FIELD_WEIGHT_NAME;

                    let desc_score = bm25_tf_score(
                        posting.tf_description,
                        doc.tokens.description.len() as f64,
                        self.index.avg_field_lengths.description,
                        k1,
                        b,
                    ) * term_idf
                        * FIELD_WEIGHT_DESCRIPTION;

                    let tags_score = bm25_tf_score(
                        posting.tf_tags,
                        doc.tokens.tags.len() as f64,
                        self.index.avg_field_lengths.tags,
                        k1,
                        b,
                    ) * term_idf
                        * FIELD_WEIGHT_TAGS;

                    *scores.entry(posting.doc_idx).or_insert(0.0) +=
                        id_score + name_score + desc_score + tags_score;
                }
            }
        }

        let mut results: Vec<SearchResult> = scores
            .into_iter()
            .filter(|(_, score)| *score > 0.0)
            .map(|(idx, score)| SearchResult {
                id: self.index.documents[idx].id.clone(),
                score,
            })
            .collect();

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
}

/// Compute BM25 score component for a single term in a single field.
fn bm25_tf_score(tf: f64, dl: f64, avg_fl: f64, k1: f64, b: f64) -> f64 {
    if tf == 0.0 {
        return 0.0;
    }
    let avg_fl = if avg_fl == 0.0 { 1.0 } else { avg_fl };
    let numerator = tf * (k1 + 1.0);
    let denominator = tf + k1 * (1.0 - b + b * (dl / avg_fl));
    numerator / denominator
}

/// Build a term frequency map from a list of tokens.
fn term_freq_map(tokens: &[String]) -> HashMap<String, f64> {
    let mut tf = HashMap::new();
    for t in tokens {
        *tf.entry(t.clone()).or_insert(0.0) += 1.0;
    }
    tf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::bm25;
    use crate::types::{DocEntry, Entry, LanguageEntry, VersionEntry};

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
                    content_hash: None,
                }],
                recommended_version: "1.0.0".to_string(),
            }],
        }
    }

    #[test]
    fn test_inverted_index_matches_linear_scan() {
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
        let doc3 = make_doc(
            "next/next",
            "Next.js",
            "React framework for production",
            &["react", "ssr"],
        );

        let entries: Vec<Entry> = vec![Entry::Doc(&doc1), Entry::Doc(&doc2), Entry::Doc(&doc3)];
        let search_index = bm25::build_index(&entries);

        let linear_results = bm25::search("react", &search_index, None);
        let inv_index = InvertedIndex::new(&search_index);
        let inv_results = inv_index.search("react", None);

        // Same number of results
        assert_eq!(linear_results.len(), inv_results.len());

        // Same IDs in same order
        for (linear, inv) in linear_results.iter().zip(inv_results.iter()) {
            assert_eq!(linear.id, inv.id);
            assert!((linear.score - inv.score).abs() < 1e-10);
        }
    }
}
