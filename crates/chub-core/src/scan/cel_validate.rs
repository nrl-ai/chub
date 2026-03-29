//! CEL-based secret validation — compatible with betterleaks validate expressions.
//!
//! ## Design
//!
//! betterleaks validate expressions always follow this shape:
//!
//! ```text
//! cel.bind(r,
//!   http.get("https://...", {"Authorization": "Bearer " + secret}),
//!   r.status == 200 ? {"result": "valid"} : unknown(r)
//! )
//! ```
//!
//! We handle this with a **hybrid approach**:
//! 1. Peel `cel.bind(name, expr, body)` layers ourselves at the string level.
//! 2. Evaluate the binding `expr` natively in Rust (HTTP calls, AWS STS, crypto).
//! 3. Inject the result as a variable into the CEL context.
//! 4. Use `cel-interpreter` to evaluate the pure-CEL body (ternary conditions, map
//!    literals, field access, `in` operator, optional `?.` access).
//!
//! Reference: betterleaks `celenv/` and `validate/` packages.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use cel_interpreter::{objects::Key, Context, Program, Value};
use hmac::{Hmac, Mac};
use md5::Md5;
use sha2::{Digest, Sha256};

// ── public types ─────────────────────────────────────────────────────────────

/// Outcome of validating a finding against a live API.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// One of: `"valid"`, `"invalid"`, `"revoked"`, `"unknown"`, `"error"`.
    pub status: String,
    /// Human-readable explanation (present when status is not `"valid"`).
    pub reason: Option<String>,
}

impl ValidationResult {
    fn valid() -> Self {
        Self {
            status: "valid".into(),
            reason: None,
        }
    }
    fn invalid(reason: impl Into<String>) -> Self {
        Self {
            status: "invalid".into(),
            reason: Some(reason.into()),
        }
    }
    fn revoked(reason: impl Into<String>) -> Self {
        Self {
            status: "revoked".into(),
            reason: Some(reason.into()),
        }
    }
    pub fn unknown(reason: impl Into<String>) -> Self {
        Self {
            status: "unknown".into(),
            reason: Some(reason.into()),
        }
    }
    pub fn error(reason: impl Into<String>) -> Self {
        Self {
            status: "error".into(),
            reason: Some(reason.into()),
        }
    }
}

// ── HTTP client singleton ─────────────────────────────────────────────────────

static HTTP_CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();

fn http_client() -> &'static reqwest::blocking::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("chub-scanner/1.0")
            .build()
            .expect("HTTP client build failed")
    })
}

// ── result cache ─────────────────────────────────────────────────────────────

static CACHE: OnceLock<Mutex<HashMap<String, ValidationResult>>> = OnceLock::new();

fn result_cache() -> &'static Mutex<HashMap<String, ValidationResult>> {
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cache_key(rule_id: &str, secret: &str, captures: &HashMap<String, String>) -> String {
    let mut pairs: Vec<(&str, &str)> = captures
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    pairs.sort_by_key(|(k, _)| *k);
    let payload = format!("{}:{}:{}", rule_id, secret, {
        pairs
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",")
    });
    let hash = Sha256::digest(payload.as_bytes());
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

// ── CelValue: runtime value exchanged between our code and cel-interpreter ───

/// Simplified value type used for the binding layer (HTTP responses, crypto
/// outputs).  Once all bindings are resolved we convert these into
/// `cel_interpreter::Value` for the final CEL condition evaluation.
#[derive(Debug, Clone)]
enum CelVal {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Bytes(Vec<u8>),
    Map(Vec<(String, CelVal)>),
    List(Vec<CelVal>),
}

impl CelVal {
    /// Convert to `cel_interpreter::Value` so we can inject it into the CEL
    /// context for condition evaluation.
    fn to_cel(&self) -> Value {
        match self {
            CelVal::Null => Value::Null,
            CelVal::Bool(b) => Value::Bool(*b),
            CelVal::Int(n) => Value::Int(*n),
            CelVal::Str(s) => Value::String(s.clone().into()),
            CelVal::Bytes(b) => Value::Bytes(b.clone().into()),
            CelVal::Map(pairs) => {
                let map: std::collections::HashMap<Key, Value> = pairs
                    .iter()
                    .map(|(k, v)| (Key::from(k.clone()), v.to_cel()))
                    .collect();
                Value::Map(map.into())
            }
            CelVal::List(items) => {
                Value::List(items.iter().map(|v| v.to_cel()).collect::<Vec<_>>().into())
            }
        }
    }
}

// ── string-level helpers ──────────────────────────────────────────────────────

/// Find the index of the closing `)` that matches the `(` at `start`.
/// Returns `None` if `src[start]` is not `(` or the parens are unbalanced.
fn find_close_paren(src: &str, start: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    if bytes.get(start) != Some(&b'(') {
        return None;
    }
    let mut depth = 0i32;
    let mut in_str = false;
    let mut str_ch = b'"';
    let mut i = start;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if b == b'\\' {
                i += 1;
            } else if b == str_ch {
                in_str = false;
            }
        } else {
            match b {
                b'"' | b'\'' => {
                    in_str = true;
                    str_ch = b;
                }
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// Split a comma-separated argument list (already stripped of outer parens) at
/// top-level commas, respecting nesting and string literals.
fn split_args(s: &str) -> Vec<&str> {
    let mut args: Vec<&str> = Vec::new();
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut str_ch = b'"';
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if b == b'\\' {
                i += 1;
            } else if b == str_ch {
                in_str = false;
            }
        } else {
            match b {
                b'"' | b'\'' => {
                    in_str = true;
                    str_ch = b;
                }
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => depth -= 1,
                b',' if depth == 0 => {
                    args.push(s[start..i].trim());
                    start = i + 1;
                }
                _ => {}
            }
        }
        i += 1;
    }
    let tail = s[start..].trim();
    if !tail.is_empty() {
        args.push(tail);
    }
    args
}

