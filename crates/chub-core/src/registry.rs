use std::collections::{HashMap, HashSet};

use crate::cache::{load_search_index, load_source_registry};
use crate::config::{load_config, SourceConfig};
use crate::normalize::normalize_language;
use crate::search::bm25::{self, build_index_from_documents};
use crate::search::tokenizer::{compact_identifier, tokenize};
use crate::types::{DocEntry, SearchIndex, SkillEntry};

// ---------------------------------------------------------------------------
// TaggedEntry
// ---------------------------------------------------------------------------

/// A tagged entry combining doc/skill with source metadata.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaggedEntry {
    #[serde(flatten)]
    pub kind: EntryKind,
    #[serde(rename = "_source")]
    pub source_name: String,
    #[serde(rename = "_type")]
    pub entry_type: &'static str,
    #[serde(skip)]
    pub source_obj: SourceConfig,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged)]
pub enum EntryKind {
    Doc(DocEntry),
    Skill(SkillEntry),
}

impl TaggedEntry {
    pub fn id(&self) -> &str {
        match &self.kind {
            EntryKind::Doc(d) => &d.id,
            EntryKind::Skill(s) => &s.id,
        }
    }

    pub fn name(&self) -> &str {
        match &self.kind {
            EntryKind::Doc(d) => &d.name,
            EntryKind::Skill(s) => &s.name,
        }
    }

    pub fn description(&self) -> &str {
        match &self.kind {
            EntryKind::Doc(d) => &d.description,
            EntryKind::Skill(s) => &s.description,
        }
    }

    pub fn tags(&self) -> &[String] {
        match &self.kind {
            EntryKind::Doc(d) => &d.tags,
            EntryKind::Skill(s) => &s.tags,
        }
    }

    pub fn source_quality(&self) -> Option<&str> {
        match &self.kind {
            EntryKind::Doc(d) => Some(&d.source),
            EntryKind::Skill(s) => Some(&s.source),
        }
    }

    pub fn languages(&self) -> Option<&[crate::types::LanguageEntry]> {
        match &self.kind {
            EntryKind::Doc(d) => Some(&d.languages),
            EntryKind::Skill(_) => None,
        }
    }

