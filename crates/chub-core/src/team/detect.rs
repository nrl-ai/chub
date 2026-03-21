use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Serialize;

/// A detected dependency from a project file.
#[derive(Debug, Clone, Serialize)]
pub struct DetectedDep {
    pub name: String,
    pub version: Option<String>,
    pub source_file: String,
    pub language: String,
}

/// A match between a detected dependency and a registry doc.
#[derive(Debug, Clone, Serialize)]
pub struct DetectedMatch {
    pub dep: DetectedDep,
    pub doc_id: String,
    pub doc_name: String,
    pub confidence: f64,
}

/// Scan the current directory for dependency files and extract dependencies.
pub fn detect_dependencies(root: &Path) -> Vec<DetectedDep> {
    let mut deps = Vec::new();

    // package.json (npm/yarn/pnpm)
    let pkg_json = root.join("package.json");
    if pkg_json.exists() {
        deps.extend(detect_npm(&pkg_json));
    }

    // requirements.txt (Python)
    let req_txt = root.join("requirements.txt");
    if req_txt.exists() {
        deps.extend(detect_requirements_txt(&req_txt));
    }

    // pyproject.toml (Python)
    let pyproject = root.join("pyproject.toml");
    if pyproject.exists() {
        deps.extend(detect_pyproject(&pyproject));
    }

    // Cargo.toml (Rust)
    let cargo_toml = root.join("Cargo.toml");
    if cargo_toml.exists() {
        deps.extend(detect_cargo(&cargo_toml));
    }

    // go.mod (Go)
    let go_mod = root.join("go.mod");
    if go_mod.exists() {
        deps.extend(detect_go_mod(&go_mod));
    }

    // Gemfile (Ruby)
    let gemfile = root.join("Gemfile");
    if gemfile.exists() {
        deps.extend(detect_gemfile(&gemfile));
    }

    // Pipfile (Python)
    let pipfile = root.join("Pipfile");
    if pipfile.exists() {
        deps.extend(detect_pipfile(&pipfile));
    }

    // pom.xml (Java/Maven)
    let pom_xml = root.join("pom.xml");
    if pom_xml.exists() {
        deps.extend(detect_pom_xml(&pom_xml));
    }

    // build.gradle (Java/Gradle)
    let build_gradle = root.join("build.gradle");
    if build_gradle.exists() {
        deps.extend(detect_build_gradle(&build_gradle));
    }

    // build.gradle.kts (Kotlin DSL)
    let build_gradle_kts = root.join("build.gradle.kts");
    if build_gradle_kts.exists() {
        deps.extend(detect_build_gradle(&build_gradle_kts));
    }

    // Deduplicate by name
    let mut seen = std::collections::HashSet::new();
    deps.retain(|d| seen.insert(d.name.clone()));

    deps
}

fn detect_npm(path: &Path) -> Vec<DetectedDep> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let mut deps = Vec::new();
    for key in &["dependencies", "devDependencies"] {
        if let Some(obj) = json.get(key).and_then(|v| v.as_object()) {
            for (name, version) in obj {
                deps.push(DetectedDep {
                    name: name.clone(),
                    version: version.as_str().map(|s| s.to_string()),
                    source_file: "package.json".to_string(),
                    language: "javascript".to_string(),
                });
            }
        }
    }
    deps
}

fn detect_requirements_txt(path: &Path) -> Vec<DetectedDep> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#') && !l.starts_with('-'))
        .map(|line| {
            let parts: Vec<&str> = line
                .splitn(2, |c: char| {
                    c == '=' || c == '>' || c == '<' || c == '!' || c == '~'
                })
                .collect();
            let name = parts[0].trim().to_string();
            let version = if parts.len() > 1 {
                Some(
                    parts[1]
                        .trim_matches(|c: char| {
                            c == '='
                                || c == '>'
                                || c == '<'
                                || c == '!'
                                || c == '~'
                                || c.is_whitespace()
                        })
                        .to_string(),
                )
            } else {
                None
            };
            DetectedDep {
                name,
                version,
                source_file: "requirements.txt".to_string(),
                language: "python".to_string(),
            }
        })
        .collect()
}

