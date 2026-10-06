use rand::RngExt;

pub const BASE62_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
pub const CODE_LEN: usize = 7;

pub fn gen_code() -> String {
    let mut rng = rand::rng();

    (0..CODE_LEN)
        .map(|_| {
            let idx = rng.random_range(0..BASE62_CHARS.len());
            BASE62_CHARS[idx] as char
        })
        .collect()
}

pub fn is_unique_violation(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .and_then(|d| d.code())
        .is_some_and(|code| code == "2067")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_codes_have_expected_shape() {
        for _ in 0..100 {
            let code = gen_code();
            assert_eq!(code.len(), CODE_LEN);
            assert!(code.bytes().all(|b| BASE62_CHARS.contains(&b)));
        }
    }
}