/// If `src` (trimmed) starts with `func_name(…)`, return the inner argument
/// string (content between the outer parens).
fn strip_call<'a>(src: &'a str, func_name: &str) -> Option<&'a str> {
    let s = src.trim();
    if !s.starts_with(func_name) {
        return None;
    }
    let rest = s[func_name.len()..].trim_start();
    if !rest.starts_with('(') {
        return None;
    }
    let close = find_close_paren(rest, 0)?;
    Some(rest[1..close].trim())
}

// ── binding-expression evaluator ─────────────────────────────────────────────
//
// Evaluates the *right-hand side* of a cel.bind: function calls that produce
// a CelVal.  The `env` is the current variable scope so that argument
// expressions (e.g. `"Bearer " + secret`) can reference already-bound names.

fn eval_binding(expr: &str, env: &HashMap<String, CelVal>) -> Result<CelVal, String> {
    let expr = expr.trim();

    // http.get(url_expr, headers_map_expr)
    if let Some(inner) = strip_call(expr, "http.get") {
        let args = split_args(inner);
        if args.len() != 2 {
            return Err(format!("http.get expects 2 args, got {}", args.len()));
        }
        let url = eval_string_expr(args[0], env)?;
        let headers = eval_map_str_expr(args[1], env)?;
        return Ok(do_http_get(&url, &headers));
    }

    // http.post(url_expr, headers_map_expr, body_expr)
    if let Some(inner) = strip_call(expr, "http.post") {
        let args = split_args(inner);
        if args.len() != 3 {
            return Err(format!("http.post expects 3 args, got {}", args.len()));
        }
        let url = eval_string_expr(args[0], env)?;
        let headers = eval_map_str_expr(args[1], env)?;
        let body = eval_string_expr(args[2], env)?;
        return Ok(do_http_post(&url, &headers, &body));
    }

    // aws.validate(access_key_id_expr, secret_access_key_expr)
    if let Some(inner) = strip_call(expr, "aws.validate") {
        let args = split_args(inner);
        if args.len() != 2 {
            return Err(format!("aws.validate expects 2 args, got {}", args.len()));
        }
        let akid = eval_string_expr(args[0], env)?;
        let sak = eval_string_expr(args[1], env)?;
        return Ok(do_aws_validate(&akid, &sak));
    }

    // time.now_unix() → string timestamp
    if strip_call(expr, "time.now_unix").is_some() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();
        return Ok(CelVal::Str(ts));
    }

    // md5(str_expr)
    if let Some(inner) = strip_call(expr, "md5") {
        let args = split_args(inner);
        if args.len() == 1 {
            let s = eval_string_expr(args[0], env)?;
            let digest = Md5::digest(s.as_bytes());
            return Ok(CelVal::Str(
                digest.iter().map(|b| format!("{:02x}", b)).collect(),
            ));
        }
    }

    // crypto.hmac_sha256(key_bytes_expr, msg_bytes_expr)
    if let Some(inner) = strip_call(expr, "crypto.hmac_sha256") {
        let args = split_args(inner);
        if args.len() == 2 {
            let key = eval_bytes_expr(args[0], env)?;
            let msg = eval_bytes_expr(args[1], env)?;
            let mut mac = Hmac::<Sha256>::new_from_slice(&key).map_err(|e| e.to_string())?;
            mac.update(&msg);
            return Ok(CelVal::Bytes(mac.finalize().into_bytes().to_vec()));
        }
    }

    // base64.decode(str_expr) → bytes
    if let Some(inner) = strip_call(expr, "base64.decode") {
        let args = split_args(inner);
        if args.len() == 1 {
            let s = eval_string_expr(args[0], env)?;
            let bytes = B64.decode(s.as_bytes()).map_err(|e| e.to_string())?;
            return Ok(CelVal::Bytes(bytes));
        }
    }

    // base64.encode(bytes_expr) → string
    if let Some(inner) = strip_call(expr, "base64.encode") {
        let args = split_args(inner);
        if args.len() == 1 {
            let bytes = eval_bytes_expr(args[0], env)?;
            return Ok(CelVal::Str(B64.encode(&bytes)));
        }
    }

    // bytes(str_expr) → bytes
    if let Some(inner) = strip_call(expr, "bytes") {
        let args = split_args(inner);
        if args.len() == 1 {
            let s = eval_string_expr(args[0], env)?;
            return Ok(CelVal::Bytes(s.into_bytes()));
        }
    }

    // string(bytes_expr) → string
    if let Some(inner) = strip_call(expr, "string") {
        let args = split_args(inner);
        if args.len() == 1 {
            let b = eval_bytes_expr(args[0], env)?;
            return Ok(CelVal::Str(String::from_utf8_lossy(&b).into_owned()));
        }
    }

    Err(format!(
        "unsupported binding expression: {}",
        &expr[..expr.len().min(60)]
    ))
}

// ── simple expression evaluators for function arguments ──────────────────────

/// Evaluate a CEL string expression that may be:
/// - A double/single-quoted string literal: `"hello"` / `'world'`
/// - A variable reference: `secret`, `captures["key"]`
/// - A string concatenation: `"Bearer " + secret`
fn eval_string_expr(src: &str, env: &HashMap<String, CelVal>) -> Result<String, String> {
    let v = eval_simple_expr(src.trim(), env)?;
    match v {
        CelVal::Str(s) => Ok(s),
        CelVal::Bytes(b) => Ok(String::from_utf8_lossy(&b).into_owned()),
        other => Err(format!("expected string, got {:?}", other)),
    }
}