fn detect_pyproject(path: &Path) -> Vec<DetectedDep> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let toml_val: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let mut deps = Vec::new();

    // [project.dependencies]
    if let Some(project_deps) = toml_val
        .get("project")
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_array())
    {
        for dep in project_deps {
            if let Some(s) = dep.as_str() {
                let name = s
                    .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
                    .next()
                    .unwrap_or(s)
                    .to_string();
                deps.push(DetectedDep {
                    name,
                    version: None,
                    source_file: "pyproject.toml".to_string(),
                    language: "python".to_string(),
                });
            }
        }
    }

    // [tool.poetry.dependencies]
    if let Some(poetry_deps) = toml_val
        .get("tool")
        .and_then(|t| t.get("poetry"))
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_table())
    {
        for (name, val) in poetry_deps {
            if name == "python" {
                continue;
            }
            let version = val.as_str().map(|s| s.to_string());
            deps.push(DetectedDep {
                name: name.clone(),
                version,
                source_file: "pyproject.toml".to_string(),
                language: "python".to_string(),
            });
        }
    }

    deps
}

fn detect_cargo(path: &Path) -> Vec<DetectedDep> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let toml_val: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let mut deps = Vec::new();
    for section in &["dependencies", "dev-dependencies"] {
        if let Some(table) = toml_val.get(section).and_then(|d| d.as_table()) {
            for (name, val) in table {
                let version = match val {
                    toml::Value::String(s) => Some(s.clone()),
                    toml::Value::Table(t) => t
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    _ => None,
                };
                deps.push(DetectedDep {
                    name: name.clone(),
                    version,
                    source_file: "Cargo.toml".to_string(),
                    language: "rust".to_string(),
                });
            }
        }
    }

    deps
}

fn detect_go_mod(path: &Path) -> Vec<DetectedDep> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut deps = Vec::new();
    let mut in_require = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("require (") || trimmed == "require (" {
            in_require = true;
            continue;
        }
        if trimmed == ")" {
            in_require = false;
            continue;
        }
        if in_require || trimmed.starts_with("require ") {
            let dep_line = if let Some(stripped) = trimmed.strip_prefix("require ") {
                stripped
            } else {
                trimmed
            };
            let parts: Vec<&str> = dep_line.split_whitespace().collect();
            if !parts.is_empty() {
                let name = parts[0].rsplit('/').next().unwrap_or(parts[0]).to_string();
                let version = parts.get(1).map(|s| s.to_string());
                deps.push(DetectedDep {
                    name,
                    version,
                    source_file: "go.mod".to_string(),
                    language: "go".to_string(),
                });
            }
        }
    }

    deps
}

fn detect_gemfile(path: &Path) -> Vec<DetectedDep> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("gem ") {
                return None;
            }
            let rest = &trimmed[4..];
            // Extract gem name from quotes
            let name = rest.split(['\'', '"']).nth(1)?.to_string();
            Some(DetectedDep {
                name,
                version: None,
                source_file: "Gemfile".to_string(),
                language: "ruby".to_string(),
            })
        })
        .collect()
}

fn detect_pipfile(path: &Path) -> Vec<DetectedDep> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let toml_val: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let mut deps = Vec::new();
    for section in &["packages", "dev-packages"] {
        if let Some(table) = toml_val.get(section).and_then(|d| d.as_table()) {
            for (name, val) in table {
                let version = val.as_str().map(|s| s.to_string());
                deps.push(DetectedDep {
                    name: name.clone(),
                    version,
                    source_file: "Pipfile".to_string(),
                    language: "python".to_string(),
                });
            }
        }
    }
    deps
}

