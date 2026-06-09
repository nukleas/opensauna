//! Number formatting helpers — locale-style thousands separators for the
//! larger figures the app shows (calories, session counts).

/// Group the digits of an integer string with comma thousands separators.
/// Strips any existing commas first, preserves a leading `-`, and drops a
/// trivial `.0`/`.00` fraction while keeping a meaningful one. Non-numeric
/// input is returned unchanged.
///
/// `"1234"` → `"1,234"`, `"1234.0"` → `"1,234"`, `"-25000"` → `"-25,000"`,
/// `"HOT BLAST"` → `"HOT BLAST"`.
pub fn with_commas(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() {
        return String::new();
    }
    let cleaned = s.replace(',', "");
    let (int_part, frac_part) = match cleaned.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (cleaned.as_str(), None),
    };
    let (sign, digits) = int_part
        .strip_prefix('-')
        .map(|d| ("-", d))
        .unwrap_or(("", int_part));
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return s.to_string();
    }

    let bytes = digits.as_bytes();
    let len = bytes.len();
    let mut grouped = String::with_capacity(len + len / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(*b as char);
    }

    match frac_part {
        Some(f) if !f.is_empty() && !f.bytes().all(|b| b == b'0') => {
            format!("{sign}{grouped}.{f}")
        }
        _ => format!("{sign}{grouped}"),
    }
}

/// Like [`with_commas`] but keeps a trailing unit suffix, e.g.
/// `"1234 Cal"` → `"1,234 Cal"`. Strings that don't start with a digit are
/// returned unchanged.
pub fn with_commas_keep_suffix(s: &str) -> String {
    let s = s.trim();
    match s.find(|c: char| !c.is_ascii_digit() && c != ',' && c != '.') {
        Some(0) => s.to_string(),
        Some(idx) => {
            let (num, rest) = s.split_at(idx);
            format!("{}{}", with_commas(num), rest)
        }
        None => with_commas(s),
    }
}

/// Comma-format an integer value.
pub fn commas_i64(n: i64) -> String {
    with_commas(&n.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_thousands() {
        assert_eq!(with_commas("1234"), "1,234");
        assert_eq!(with_commas("1234567"), "1,234,567");
        assert_eq!(with_commas("999"), "999");
        assert_eq!(with_commas("0"), "0");
        assert_eq!(with_commas("1234.0"), "1,234");
        assert_eq!(with_commas("1234.50"), "1,234.50");
        assert_eq!(with_commas("-25000"), "-25,000");
        assert_eq!(with_commas("12,345"), "12,345");
        assert_eq!(with_commas("HOT BLAST"), "HOT BLAST");
    }

    #[test]
    fn keeps_suffix() {
        assert_eq!(with_commas_keep_suffix("1234 Cal"), "1,234 Cal");
        assert_eq!(with_commas_keep_suffix("192 Cal"), "192 Cal");
        assert_eq!(with_commas_keep_suffix("Cal"), "Cal");
    }
}
