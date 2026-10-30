//! Access-code hashing (hlp_password.js): bcrypt hashes, with legacy
//! plaintext codes accepted (and offered an upgrade) until rehashed.

/// bcrypt's rounds.
pub const BCRYPT_ROUNDS: u32 = 10;

/// isHashed: `/^\$2[aby]\$\d{2}\$/`
pub fn is_hashed(p_stored: &str) -> bool {
    let b = p_stored.as_bytes();
    b.len() >= 7
        && &b[..2] == b"$2"
        && b"aby".contains(&b[2])
        && b[3] == b'$'
        && b[4].is_ascii_digit()
        && b[5].is_ascii_digit()
        && b[6] == b'$'
}

/// hash: a bcrypt `$2b$` hash (node bcrypt's format).
pub fn hash(p_plain: &str) -> Option<String> {
    bcrypt::hash(p_plain, BCRYPT_ROUNDS).ok()
}

/// verify: bcrypt for a hash, an exact match for a legacy plaintext code.
pub fn verify(p_plain: &str, p_stored: &str) -> bool {
    if is_hashed(p_stored) {
        return bcrypt::verify(p_plain, p_stored).unwrap_or(false);
    }
    p_plain == p_stored
}

/// verifyAndUpgrade: like verify; a matching plaintext code is handed to
/// p_on_upgrade as a fresh hash.
pub fn verify_and_upgrade(p_plain: &str, p_stored: &str, p_on_upgrade: impl FnOnce(String)) -> bool {
    if is_hashed(p_stored) {
        return bcrypt::verify(p_plain, p_stored).unwrap_or(false);
    }
    if p_plain != p_stored {
        return false;
    }
    match hash(p_plain) {
        Some(h) => p_on_upgrade(h),
        None => eprintln!("[hlp_password] upgrade hash failed"),
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_and_legacy_codes() {
        let h = hash("code1").unwrap();
        assert!(is_hashed(&h) && verify("code1", &h) && !verify("code2", &h));
        assert!(verify("plain", "plain") && !verify("plain", "other"));
        let mut v_new = None;
        assert!(verify_and_upgrade("plain", "plain", |h| v_new = Some(h)));
        assert!(verify("plain", &v_new.unwrap()));
    }
}