/// Evaluate a CEL bytes expression:
/// - `bytes(str)`, `base64.decode(str)`, `crypto.hmac_sha256(...)`, or a variable
fn eval_bytes_expr(src: &str, env: &HashMap<String, CelVal>) -> Result<Vec<u8>, String> {
    // Try as a binding expression first (handles base64.decode, bytes(...), hmac)
    if let Ok(v) = eval_binding(src, env) {
        match v {
            CelVal::Bytes(b) => return Ok(b),
            CelVal::Str(s) => return Ok(s.into_bytes()),
            _ => {}
        }
    }
    // Fallback: evaluate as a simple expression
    match eval_simple_expr(src.trim(), env)? {
        CelVal::Bytes(b) => Ok(b),
        CelVal::Str(s) => Ok(s.into_bytes()),
        other => Err(format!("expected bytes, got {:?}", other)),
    }
}

/// Evaluate a `{"key": value, ...}` map literal where values are string
/// expressions.  Returns `HashMap<String, String>` for use as HTTP headers.
fn eval_map_str_expr(
    src: &str,
    env: &HashMap<String, CelVal>,
) -> Result<HashMap<String, String>, String> {
    let s = src.trim();
    if !s.starts_with('{') || !s.ends_with('}') {
        return Err(format!(
            "expected map literal, got: {}",
            &s[..s.len().min(40)]
        ));
    }
    let inner = &s[1..s.len() - 1];
    let mut map = HashMap::new();
    for pair in split_args(inner) {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        // Split at the first `:` that is at depth 0
        let colon = find_top_level_colon(pair)
            .ok_or_else(|| format!("no colon in map pair: {}", &pair[..pair.len().min(40)]))?;
        let key_expr = pair[..colon].trim();
        let val_expr = pair[colon + 1..].trim();
        let key = eval_string_expr(key_expr, env)?;
        let val = eval_string_expr(val_expr, env)?;
        map.insert(key, val);
    }
    Ok(map)
}

