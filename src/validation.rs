//! Validation helpers (hlp_validation.js).
//!
//! Inputs are JSON values, as the JavaScript helpers received them: a
//! function that called a string method on its argument fails (Err, the
//! TypeError) for a value that has no such method.

use serde_json::Value;

/// `/^[_a-z0-9]+$/i`; Err for a non-string (str.match is not a function).
pub fn is_alphanumeric(p_value: Option<&Value>) -> Result<bool, String> {
    match p_value {
        None | Some(Value::Null) => Ok(false),
        Some(Value::String(t)) => Ok(!t.is_empty() && t.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')),
        Some(_) => Err("str.match is not a function".into()),
    }
}

/// isAlphanumeric for a string.
pub fn is_alphanumeric_str(p_text: &str) -> bool {
    !p_text.is_empty() && p_text.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// `/([a-z0-9]+\.)([a-z0-9]+\.)([a-z0-9])+$/i` (e.g. 5a.0.0).
pub fn is_version_format(p_text: &str) -> bool {
    let v = regex::Regex::new(r"(?i)([a-z0-9]+\.)([a-z0-9]+\.)([a-z0-9])+$").unwrap();
    v.is_match(p_text)
}

/// `/^[A-Za-z0-9( )]+$/i`: letters, digits and spaces (`( )` is a group
/// around the space, not literal parentheses).
pub fn is_alphanumeric_sentence(p_text: &str) -> bool {
    !p_text.is_empty() && p_text.chars().all(|c| c.is_ascii_alphanumeric() || c == ' ')
}

fn is_email_part(p_part: &str) -> bool {
    if p_part.is_empty() {
        return false;
    }
    let mut v_prev_sep = true;
    for u in p_part.encode_utf16() {
        let v_word = (48..=57).contains(&u) || (65..=90).contains(&u) || (97..=122).contains(&u) || u == 95;
        let v_sep = u == 46 || u == 45;
        if !v_word && !v_sep {
            return false;
        }
        if v_sep && v_prev_sep {
            return false;
        }
        v_prev_sep = v_sep;
    }
    !v_prev_sep
}

/// isEmail: length cap, one '@', word characters separated by single '.' or
/// '-', a '.' in the domain (linear scan).
pub fn is_email(p_value: Option<&Value>) -> bool {
    let Some(Value::String(t)) = p_value else { return false };
    is_email_str(t)
}

pub fn is_email_str(t: &str) -> bool {
    let v_len = t.encode_utf16().count();
    if v_len == 0 || v_len > 254 {
        return false;
    }
    let (Some(i), Some(j)) = (t.find('@'), t.rfind('@')) else { return false };
    if i == 0 || i != j || i == t.len() - 1 {
        return false;
    }
    let v_domain = &t[i + 1..];
    v_domain.contains('.') && is_email_part(&t[..i]) && is_email_part(v_domain)
}

/// `.length` of a JSON value (undefined for numbers, booleans, objects
/// without a length).
fn length(p_value: &Value) -> Option<usize> {
    match p_value {
        Value::String(t) => Some(t.encode_utf16().count()),
        Value::Array(a) => Some(a.len()),
        Value::Object(m) => m.get("length").and_then(Value::as_f64).map(|f| f as usize),
        _ => None,
    }
}

/// isValidAdminUsername: 3-50 characters, alphanumeric or an email.
pub fn is_valid_admin_username(p_value: Option<&Value>) -> Result<bool, String> {
    let Some(v) = p_value.filter(|v| !v.is_null()) else { return Ok(false) };
    if let Some(l) = length(v) {
        if !(3..=50).contains(&l) {
            return Ok(false);
        }
    }
    Ok(is_alphanumeric(Some(v))? || is_email(Some(v)))
}

/// isValidAdminPassword: 8-128 characters, no whitespace.
pub fn is_valid_admin_password(p_value: Option<&Value>) -> bool {
    let Some(v) = p_value.filter(|v| !v.is_null()) else { return false };
    if let Some(l) = length(v) {
        if !(8..=128).contains(&l) {
            return false;
        }
    }
    // !/\s/.test(String(password))
    !crate::util::js_to_string(v).chars().any(|c| c.is_whitespace() || c == '\u{feff}')
}

/// protectedFromInjection: SQL comment markers become a space.
pub fn protected_from_injection(p_text: &str) -> String {
    p_text.replace("--", " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn like_hlp_validation() {
        assert_eq!(is_valid_admin_username(Some(&json!("admin"))), Ok(true));
        assert_eq!(is_valid_admin_username(Some(&json!("a.b@c.io"))), Ok(true));
        assert_eq!(is_valid_admin_username(Some(&json!("ab"))), Ok(false));
        assert_eq!(is_valid_admin_username(Some(&json!("a!b"))), Ok(false));
        assert!(is_valid_admin_username(Some(&json!(12345))).is_err());
        assert!(is_valid_admin_password(Some(&json!("Secret123!"))));
        assert!(!is_valid_admin_password(Some(&json!("has space 123"))));
        assert!(!is_email_str("a..b@c.io") && !is_email_str("a@b") && !is_email_str("@b.io"));
        assert!(is_version_format("5a.0.0") && !is_version_format("5.0"));
        assert!(is_alphanumeric_sentence("This is a 5 Sample") && !is_alphanumeric_sentence("x!"));
        assert!(!is_alphanumeric_sentence("(x)"), "parentheses are not allowed");
        assert_eq!(protected_from_injection("a--b"), "a b");
    }
}