fn detect_pom_xml(path: &Path) -> Vec<DetectedDep> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut deps = Vec::new();
    let mut in_dependency = false;
    let mut group_id = String::new();
    let mut artifact_id = String::new();
    let mut version = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "<dependency>" {
            in_dependency = true;
            group_id.clear();
            artifact_id.clear();
            version = None;
            continue;
        }
        if trimmed == "</dependency>" {
            if in_dependency && !artifact_id.is_empty() {
                deps.push(DetectedDep {
                    name: if group_id.is_empty() {
                        artifact_id.clone()
                    } else {
                        format!("{}:{}", group_id, artifact_id)
                    },
                    version: version.clone(),
                    source_file: "pom.xml".to_string(),
                    language: "java".to_string(),
                });
            }
            in_dependency = false;
            continue;
        }
        if in_dependency {
            if let Some(val) = extract_xml_value(trimmed, "groupId") {
                group_id = val;
            } else if let Some(val) = extract_xml_value(trimmed, "artifactId") {
                artifact_id = val;
            } else if let Some(val) = extract_xml_value(trimmed, "version") {
                if !val.starts_with("${") {
                    version = Some(val);
                }
            }
        }
    }

    deps
}

fn extract_xml_value(line: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    if let Some(start) = line.find(&open) {
        if let Some(end) = line.find(&close) {
            let val = &line[start + open.len()..end];
            return Some(val.trim().to_string());
        }
    }
    None
}

fn detect_build_gradle(path: &Path) -> Vec<DetectedDep> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut deps = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        // Match patterns like: implementation 'group:artifact:version'
        // or: implementation "group:artifact:version"
        for keyword in &[
            "implementation",
            "api",
            "compileOnly",
            "runtimeOnly",
            "testImplementation",
        ] {
            if !trimmed.starts_with(keyword) {
                continue;
            }
            let rest = &trimmed[keyword.len()..].trim_start();
            // Extract quoted string
            let quote = if rest.starts_with('\'') {
                '\''
            } else if rest.starts_with('"') {
                '"'
            } else if rest.starts_with('(') {
                // implementation("group:artifact:version")
                let inner = rest.trim_start_matches('(').trim_end_matches(')');
                if inner.starts_with('\'') {
                    '\''
                } else if inner.starts_with('"') {
                    '"'
                } else {
                    continue;
                }
            } else {
                continue;
            };
            let content_str = if rest.starts_with('(') {
                rest.trim_start_matches('(').trim_end_matches(')')
            } else {
                rest
            };
            let parts: Vec<&str> = content_str.trim_matches(quote).split(':').collect();
            if parts.len() >= 2 {
                let name = format!("{}:{}", parts[0], parts[1]);
                let version = parts.get(2).map(|s| s.to_string());
                deps.push(DetectedDep {
                    name,
                    version,
                    source_file: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    language: "java".to_string(),
                });
            }
            break;
        }
    }

    deps
}

/// Match detected dependencies to known docs in the registry.
/// Uses simple name matching — the dep name is searched against doc IDs.
pub fn match_deps_to_docs(
    deps: &[DetectedDep],
    doc_ids: &[(String, String)], // (id, name) pairs from registry
) -> Vec<DetectedMatch> {
    let mut matches = Vec::new();

    // Build a lookup: lowercase name → (id, name)
    let mut id_by_name: HashMap<String, (String, String)> = HashMap::new();
    for (id, name) in doc_ids {
        // Index by last segment of id (e.g., "openai/chat" → "openai")
        let parts: Vec<&str> = id.split('/').collect();
        if !parts.is_empty() {
            id_by_name.insert(parts[0].to_lowercase(), (id.clone(), name.clone()));
        }
        // Also index by full id
        id_by_name.insert(id.to_lowercase(), (id.clone(), name.clone()));
    }

    for dep in deps {
        let dep_lower = dep.name.to_lowercase();

        // Try exact match on first segment
        if let Some((doc_id, doc_name)) = id_by_name.get(&dep_lower) {
            matches.push(DetectedMatch {
                dep: dep.clone(),
                doc_id: doc_id.clone(),
                doc_name: doc_name.clone(),
                confidence: 1.0,
            });
            continue;
        }

        // Try partial match
        for (key, (doc_id, doc_name)) in &id_by_name {
            if key.contains(&dep_lower) || dep_lower.contains(key.as_str()) {
                matches.push(DetectedMatch {
                    dep: dep.clone(),
                    doc_id: doc_id.clone(),
                    doc_name: doc_name.clone(),
                    confidence: 0.5,
                });
                break;
            }
        }
    }

    matches
}