    pub fn as_doc(&self) -> Option<&DocEntry> {
        match &self.kind {
            EntryKind::Doc(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_skill(&self) -> Option<&SkillEntry> {
        match &self.kind {
            EntryKind::Skill(s) => Some(s),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// MergedRegistry + loading
// ---------------------------------------------------------------------------

/// Merged data from all sources.
#[derive(Debug)]
pub struct MergedRegistry {
    pub docs: Vec<TaggedEntry>,
    pub skills: Vec<TaggedEntry>,
    pub search_index: Option<SearchIndex>,
}

/// Build a `source:id` lookup key for namespaced search.
fn search_lookup_id(source: &str, entry_id: &str) -> String {
    format!("{}:{}", source, entry_id)
}

/// Normalize a query string: trim + collapse whitespace.
fn normalize_query(query: &str) -> String {
    query.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Namespace a search index's document IDs with the source name.
fn namespace_search_index(mut index: SearchIndex, source_name: &str) -> SearchIndex {
    for doc in &mut index.documents {
        doc.id = search_lookup_id(source_name, &doc.id);
    }
    // Rebuild inverted index after namespacing
    index.inverted_index = None;
    index
}

/// Load and merge entries from all configured sources.
pub fn load_merged() -> MergedRegistry {
    let config = load_config();
    let mut all_docs = Vec::new();
    let mut all_skills = Vec::new();
    let mut search_indexes = Vec::new();

    for source in &config.sources {
        let registry = match load_source_registry(source) {
            Some(r) => r,
            None => continue,
        };

        if let Some(idx) = load_search_index(source) {
            search_indexes.push(namespace_search_index(idx, &source.name));
        }

        for doc in registry.docs {
            all_docs.push(TaggedEntry {
                kind: EntryKind::Doc(doc),
                source_name: source.name.clone(),
                entry_type: "doc",
                source_obj: source.clone(),
            });
        }

        for skill in registry.skills {
            all_skills.push(TaggedEntry {
                kind: EntryKind::Skill(skill),
                source_name: source.name.clone(),
                entry_type: "skill",
                source_obj: source.clone(),
            });
        }
    }

    // Merge search indexes
    let search_index = merge_search_indexes(search_indexes);

    MergedRegistry {
        docs: all_docs,
        skills: all_skills,
        search_index,
    }
}

fn merge_search_indexes(indexes: Vec<SearchIndex>) -> Option<SearchIndex> {
    if indexes.is_empty() {
        return None;
    }
    if indexes.len() == 1 {
        let single = indexes.into_iter().next().unwrap();
        // Rebuild with inverted index if missing
        if single.inverted_index.is_some() {
            return Some(single);
        }
        return Some(build_index_from_documents(single.documents, single.params));
    }

    let params = indexes[0].params.clone();
    let all_documents: Vec<_> = indexes.into_iter().flat_map(|idx| idx.documents).collect();
    Some(build_index_from_documents(all_documents, params))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn get_all_entries(merged: &MergedRegistry) -> Vec<&TaggedEntry> {
    merged.docs.iter().chain(merged.skills.iter()).collect()
}

fn apply_source_filter(entries: Vec<&TaggedEntry>) -> Vec<&TaggedEntry> {
    let config = load_config();
    let allowed: Vec<String> = config
        .source
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .collect();
    entries
        .into_iter()
        .filter(|e| {
            e.source_quality()
                .map(|s| allowed.contains(&s.to_lowercase()))
                .unwrap_or(true)
        })
        .collect()
}

fn apply_filters<'a>(
    entries: Vec<&'a TaggedEntry>,
    filters: &SearchFilters,
) -> Vec<&'a TaggedEntry> {
    let mut result = entries;

    if let Some(ref tags) = filters.tags {
        let filter_tags: Vec<String> = tags.split(',').map(|t| t.trim().to_lowercase()).collect();
        result.retain(|e| {
            filter_tags
                .iter()
                .all(|ft| e.tags().iter().any(|t| t.to_lowercase() == *ft))
        });
    }

    if let Some(ref lang) = filters.lang {
        let normalized = normalize_language(lang);
        result.retain(|e| {
            e.languages()
                .map(|langs| langs.iter().any(|l| l.language == normalized))
                .unwrap_or(false)
        });
    }

    if let Some(ref entry_type) = filters.entry_type {
        result.retain(|e| e.entry_type == *entry_type);
    }

    result
}

#[derive(Debug, Default)]
pub struct SearchFilters {
    pub tags: Option<String>,
    pub lang: Option<String>,
    pub entry_type: Option<String>,
}

pub fn is_multi_source() -> bool {
    load_config().sources.len() > 1
}

// ---------------------------------------------------------------------------
// Levenshtein fuzzy matching (ported from JS)
// ---------------------------------------------------------------------------

fn levenshtein_distance(a: &str, b: &str, max_distance: usize) -> usize {
    if a == b {
        return 0;
    }
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let diff = if a.len() > b.len() {
        a.len() - b.len()
    } else {
        b.len() - a.len()
    };
    if diff > max_distance {
        return max_distance + 1;
    }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut previous: Vec<usize> = (0..=b_bytes.len()).collect();
    let mut current = vec![0usize; b_bytes.len() + 1];

    for i in 1..=a_bytes.len() {
        current[0] = i;
        let mut row_min = current[0];
        for j in 1..=b_bytes.len() {
            let cost = if a_bytes[i - 1] == b_bytes[j - 1] {
                0
            } else {
                1
            };
            current[j] = (previous[j] + 1)
                .min(current[j - 1] + 1)
                .min(previous[j - 1] + cost);
            row_min = row_min.min(current[j]);
        }
        if row_min > max_distance {
            return max_distance + 1;
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[b_bytes.len()]
}

struct CompactWeights {
    exact: f64,
    prefix: f64,
    contains: f64,
    fuzzy: f64,
}

fn score_compact_candidate(
    query_compact: &str,
    candidate_compact: &str,
    weights: &CompactWeights,
) -> f64 {
    if query_compact.is_empty() || candidate_compact.is_empty() {
        return 0.0;
    }
    if candidate_compact == query_compact {
        return weights.exact;
    }
    if query_compact.len() < 3 {
        return 0.0;
    }

    let length_penalty = if candidate_compact.len() > query_compact.len() {
        candidate_compact.len() - query_compact.len()
    } else {
        query_compact.len() - candidate_compact.len()
    } as f64;

    let min_len = candidate_compact.len().min(query_compact.len()) as f64;
    let max_len = candidate_compact.len().max(query_compact.len()) as f64;
    let length_ratio = min_len / max_len;

    if (candidate_compact.starts_with(query_compact)
        || query_compact.starts_with(candidate_compact))
        && length_ratio >= 0.6
    {
        return (weights.prefix - length_penalty).max(0.0);
    }

    if (candidate_compact.contains(query_compact) || query_compact.contains(candidate_compact))
        && length_ratio >= 0.75
    {
        return (weights.contains - length_penalty).max(0.0);
    }

    if query_compact.len() < 5 {
        return 0.0;
    }

    let max_dist = if query_compact.len() <= 5 {
        1
    } else if query_compact.len() <= 8 {
        2
    } else {
        3
    };

    let distance = levenshtein_distance(query_compact, candidate_compact, max_dist);
    if distance > max_dist {
        return 0.0;
    }

    (weights.fuzzy - (distance as f64 * 20.0) - length_penalty).max(0.0)
}

fn split_compact_segments(text: &str) -> Vec<String> {
    let mut segments: HashSet<String> = HashSet::new();
    for seg in text.split('/') {
        let c = compact_identifier(seg);
        if !c.is_empty() {
            segments.insert(c);
        }
    }
    for seg in text.split(&['/', '_', '.', ' ', '-'][..]) {
        let c = compact_identifier(seg);
        if !c.is_empty() {
            segments.insert(c);
        }
    }
    segments.into_iter().collect()
}

fn score_entry_lexical_variant(entry: &TaggedEntry, query_compact: &str) -> f64 {
    if query_compact.len() < 2 {
        return 0.0;
    }

    let name_compact = compact_identifier(entry.name());
    let id_compact = compact_identifier(entry.id());
    let id_segments = split_compact_segments(entry.id());
    let name_segments = split_compact_segments(entry.name());

    let mut best = 0.0f64;

    best = best.max(score_compact_candidate(
        query_compact,
        &name_compact,
        &CompactWeights {
            exact: 620.0,
            prefix: 560.0,
            contains: 520.0,
            fuzzy: 500.0,
        },
    ));

    best = best.max(score_compact_candidate(
        query_compact,
        &id_compact,
        &CompactWeights {
            exact: 600.0,
            prefix: 540.0,
            contains: 500.0,
            fuzzy: 470.0,
        },
    ));

    for (idx, segment) in id_segments.iter().enumerate() {
        let seg_score = score_compact_candidate(
            query_compact,
            segment,
            &CompactWeights {
                exact: 580.0,
                prefix: 530.0,
                contains: 490.0,
                fuzzy: 460.0,
            },
        );
        if seg_score == 0.0 {
            continue;
        }

        let mut bonus = 0.0;
        if idx == 0 {
            bonus += 10.0;
        }
        if idx == id_segments.len() - 1 {
            bonus += 10.0;
        }
        if query_compact == id_segments[0] {
            bonus += 60.0;
        }
        if query_compact == id_segments[id_segments.len() - 1] {
            bonus += 25.0;
        }
        if id_segments.len() > 1
            && query_compact == id_segments[0]
            && query_compact == id_segments[id_segments.len() - 1]
        {
            bonus += 40.0;
        }

        best = best.max(seg_score + bonus);
    }

    for segment in &name_segments {
        best = best.max(score_compact_candidate(
            query_compact,
            segment,
            &CompactWeights {
                exact: 560.0,
                prefix: 520.0,
                contains: 480.0,
                fuzzy: 450.0,
            },
        ));
    }

    best
}

fn score_entry_lexical_boost(
    entry: &TaggedEntry,
    normalized_query: &str,
    rescue_terms: &[String],
) -> f64 {
    let mut query_compacts: Vec<String> = vec![compact_identifier(normalized_query)];
    for term in rescue_terms {
        let c = compact_identifier(term);
        if !query_compacts.contains(&c) {
            query_compacts.push(c);
        }
    }
    query_compacts.retain(|c| c.len() >= 2);

    let mut best = 0.0f64;
    for qc in &query_compacts {
        best = best.max(score_entry_lexical_variant(entry, qc));
    }
    best
}

fn get_missing_query_terms(normalized_query: &str, index: &SearchIndex) -> Vec<String> {
    match &index.inverted_index {
        Some(inv) => tokenize(normalized_query)
            .into_iter()
            .filter(|term| !inv.contains_key(term.as_str()))
            .collect(),
        None => vec![],
    }
}

fn should_run_global_lexical_scan(
    normalized_query: &str,
    result_count: usize,
    index: &Option<SearchIndex>,
) -> bool {
    let idx = match index {
        Some(idx) => idx,
        None => return true,
    };

    if result_count == 0 {
        return true;
    }
    if idx.inverted_index.is_none() {
        return false;
    }

    let query_terms = tokenize(normalized_query);
    if query_terms.len() < 2 {
        return false;
    }

    !get_missing_query_terms(normalized_query, idx).is_empty()
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

/// Search entries by query string.
pub fn search_entries(
    query: &str,
    filters: &SearchFilters,
    merged: &MergedRegistry,
) -> Vec<TaggedEntry> {
    let normalized_query = normalize_query(query);
    let entries = apply_source_filter(get_all_entries(merged));

    // Deduplicate
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for entry in entries {
        let key = search_lookup_id(&entry.source_name, entry.id());
        if seen.insert(key) {
            deduped.push(entry);
        }
    }

    // Build entry lookup by namespaced id
    let entry_by_key: HashMap<String, &TaggedEntry> = deduped
        .iter()
        .map(|e| (search_lookup_id(&e.source_name, e.id()), *e))
        .collect();

    if normalized_query.is_empty() {
        let filtered = apply_filters(deduped, filters);
        return filtered.into_iter().cloned().collect();
    }

    let mut result_by_key: HashMap<String, (&TaggedEntry, f64)> = HashMap::new();

    if let Some(ref search_index) = merged.search_index {
        // BM25 search — results use namespaced IDs
        let bm25_results = bm25::search(&normalized_query, search_index, None);
        for r in &bm25_results {
            if let Some(entry) = entry_by_key.get(&r.id) {
                let key = search_lookup_id(&entry.source_name, entry.id());
                if r.score > 0.0 {
                    result_by_key.insert(key, (*entry, r.score));
                }
            }
        }
    } else {
        // Fallback: keyword matching
        let q = normalized_query.to_lowercase();
        let words: Vec<&str> = q.split_whitespace().collect();

        for entry in &deduped {
            let mut score = 0.0f64;
            let id_lower = entry.id().to_lowercase();
            let name_lower = entry.name().to_lowercase();

            if id_lower == q {
                score += 100.0;
            } else if id_lower.contains(&q) {
                score += 50.0;
            }

            if name_lower == q {
                score += 80.0;
            } else if name_lower.contains(&q) {
                score += 40.0;
            }

            for word in &words {
                if id_lower.contains(word) {
                    score += 10.0;
                }
                if name_lower.contains(word) {
                    score += 10.0;
                }
                if entry.description().to_lowercase().contains(word) {
                    score += 5.0;
                }
                if entry.tags().iter().any(|t| t.to_lowercase().contains(word)) {
                    score += 15.0;
                }
            }

            if score > 0.0 {
                let key = search_lookup_id(&entry.source_name, entry.id());
                result_by_key.insert(key, (*entry, score));
            }
        }
    }

    // Lexical boost: fuzzy matching to rescue entries BM25 missed
    let lexical_candidates = if !should_run_global_lexical_scan(
        &normalized_query,
        result_by_key.len(),
        &merged.search_index,
    ) {
        // Only boost existing results
        result_by_key.values().map(|(e, _)| *e).collect::<Vec<_>>()
    } else {
        // Scan all entries
        deduped.clone()
    };

    let rescue_terms: Vec<String> = if !result_by_key.is_empty() {
        if let Some(ref idx) = merged.search_index {
            get_missing_query_terms(&normalized_query, idx)
                .into_iter()
                .filter(|t| t.len() >= 5)
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    for entry in &lexical_candidates {
        let boost = score_entry_lexical_boost(entry, &normalized_query, &rescue_terms);
        if boost == 0.0 {
            continue;
        }

        let key = search_lookup_id(&entry.source_name, entry.id());
        if let Some(existing) = result_by_key.get_mut(&key) {
            existing.1 += boost;
        } else {
            result_by_key.insert(key, (*entry, boost));
        }
    }

    // Apply filters
    let mut results: Vec<(&TaggedEntry, f64)> = result_by_key.into_values().collect();
    let filtered_entries: Vec<&TaggedEntry> = {
        let refs: Vec<&TaggedEntry> = results.iter().map(|(e, _)| *e).collect();
        apply_filters(refs, filters)
    };
    let filtered_set: HashSet<*const TaggedEntry> =
        filtered_entries.iter().map(|e| *e as *const _).collect();
    results.retain(|(e, _)| filtered_set.contains(&(*e as *const _)));

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results.into_iter().map(|(e, _)| e.clone()).collect()
}

// ---------------------------------------------------------------------------
// Entry lookup
// ---------------------------------------------------------------------------

pub struct EntryLookup {
    pub entry: Option<TaggedEntry>,
    pub ambiguous: bool,
    pub alternatives: Vec<String>,
}

pub fn get_entry(id_or_namespaced: &str, merged: &MergedRegistry) -> EntryLookup {
    let normalized = normalize_query(id_or_namespaced);
    let all = apply_source_filter(get_all_entries(merged));

    // Check for source:id format
    if let Some(colon_idx) = normalized.find(':') {
        let source_name = &normalized[..colon_idx];
        let id = &normalized[colon_idx + 1..];
        let entry = all
            .into_iter()
            .find(|e| e.source_name == source_name && e.id() == id)
            .cloned();
        return EntryLookup {
            entry,
            ambiguous: false,
            alternatives: vec![],
        };
    }

    // Bare id
    let matches: Vec<&TaggedEntry> = all.into_iter().filter(|e| e.id() == normalized).collect();

    match matches.len() {
        0 => EntryLookup {
            entry: None,
            ambiguous: false,
            alternatives: vec![],
        },
        1 => EntryLookup {
            entry: Some(matches[0].clone()),
            ambiguous: false,
            alternatives: vec![],
        },
        _ => EntryLookup {
            entry: None,
            ambiguous: true,
            alternatives: matches
                .iter()
                .map(|e| format!("{}:{}", e.source_name, e.id()))
                .collect(),
        },
    }
}

/// List entries with optional filters.
pub fn list_entries(filters: &SearchFilters, merged: &MergedRegistry) -> Vec<TaggedEntry> {
    let entries = apply_source_filter(get_all_entries(merged));

    // Deduplicate
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for entry in entries {
        let key = format!("{}:{}", entry.source_name, entry.id());
        if seen.insert(key) {
            deduped.push(entry);
        }
    }

    let filtered = apply_filters(deduped, filters);
    filtered.into_iter().cloned().collect()
}

// ---------------------------------------------------------------------------
// Path resolution
// ---------------------------------------------------------------------------

pub enum ResolvedPath {
    Ok {
        source: SourceConfig,
        path: String,
        files: Vec<String>,
    },
    NeedsLanguage {
        available: Vec<String>,
    },
    VersionNotFound {
        requested: String,
        available: Vec<String>,
    },
}

pub fn resolve_doc_path(
    entry: &TaggedEntry,
    language: Option<&str>,
    version: Option<&str>,
) -> Option<ResolvedPath> {
    match &entry.kind {
        EntryKind::Skill(s) => {
            if s.path.is_empty() {
                return None;
            }
            Some(ResolvedPath::Ok {
                source: entry.source_obj.clone(),
                path: s.path.clone(),
                files: s.files.clone(),
            })
        }
        EntryKind::Doc(d) => {
            let lang = language.map(normalize_language);

            let lang_obj = if let Some(ref lang) = lang {
                d.languages.iter().find(|l| l.language == *lang)
            } else if d.languages.len() == 1 {
                d.languages.first()
            } else {
                return Some(ResolvedPath::NeedsLanguage {
                    available: d.languages.iter().map(|l| l.language.clone()).collect(),
                });
            };

            let lang_obj = lang_obj?;

            let ver_obj = if let Some(version) = version {
                match lang_obj.versions.iter().find(|v| v.version == version) {
                    Some(v) => v,
                    None => {
                        return Some(ResolvedPath::VersionNotFound {
                            requested: version.to_string(),
                            available: lang_obj
                                .versions
                                .iter()
                                .map(|v| v.version.clone())
                                .collect(),
                        })
                    }
                }
            } else {
                let rec = &lang_obj.recommended_version;
                lang_obj
                    .versions
                    .iter()
                    .find(|v| v.version == *rec)
                    .or(lang_obj.versions.first())?
            };

            if ver_obj.path.is_empty() {
                return None;
            }

            Some(ResolvedPath::Ok {
                source: entry.source_obj.clone(),
                path: ver_obj.path.clone(),
                files: ver_obj.files.clone(),
            })
        }
    }
}

pub fn resolve_entry_file(
    resolved: &ResolvedPath,
    entry_type: &str,
) -> Option<(String, String, Vec<String>)> {
    match resolved {
        ResolvedPath::Ok { path, files, .. } => {
            let file_name = if entry_type == "skill" {
                "SKILL.md"
            } else {
                "DOC.md"
            };
            Some((
                format!("{}/{}", path, file_name),
                path.clone(),
                files.clone(),
            ))
        }
        _ => None,
    }
}