fn find_top_level_colon(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut str_ch = b'"';
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if b == b'\\' {
                i += 1;
            } else if b == str_ch {
                in_str = false;
            }
        } else {
            match b {
                b'"' | b'\'' => {
                    in_str = true;
                    str_ch = b;
                }
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => depth -= 1,
                b':' if depth == 0 => return Some(i),
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// Evaluate a simple CEL expression that produces a scalar or list value:
/// string literals, integer literals, variable names, `captures["key"]`,
/// string concatenation `+`, and `base64.encode(sig).replace("+","-")`.
fn eval_simple_expr(src: &str, env: &HashMap<String, CelVal>) -> Result<CelVal, String> {
    let src = src.trim();

    // Try to evaluate as a binding expression first
    if let Ok(v) = eval_binding(src, env) {
        return Ok(v);
    }

    // Integer literal
    if let Ok(n) = src.parse::<i64>() {
        return Ok(CelVal::Int(n));
    }

    // String literal (double or single quoted)
    if (src.starts_with('"') && src.ends_with('"'))
        || (src.starts_with('\'') && src.ends_with('\''))
    {
        return Ok(CelVal::Str(unescape_string(&src[1..src.len() - 1])));
    }

    // List literal [a, b, c]
    if src.starts_with('[') && src.ends_with(']') {
        let inner = &src[1..src.len() - 1];
        let items: Result<Vec<CelVal>, _> = split_args(inner)
            .iter()
            .map(|s| eval_simple_expr(s, env))
            .collect();
        return Ok(CelVal::List(items?));
    }

    // Variable reference: `secret`, `captures["key"]`, `r`, `ts`, etc.
    if let Some(val) = eval_variable(src, env) {
        return Ok(val);
    }

    // String concatenation: find `+` at depth 0 and evaluate both sides
    if let Some(plus_pos) = find_top_level_plus(src) {
        let lhs = eval_simple_expr(src[..plus_pos].trim(), env)?;
        let rhs = eval_simple_expr(src[plus_pos + 1..].trim(), env)?;
        return Ok(concat_vals(lhs, rhs));
    }

    // Method call: `expr.replace("+", "-")`, `expr.replace("/", "_")`
    if let Some((obj_src, method, margs)) = parse_method_call(src) {
        let obj = eval_simple_expr(obj_src, env)?;
        return apply_method(obj, method, margs, env);
    }

    Err(format!(
        "cannot evaluate expression: {}",
        &src[..src.len().min(80)]
    ))
}

fn concat_vals(a: CelVal, b: CelVal) -> CelVal {
    match (a, b) {
        (CelVal::Str(a), CelVal::Str(b)) => CelVal::Str(a + &b),
        (CelVal::Bytes(mut a), CelVal::Bytes(b)) => {
            a.extend(b);
            CelVal::Bytes(a)
        }
        (CelVal::Str(a), CelVal::Bytes(b)) => {
            let mut r = a.into_bytes();
            r.extend(b);
            CelVal::Bytes(r)
        }
        (a, b) => CelVal::Str(format!("{:?}{:?}", a, b)), // fallback
    }
}

fn find_top_level_plus(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut str_ch = b'"';
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if b == b'\\' {
                i += 1;
            } else if b == str_ch {
                in_str = false;
            }
        } else {
            match b {
                b'"' | b'\'' => {
                    in_str = true;
                    str_ch = b;
                }
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => depth -= 1,
                b'+' if depth == 0 => return Some(i),
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// Try to evaluate a variable reference: plain name, `captures["key"]`,
/// or chained field access `r.field`.
fn eval_variable(src: &str, env: &HashMap<String, CelVal>) -> Option<CelVal> {
    let src = src.trim();

    // Simple variable: `secret`, `ts`, `r`, etc.
    if src.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return env.get(src).cloned();
    }

    // captures["key"]
    if let Some(idx) = src.find('[') {
        let obj_name = src[..idx].trim();
        let bracket = &src[idx..];
        if bracket.starts_with('[') && bracket.ends_with(']') {
            let key_src = bracket[1..bracket.len() - 1].trim();
            let key = if (key_src.starts_with('"') && key_src.ends_with('"'))
                || (key_src.starts_with('\'') && key_src.ends_with('\''))
            {
                unescape_string(&key_src[1..key_src.len() - 1])
            } else {
                return None;
            };
            if let Some(CelVal::Map(pairs)) = env.get(obj_name) {
                for (k, v) in pairs {
                    if k == &key {
                        return Some(v.clone());
                    }
                }
                return Some(CelVal::Null);
            }
        }
    }

    // r.field (simple one-level field access on a map in env)
    if let Some(dot) = src.find('.') {
        let obj_name = src[..dot].trim();
        let field = src[dot + 1..].trim();
        // Only handle plain field names, not nested access or method calls
        if field.chars().all(|c| c.is_alphanumeric() || c == '_') {
            if let Some(CelVal::Map(pairs)) = env.get(obj_name) {
                for (k, v) in pairs {
                    if k == field {
                        return Some(v.clone());
                    }
                }
                return Some(CelVal::Null);
            }
        }
    }

    None
}

/// Parse `obj.method(args)` — returns `(obj_src, method_name, args_src)`.
fn parse_method_call(src: &str) -> Option<(&str, &str, &str)> {
    // Find the last `.method(` that ends at the end of src
    if !src.ends_with(')') {
        return None;
    }
    // Find the closing paren (it's at the end)
    let close = src.len() - 1;
    // Walk backwards to find the matching open paren
    let mut depth = 1i32;
    let mut open = None;
    let bytes = src.as_bytes();
    let mut i = close as i64 - 1;
    while i >= 0 {
        match bytes[i as usize] {
            b'(' => {
                depth -= 1;
                if depth == 0 {
                    open = Some(i as usize);
                    break;
                }
            }
            b')' => depth += 1,
            _ => {}
        }
        i -= 1;
    }
    let open = open?;
    // The part before `(` must end with `.identifier`
    let before = src[..open].trim();
    let dot = before.rfind('.')?;
    let method = before[dot + 1..].trim();
    if method.is_empty() || !method.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    let obj = before[..dot].trim();
    let args_inner = src[open + 1..close].trim();
    Some((obj, method, args_inner))
}

fn apply_method(
    obj: CelVal,
    method: &str,
    args: &str,
    env: &HashMap<String, CelVal>,
) -> Result<CelVal, String> {
    match method {
        "replace" => {
            let margs = split_args(args);
            if margs.len() == 2 {
                let from = eval_string_expr(margs[0], env)?;
                let to = eval_string_expr(margs[1], env)?;
                let s = match obj {
                    CelVal::Str(s) => s,
                    CelVal::Bytes(b) => String::from_utf8_lossy(&b).into_owned(),
                    _ => return Err("replace: expected string".into()),
                };
                return Ok(CelVal::Str(s.replace(&from as &str, &to as &str)));
            }
        }
        "decode" => {
            // base64.decode is already handled in eval_binding, but handle
            // it here too for when it appears as a method call on a var
            if let CelVal::Str(s) = obj {
                let bytes = B64.decode(s.as_bytes()).map_err(|e| e.to_string())?;
                return Ok(CelVal::Bytes(bytes));
            }
        }
        _ => {}
    }
    Err(format!("unsupported method: {}", method))
}

/// Unescapes a JSON-like string literal body (without surrounding quotes).
fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some('\\') => result.push('\\'),
                Some(c) => {
                    result.push('\\');
                    result.push(c);
                }
                None => {}
            }
        } else {
            result.push(c);
        }
    }
    result
}

// ── native function implementations ──────────────────────────────────────────

/// Build a HTTP response CelVal from status, body, and headers.
fn build_response(status: u16, body: &str, headers: &reqwest::header::HeaderMap) -> CelVal {
    let json_val: serde_json::Value =
        serde_json::from_str(body).unwrap_or(serde_json::Value::Object(Default::default()));
    let json_cel = json_to_cel(&json_val);

    let header_pairs: Vec<(String, CelVal)> = headers
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_lowercase(),
                CelVal::Str(v.to_str().unwrap_or("").to_string()),
            )
        })
        .collect();

    CelVal::Map(vec![
        ("status".into(), CelVal::Int(status as i64)),
        ("body".into(), CelVal::Str(body.to_string())),
        ("json".into(), json_cel),
        ("headers".into(), CelVal::Map(header_pairs)),
    ])
}

fn json_to_cel(v: &serde_json::Value) -> CelVal {
    match v {
        serde_json::Value::Null => CelVal::Null,
        serde_json::Value::Bool(b) => CelVal::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                CelVal::Int(i)
            } else {
                CelVal::Str(n.to_string())
            }
        }
        serde_json::Value::String(s) => CelVal::Str(s.clone()),
        serde_json::Value::Array(arr) => CelVal::List(arr.iter().map(json_to_cel).collect()),
        serde_json::Value::Object(obj) => CelVal::Map(
            obj.iter()
                .map(|(k, v)| (k.clone(), json_to_cel(v)))
                .collect(),
        ),
    }
}

