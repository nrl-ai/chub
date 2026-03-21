use serde::{Deserialize, Serialize};

/// A version entry within a language variant of a doc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionEntry {
    pub version: String,
    pub path: String,
    pub files: Vec<String>,
    pub size: u64,
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
}

/// A language variant containing versions for a doc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanguageEntry {
    pub language: String,
    pub versions: Vec<VersionEntry>,
    #[serde(rename = "recommendedVersion")]
    pub recommended_version: String,
}

/// A documentation entry in the registry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: String,
    pub tags: Vec<String>,
    pub languages: Vec<LanguageEntry>,
}

/// A skill entry in the registry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: String,
    pub tags: Vec<String>,
    pub path: String,
    pub files: Vec<String>,
    pub size: u64,
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
}

/// The top-level registry.json structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Registry {
    pub version: String,
    pub generated: String,
    pub docs: Vec<DocEntry>,
    pub skills: Vec<SkillEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

/// A document in the BM25 search index.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchDocument {
    pub id: String,
    pub tokens: SearchTokens,
}

/// Tokenized fields for a search document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchTokens {
    #[serde(default)]
    pub id: Vec<String>,
    pub name: Vec<String>,
    pub description: Vec<String>,
    pub tags: Vec<String>,
}

/// BM25 parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bm25Params {
    pub k1: f64,
    pub b: f64,
}

/// Average field lengths in the search index.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AvgFieldLengths {
    #[serde(default)]
    pub id: f64,
    pub name: f64,
    pub description: f64,
    pub tags: f64,
}

/// The BM25 search index (search-index.json).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchIndex {
    pub version: String,
    pub algorithm: String,
    pub params: Bm25Params,
    #[serde(rename = "totalDocs")]
    pub total_docs: usize,
    #[serde(rename = "avgFieldLengths")]
    pub avg_field_lengths: AvgFieldLengths,
    pub idf: std::collections::HashMap<String, f64>,
    pub documents: Vec<SearchDocument>,
    /// Inverted index: term → list of document indexes.
    #[serde(
        rename = "invertedIndex",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub inverted_index: Option<std::collections::HashMap<String, Vec<usize>>>,
}

/// A unified entry type for search indexing (combines doc and skill fields).
pub enum Entry<'a> {
    Doc(&'a DocEntry),
    Skill(&'a SkillEntry),
}

impl<'a> Entry<'a> {
    pub fn id(&self) -> &str {
        match self {
            Entry::Doc(d) => &d.id,
            Entry::Skill(s) => &s.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Entry::Doc(d) => &d.name,
            Entry::Skill(s) => &s.name,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Entry::Doc(d) => &d.description,
            Entry::Skill(s) => &s.description,
        }
    }

    pub fn tags(&self) -> &[String] {
        match self {
            Entry::Doc(d) => &d.tags,
            Entry::Skill(s) => &s.tags,
        }
    }
}

/// Parsed YAML frontmatter from DOC.md or SKILL.md.
#[derive(Debug, Clone, Default)]
pub struct Frontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: FrontmatterMetadata,
}

/// Metadata block within frontmatter.
#[derive(Debug, Clone, Default)]
pub struct FrontmatterMetadata {
    pub languages: Option<String>,
    pub versions: Option<String>,
    pub source: Option<String>,
    pub tags: Option<String>,
    pub updated_on: Option<String>,
}

/// Author-provided registry.json (may have partial data).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorRegistry {
    #[serde(default)]
    pub docs: Vec<AuthorDoc>,
    #[serde(default)]
    pub skills: Vec<AuthorSkill>,
}

/// A doc entry in an author's registry.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorDoc {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub languages: Option<Vec<LanguageEntry>>,
}

/// A skill entry in an author's registry.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorSkill {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    pub path: String,
    #[serde(default)]
    pub files: Option<Vec<String>>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(rename = "lastUpdated", default)]
    pub last_updated: Option<String>,
}
