//! `$$HASH$$('plaintext')` directives in a configuration
//! (js_config_handler.js): each top-level string value holding one becomes
//! a bcrypt hash in memory, and the directives in the config file are
//! replaced by hashes so the plaintext is gone after the first run.

use regex::Regex;
use serde_json::Value;
use std::path::Path;
use std::sync::OnceLock;

use crate::colors as c;

/// bcryptjs's rounds.
pub const BCRYPT_SALT_ROUNDS: u32 = 10;

fn memory_pattern() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r#"\$\$HASH\$\$\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap())
}

fn file_pattern() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r#"\$\$HASH\$\$\(\s*(?:'([^']+)'|\\?"([^"\\]+)\\?")\s*\)"#).unwrap())
}

/// bcryptjs.hashSync(p, 10): a `$2a$10$` hash.
pub fn bcrypt_hash(p_plain: &str) -> Option<String> {
    bcrypt::hash_with_result(p_plain, BCRYPT_SALT_ROUNDS).ok().map(|h| h.format_for_version(bcrypt::Version::TwoA))
}

/// extractHashDirective: the plaintext of a `$$HASH$$('...')` value.
pub fn extract_hash_directive(p_value: &Value) -> Option<String> {
    let Value::String(t) = p_value else { return None };
    memory_pattern().captures(t).map(|m| m[1].to_string())
}

/// isBcryptHash: /^\$2[abxy]\$\d{2}\$/
pub fn is_bcrypt_hash(p_value: Option<&Value>) -> bool {
    let Some(Value::String(t)) = p_value else { return false };
    let b = t.as_bytes();
    b.len() >= 7
        && &b[..2] == b"$2"
        && b"abxy".contains(&b[2])
        && b[3] == b'$'
        && b[4].is_ascii_digit()
        && b[5].is_ascii_digit()
        && b[6] == b'$'
}

/// processConfigInMemory: true when a value changed.
pub fn process_in_memory(p_config: &mut Value) -> bool {
    let Value::Object(m) = p_config else { return false };
    let mut v_changed = false;
    for (k, v) in m.iter_mut() {
        let Some(v_plain) = extract_hash_directive(v) else { continue };
        let Some(v_hash) = bcrypt_hash(&v_plain) else { continue };
        *v = Value::String(v_hash);
        v_changed = true;
        println!("{}[ConfigHandler] Hashed sensitive config value: {}{}", c::LOG, k, c::RESET);
    }
    v_changed
}

/// persistHashes: replaces the directives in the file; true when it changed.
pub fn persist_hashes(p_file: &Path) -> bool {
    let v_raw = match std::fs::read_to_string(p_file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}[ConfigHandler] Cannot read config file for persistence: {}{}", c::ERROR, e, c::RESET);
            return false;
        }
    };
    let mut v_changed = false;
    let v_updated = file_pattern().replace_all(&v_raw, |caps: &regex::Captures| {
        let v_plain = caps.get(1).or(caps.get(2)).map(|m| m.as_str()).unwrap_or("");
        if v_plain.is_empty() {
            return caps[0].to_string();
        }
        v_changed = true;
        bcrypt_hash(v_plain).unwrap_or_else(|| caps[0].to_string())
    });
    if !v_changed {
        return false;
    }
    match std::fs::write(p_file, v_updated.as_bytes()) {
        Ok(()) => println!("{}[ConfigHandler] Config file updated — hashed values persisted.{}", c::SUCCESS, c::RESET),
        Err(e) => eprintln!("{}[ConfigHandler] Failed to write config file: {}{}", c::ERROR, e, c::RESET),
    }
    true
}

/// handleConfig: in memory, then the file when something changed.
pub fn handle_config(p_config: &mut Value, p_file: &Path) {
    if process_in_memory(p_config) {
        persist_hashes(p_file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn hashes_directives_in_memory_and_file() {
        let v_dir = tempfile::tempdir().unwrap();
        let v_file = v_dir.path().join("s.config");
        let v_text = "{\n  // admin\n  \"admin_password\": \"$$HASH$$('Secret123!')\",\n  \"other\": \"$$HASH$$(\\\"x y\\\")\"\n}\n";
        std::fs::write(&v_file, v_text).unwrap();
        let mut v_cfg = json!({"admin_password": "$$HASH$$('Secret123!')", "other": "$$HASH$$(\"x y\")", "n": 1});
        handle_config(&mut v_cfg, &v_file);
        let v_hash = v_cfg["admin_password"].as_str().unwrap();
        assert!(v_hash.starts_with("$2a$10$") && bcrypt::verify("Secret123!", v_hash).unwrap());
        assert!(bcrypt::verify("x y", v_cfg["other"].as_str().unwrap()).unwrap());
        let v_after = std::fs::read_to_string(&v_file).unwrap();
        assert!(v_after.contains("// admin"), "comments survive");
        assert!(!v_after.contains("$$HASH$$"));
        let v_parsed: Value = serde_json::from_str(&crate::config::strip_json_comments(&v_after)).unwrap();
        assert!(bcrypt::verify("Secret123!", v_parsed["admin_password"].as_str().unwrap()).unwrap());
        assert!(is_bcrypt_hash(Some(&json!(v_hash))) && !is_bcrypt_hash(Some(&json!("plain"))));
    }
}