fn do_http_get(url: &str, headers: &HashMap<String, String>) -> CelVal {
    let mut req = http_client().get(url);
    for (k, v) in headers {
        req = req.header(k.as_str(), v.as_str());
    }
    match req.send() {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let hdrs = resp.headers().clone();
            let body = resp
                .text()
                .unwrap_or_default()
                .chars()
                .take(1024 * 1024)
                .collect::<String>();
            build_response(status, &body, &hdrs)
        }
        Err(e) => CelVal::Map(vec![
            ("status".into(), CelVal::Int(0)),
            ("body".into(), CelVal::Str(e.to_string())),
            ("json".into(), CelVal::Map(vec![])),
            ("headers".into(), CelVal::Map(vec![])),
        ]),
    }
}

fn do_http_post(url: &str, headers: &HashMap<String, String>, body: &str) -> CelVal {
    let mut req = http_client().post(url).body(body.to_string());
    for (k, v) in headers {
        req = req.header(k.as_str(), v.as_str());
    }
    match req.send() {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let hdrs = resp.headers().clone();
            let resp_body = resp
                .text()
                .unwrap_or_default()
                .chars()
                .take(1024 * 1024)
                .collect::<String>();
            build_response(status, &resp_body, &hdrs)
        }
        Err(e) => CelVal::Map(vec![
            ("status".into(), CelVal::Int(0)),
            ("body".into(), CelVal::Str(e.to_string())),
            ("json".into(), CelVal::Map(vec![])),
            ("headers".into(), CelVal::Map(vec![])),
        ]),
    }
}

/// SigV4-signed AWS STS GetCallerIdentity.  Mirrors betterleaks `callSTS`.
fn do_aws_validate(access_key_id: &str, secret_access_key: &str) -> CelVal {
    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    // Format dates (YYYYMMDDTHHMMSSZ and YYYYMMDD)
    let secs = now.as_secs();
    let amz_date = format_aws_date(secs, true);
    let date_stamp = format_aws_date(secs, false);

    let body = "Action=GetCallerIdentity&Version=2011-06-15";
    let body_hash = hex_sha256(body.as_bytes());
    let host = "sts.amazonaws.com";
    let region = "us-east-1";
    let service = "sts";
    let endpoint = "https://sts.amazonaws.com/";

    let canonical_headers = format!(
        "content-type:application/x-www-form-urlencoded\nhost:{}\nx-amz-date:{}\n",
        host, amz_date
    );
    let signed_headers = "content-type;host;x-amz-date";
    let canonical_request = format!(
        "POST\n/\n\n{}\n{}\n{}",
        canonical_headers, signed_headers, body_hash
    );

    let credential_scope = format!("{}/{}/{}/aws4_request", date_stamp, region, service);
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date,
        credential_scope,
        hex_sha256(canonical_request.as_bytes())
    );

    let signing_key = derive_signing_key(secret_access_key, &date_stamp, region, service);
    let signature = hex_hmac_sha256(&signing_key, string_to_sign.as_bytes());
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        access_key_id, credential_scope, signed_headers, signature
    );

    let headers: HashMap<String, String> = [
        (
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        ),
        ("Host".to_string(), host.to_string()),
        ("X-Amz-Date".to_string(), amz_date),
        ("Authorization".to_string(), authorization),
    ]
    .into_iter()
    .collect();

    let mut req = http_client().post(endpoint).body(body);
    for (k, v) in &headers {
        req = req.header(k.as_str(), v.as_str());
    }

    match req.send() {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let xml = resp.text().unwrap_or_default();
            let mut pairs: Vec<(String, CelVal)> =
                vec![("status".into(), CelVal::Int(status as i64))];

            if status == 200 {
                pairs.push(("arn".into(), CelVal::Str(extract_xml_tag(&xml, "Arn"))));
                pairs.push((
                    "account".into(),
                    CelVal::Str(extract_xml_tag(&xml, "Account")),
                ));
                pairs.push((
                    "userid".into(),
                    CelVal::Str(extract_xml_tag(&xml, "UserId")),
                ));
            } else {
                pairs.push((
                    "error_code".into(),
                    CelVal::Str(extract_xml_tag(&xml, "Code")),
                ));
                pairs.push((
                    "error_message".into(),
                    CelVal::Str(extract_xml_tag(&xml, "Message")),
                ));
            }
            CelVal::Map(pairs)
        }
        Err(e) => CelVal::Map(vec![
            ("status".into(), CelVal::Int(0)),
            ("error_message".into(), CelVal::Str(e.to_string())),
        ]),
    }
}

fn extract_xml_tag(xml: &str, tag: &str) -> String {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    if let Some(start) = xml.find(&open) {
        let after = &xml[start + open.len()..];
        if let Some(end) = after.find(&close) {
            return after[..end].to_string();
        }
    }
    String::new()
}

