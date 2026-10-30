//! String helpers (hlp_strings.js).

use rand::Rng;

/// generateRandomString(n): alphanumeric characters.
pub fn generate_random_string(p_len: usize) -> String {
    const CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut r = rand::thread_rng();
    (0..p_len).map(|_| CHARS[r.gen_range(0..CHARS.len())] as char).collect()
}

/// pad2: a leading zero below 10.
pub fn pad2(p_number: i64) -> String {
    format!("{}{}", if p_number < 10 { "0" } else { "" }, p_number)
}

/// fn_isValidAccountName: alphanumeric or an email.
pub fn is_valid_account_name(p_text: &str) -> bool {
    crate::validation::is_alphanumeric_str(p_text) || crate::validation::is_email_str(p_text)
}

/// fn_getCSSTraffic (js_styleHelper.js): a Bootstrap text class for a byte count.
pub fn css_traffic(p_bytes: f64) -> &'static str {
    if p_bytes <= 1000.0 {
        "text-danger"
    } else if p_bytes <= 10000.0 {
        "text-warning"
    } else if p_bytes <= 100000.0 {
        "text-primary"
    } else if p_bytes <= 1000000.0 {
        "text-success"
    } else {
        "text-disabled"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers() {
        let s = generate_random_string(12);
        assert_eq!(s.len(), 12);
        assert!(s.bytes().all(|b| b.is_ascii_alphanumeric()));
        assert_eq!((pad2(5), pad2(12)), ("05".to_string(), "12".to_string()));
        assert!(is_valid_account_name("a@b.io") && !is_valid_account_name("a b"));
        assert_eq!((css_traffic(5.0), css_traffic(2e6)), ("text-danger", "text-disabled"));
    }
}