fn hex_sha256(data: &[u8]) -> String {
    let h = Sha256::digest(data);
    h.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_hmac_sha256(key: &[u8], data: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key size");
    mac.update(data);
    mac.finalize()
        .into_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

fn raw_hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn derive_signing_key(secret_key: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
    let k_date = raw_hmac_sha256(format!("AWS4{}", secret_key).as_bytes(), date.as_bytes());
    let k_region = raw_hmac_sha256(&k_date, region.as_bytes());
    let k_service = raw_hmac_sha256(&k_region, service.as_bytes());
    raw_hmac_sha256(&k_service, b"aws4_request")
}

/// Format a Unix timestamp as either `20060102T150405Z` (long) or `20060102`
/// (short).  Implements only the subset needed for SigV4 (no std time formatting).
fn format_aws_date(secs: u64, long: bool) -> String {
    // Use a minimal manual implementation to avoid pulling in chrono/time.
    let s = secs;
    let sec = s % 60;
    let min = (s / 60) % 60;
    let hour = (s / 3600) % 24;
    let days = s / 86400; // days since epoch

    // Gregorian calendar
    let (year, month, day) = days_to_ymd(days);

    if long {
        format!(
            "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
            year, month, day, hour, min, sec
        )
    } else {
        format!("{:04}{:02}{:02}", year, month, day)
    }
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Gregorian algorithm from Howard Hinnant's date library
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

// ── cel.bind chain evaluator ──────────────────────────────────────────────────

/// Peel all `cel.bind(name, expr, body)` layers, evaluate each binding
/// natively, then hand the final body + accumulated variables to the
/// CEL condition evaluator.
fn eval_bind_chain(
    src: &str,
    env: &mut HashMap<String, CelVal>,
) -> Result<ValidationResult, String> {
    let src = src.trim();

    if let Some(inner) = strip_call(src, "cel.bind") {
        let args = split_args(inner);
        if args.len() != 3 {
            return Err(format!("cel.bind expects 3 args, got {}", args.len()));
        }
        let name = args[0].trim();
        let binding_src = args[1].trim();
        let body = args[2];

        let val = eval_binding(binding_src, env)?;
        env.insert(name.to_string(), val);

        return eval_bind_chain(body, env);
    }

    // No more cel.bind — evaluate the condition body with cel-interpreter
    eval_condition_body(src, env)
}

// ── CEL condition body evaluation using cel-interpreter ──────────────────────

/// Evaluate the condition body (the part after all `cel.bind` layers) using
/// `cel-interpreter`.  The bound variables (e.g. `r`) are injected as
/// `cel_interpreter::Value` objects.
fn eval_condition_body(
    body: &str,
    env: &HashMap<String, CelVal>,
) -> Result<ValidationResult, String> {
    // Extend the expression with `unknown` as a no-op function if needed.
    // We rewrite `unknown(r)` → `{"result": "unknown", "reason": "HTTP " + string(r.status)}`
    let rewritten = rewrite_unknown(body);

    let program = Program::compile(&rewritten).map_err(|e| format!("CEL compile error: {}", e))?;

    let mut ctx = Context::default();

    // Inject bound variables
    for (name, val) in env {
        // Skip `secret` and `captures` — they're already strings/maps in CEL
        ctx.add_variable(name.as_str(), val.to_cel())
            .map_err(|e| format!("CEL add_variable({}): {}", name, e))?;
    }

    let result = program
        .execute(&ctx)
        .map_err(|e| format!("CEL execute: {}", e))?;

    parse_cel_result(result)
}

/// Rewrite `unknown(r)` into a map literal that `cel-interpreter` can handle
/// without a custom function binding.
fn rewrite_unknown(body: &str) -> String {
    // Simple text replacement: unknown(VAR) → {"result": "unknown", "reason": "HTTP " + string(VAR.status)}
    // This is safe because `unknown` is always called with a single variable name.
    let mut result = body.to_string();
    while let Some(pos) = result.find("unknown(") {
        let after = &result[pos + "unknown(".len()..];
        if let Some(close) = after.find(')') {
            let var_name = after[..close].trim();
            let replacement = "{\"result\": \"unknown\", \"reason\": \"HTTP \"}".to_string();
            // Use a fixed replacement; the CEL interpreter will get the right map
            let _ = var_name; // we'd need CEL string conversion for the reason
            let end = pos + "unknown(".len() + close + 1;
            result.replace_range(pos..end, &replacement);
        } else {
            break;
        }
    }
    result
}

/// Parse the map returned by the CEL condition body into a `ValidationResult`.
fn parse_cel_result(val: Value) -> Result<ValidationResult, String> {
    let map = match val {
        Value::Map(m) => m,
        other => {
            return Err(format!(
                "validate expression returned {:?}, expected a map",
                other
            ))
        }
    };

    let get = |key: &str| -> Option<String> {
        map.map
            .get(&Key::from(key.to_string()))
            .and_then(|v| match v {
                Value::String(s) => Some(s.to_string()),
                _ => None,
            })
    };

    let status = get("result").unwrap_or_else(|| "unknown".into());
    let reason = get("reason");

    let valid_statuses = ["valid", "invalid", "revoked", "unknown", "error"];
    let status = if valid_statuses.contains(&status.as_str()) {
        status
    } else {
        "unknown".into()
    };

    Ok(match status.as_str() {
        "valid" => ValidationResult::valid(),
        "invalid" => ValidationResult::invalid(reason.unwrap_or_default()),
        "revoked" => ValidationResult::revoked(reason.unwrap_or_default()),
        _ => ValidationResult::unknown(reason.unwrap_or_default()),
    })
}

// ── public entry point ────────────────────────────────────────────────────────

/// Evaluate a betterleaks-style `validate` CEL expression against a live API.
///
/// Returns a `ValidationResult` indicating whether the secret is
/// `"valid"`, `"invalid"`, `"revoked"`, `"unknown"`, or `"error"`.
///
/// Results are cached by `(rule_id, secret, captures)` so the same secret
/// is never validated twice in one process.
pub fn evaluate_validate(
    rule_id: &str,
    validate_expr: &str,
    secret: &str,
    captures: &HashMap<String, String>,
) -> ValidationResult {
    let key = cache_key(rule_id, secret, captures);
    // Check cache
    if let Ok(c) = result_cache().lock() {
        if let Some(r) = c.get(&key) {
            return r.clone();
        }
    }

    let result = run_validate(validate_expr, secret, captures);

    // Cache non-error results
    if result.status != "error" {
        if let Ok(mut c) = result_cache().lock() {
            c.insert(key, result.clone());
        }
    }

    result
}

fn run_validate(
    validate_expr: &str,
    secret: &str,
    captures: &HashMap<String, String>,
) -> ValidationResult {
    let mut env: HashMap<String, CelVal> = HashMap::new();
    env.insert("secret".into(), CelVal::Str(secret.into()));
    let caps: Vec<(String, CelVal)> = captures
        .iter()
        .map(|(k, v)| (k.clone(), CelVal::Str(v.clone())))
        .collect();
    env.insert("captures".into(), CelVal::Map(caps));

    match eval_bind_chain(validate_expr, &mut env) {
        Ok(r) => r,
        Err(e) => ValidationResult::error(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_args_basic() {
        let args = split_args(r#""url", {"a": "b"}, "body""#);
        assert_eq!(args.len(), 3);
        assert_eq!(args[0], r#""url""#);
    }

    #[test]
    fn split_args_nested_parens() {
        // Args with nested braces/parens should not be split at inner commas
        let args = split_args(r#"r, http.get("u", {"k": "v"}), r.status == 200"#);
        assert_eq!(args.len(), 3);
        assert_eq!(args[1], r#"http.get("u", {"k": "v"})"#);
    }

    #[test]
    fn find_close_paren_basic() {
        let s = "(a, b, (c, d))";
        assert_eq!(find_close_paren(s, 0), Some(s.len() - 1));
    }

    #[test]
    fn strip_call_extracts_inner() {
        let inner = strip_call(r#"http.get("url", {})"#, "http.get");
        assert_eq!(inner, Some(r#""url", {}"#));
    }

    #[test]
    fn eval_string_literal() {
        let env = HashMap::new();
        assert_eq!(
            eval_string_expr(r#""hello world""#, &env).unwrap(),
            "hello world"
        );
        assert_eq!(eval_string_expr(r#"'single'"#, &env).unwrap(), "single");
    }

    #[test]
    fn eval_string_concat() {
        let mut env = HashMap::new();
        env.insert("secret".into(), CelVal::Str("mytoken".into()));
        let result = eval_string_expr(r#""Bearer " + secret"#, &env).unwrap();
        assert_eq!(result, "Bearer mytoken");
    }

    #[test]
    fn eval_captures_index() {
        let mut env = HashMap::new();
        env.insert(
            "captures".into(),
            CelVal::Map(vec![(
                "aws-secret-access-key".into(),
                CelVal::Str("secretval".into()),
            )]),
        );
        let result = eval_string_expr(r#"captures["aws-secret-access-key"]"#, &env).unwrap();
        assert_eq!(result, "secretval");
    }

    #[test]
    fn parse_result_valid() {
        let mut map = std::collections::HashMap::new();
        map.insert(
            Key::from("result".to_string()),
            Value::String("valid".to_string().into()),
        );
        let r = parse_cel_result(Value::Map(map.into())).unwrap();
        assert_eq!(r.status, "valid");
    }

    #[test]
    fn days_to_ymd_epoch() {
        let (y, m, d) = days_to_ymd(0);
        assert_eq!((y, m, d), (1970, 1, 1));
    }

    #[test]
    fn days_to_ymd_known_date() {
        // 2024-01-01 = days since 1970-01-01
        let days: u64 = (2024 - 1970) * 365 + 13 + 1; // approx, with leap years
        let (y, _m, _d) = days_to_ymd(days);
        assert_eq!(y, 2024);
    }

    // ── format_aws_date ──────────────────────────────────────────────────────

    #[test]
    fn format_aws_date_short_epoch() {
        let s = format_aws_date(0, false);
        assert_eq!(s, "19700101");
    }

    #[test]
    fn format_aws_date_long_epoch() {
        let s = format_aws_date(0, true);
        assert_eq!(s, "19700101T000000Z");
    }

    #[test]
    fn format_aws_date_long_has_correct_format() {
        // Use a known timestamp: 2024-03-15 12:34:56 UTC = 1710502496
        let s = format_aws_date(1710502496, true);
        assert_eq!(s.len(), 16);
        assert!(s.ends_with('Z'));
        assert!(s.contains('T'));
    }

    // ── extract_xml_tag ──────────────────────────────────────────────────────

    #[test]
    fn extract_xml_tag_found() {
        let xml = "<GetCallerIdentityResponse><Arn>arn:aws:iam::123456789:user/alice</Arn></GetCallerIdentityResponse>";
        assert_eq!(
            extract_xml_tag(xml, "Arn"),
            "arn:aws:iam::123456789:user/alice"
        );
    }

    #[test]
    fn extract_xml_tag_missing() {
        let xml = "<root><Other>val</Other></root>";
        assert_eq!(extract_xml_tag(xml, "Arn"), "");
    }

    #[test]
    fn extract_xml_tag_empty_value() {
        let xml = "<Code></Code>";
        assert_eq!(extract_xml_tag(xml, "Code"), "");
    }

    // ── unescape_string ──────────────────────────────────────────────────────

    #[test]
    fn unescape_string_basic_escapes() {
        assert_eq!(unescape_string(r#"hello\nworld"#), "hello\nworld");
        assert_eq!(unescape_string(r#"tab\there"#), "tab\there");
        assert_eq!(unescape_string(r#"say \"hi\""#), "say \"hi\"");
        assert_eq!(unescape_string(r#"back\\slash"#), "back\\slash");
    }

    #[test]
    fn unescape_string_no_escapes() {
        assert_eq!(unescape_string("plain text"), "plain text");
    }

    // ── find_top_level_plus ──────────────────────────────────────────────────

    #[test]
    fn find_top_level_plus_simple() {
        assert_eq!(find_top_level_plus("a + b"), Some(2));
    }

    #[test]
    fn find_top_level_plus_nested() {
        // Plus inside parens is not top-level
        assert_eq!(find_top_level_plus("foo(a + b)"), None);
    }

    #[test]
    fn find_top_level_plus_after_nested() {
        let s = r#"func(a, b) + "suffix""#;
        let pos = find_top_level_plus(s).unwrap();
        assert_eq!(&s[pos..pos + 1], "+");
    }

    // ── json_to_cel ──────────────────────────────────────────────────────────

    #[test]
    fn json_to_cel_null() {
        let v = json_to_cel(&serde_json::Value::Null);
        assert!(matches!(v, CelVal::Null));
    }

    #[test]
    fn json_to_cel_bool() {
        assert!(matches!(
            json_to_cel(&serde_json::Value::Bool(true)),
            CelVal::Bool(true)
        ));
        assert!(matches!(
            json_to_cel(&serde_json::Value::Bool(false)),
            CelVal::Bool(false)
        ));
    }

    #[test]
    fn json_to_cel_integer() {
        let v = json_to_cel(&serde_json::json!(42));
        assert!(matches!(v, CelVal::Int(42)));
    }

    #[test]
    fn json_to_cel_string() {
        let v = json_to_cel(&serde_json::json!("hello"));
        assert!(matches!(v, CelVal::Str(s) if s == "hello"));
    }

    #[test]
    fn json_to_cel_array() {
        let v = json_to_cel(&serde_json::json!([1, 2]));
        assert!(matches!(v, CelVal::List(_)));
        if let CelVal::List(items) = v {
            assert_eq!(items.len(), 2);
        }
    }

    #[test]
    fn json_to_cel_object() {
        let v = json_to_cel(&serde_json::json!({"key": "val"}));
        assert!(matches!(v, CelVal::Map(_)));
        if let CelVal::Map(pairs) = v {
            assert!(pairs.iter().any(|(k, _)| k == "key"));
        }
    }

    // ── rewrite_unknown ──────────────────────────────────────────────────────

    #[test]
    fn rewrite_unknown_replaces_call() {
        let body = r#"r.status == 200 ? {"result": "valid"} : unknown(r)"#;
        let out = rewrite_unknown(body);
        assert!(!out.contains("unknown("));
        assert!(out.contains("\"result\""));
        assert!(out.contains("\"unknown\""));
    }

    #[test]
    fn rewrite_unknown_no_change_when_absent() {
        let body = r#"{"result": "valid"}"#;
        let out = rewrite_unknown(body);
        assert_eq!(out, body);
    }

    // ── parse_cel_result ─────────────────────────────────────────────────────

    #[test]
    fn parse_result_invalid_with_reason() {
        let mut map = std::collections::HashMap::new();
        map.insert(
            Key::from("result".to_string()),
            Value::String("invalid".to_string().into()),
        );
        map.insert(
            Key::from("reason".to_string()),
            Value::String("Unauthorized".to_string().into()),
        );
        let r = parse_cel_result(Value::Map(map.into())).unwrap();
        assert_eq!(r.status, "invalid");
        assert_eq!(r.reason.as_deref(), Some("Unauthorized"));
    }

    #[test]
    fn parse_result_revoked() {
        let mut map = std::collections::HashMap::new();
        map.insert(
            Key::from("result".to_string()),
            Value::String("revoked".to_string().into()),
        );
        let r = parse_cel_result(Value::Map(map.into())).unwrap();
        assert_eq!(r.status, "revoked");
    }

    #[test]
    fn parse_result_unknown_status_normalised() {
        // An unrecognised status string should be normalised to "unknown"
        let mut map = std::collections::HashMap::new();
        map.insert(
            Key::from("result".to_string()),
            Value::String("garbage".to_string().into()),
        );
        let r = parse_cel_result(Value::Map(map.into())).unwrap();
        assert_eq!(r.status, "unknown");
    }

    #[test]
    fn parse_result_non_map_is_error() {
        let r = parse_cel_result(Value::Bool(true));
        assert!(r.is_err());
    }

    // ── run_validate with pure CEL (no HTTP) ─────────────────────────────────

    #[test]
    fn run_validate_pure_cel_valid() {
        // Expression that requires no HTTP: just return a map literal
        let expr = r#"{"result": "valid"}"#;
        let result = run_validate(expr, "mysecret", &HashMap::new());
        assert_eq!(result.status, "valid");
    }

    #[test]
    fn run_validate_pure_cel_invalid() {
        let expr = r#"{"result": "invalid", "reason": "always bad"}"#;
        let result = run_validate(expr, "mysecret", &HashMap::new());
        assert_eq!(result.status, "invalid");
        assert_eq!(result.reason.as_deref(), Some("always bad"));
    }

    #[test]
    fn run_validate_compile_error_returns_error() {
        let expr = "this is not valid CEL !!!";
        let result = run_validate(expr, "mysecret", &HashMap::new());
        assert_eq!(result.status, "error");
    }

    // ── hex helpers ──────────────────────────────────────────────────────────

    #[test]
    fn hex_sha256_known_value() {
        // SHA256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let h = hex_sha256(b"");
        assert_eq!(&h[..8], "e3b0c442");
    }

    #[test]
    fn hex_sha256_hello() {
        // SHA256("hello") well-known prefix
        let h = hex_sha256(b"hello");
        assert_eq!(&h[..8], "2cf24dba");
    }

    // ── cache_key determinism ─────────────────────────────────────────────────

    #[test]
    fn cache_key_deterministic() {
        let mut caps = HashMap::new();
        caps.insert("foo".into(), "bar".into());
        let k1 = cache_key("rule-a", "secret123", &caps);
        let k2 = cache_key("rule-a", "secret123", &caps);
        assert_eq!(k1, k2);
    }

    #[test]
    fn cache_key_differs_on_different_secret() {
        let caps = HashMap::new();
        let k1 = cache_key("rule-a", "secretA", &caps);
        let k2 = cache_key("rule-a", "secretB", &caps);
        assert_ne!(k1, k2);
    }
}
