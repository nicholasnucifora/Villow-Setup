use crate::error::{Error, Result};

// Lex only SQL's outer layer. Quoted strings, identifiers, dollar-quoted
// function bodies and nested comments are opaque. This is not a SQL executor
// or a semicolon splitter. Signed source still requires publisher review.
pub fn validate_transactional(sql: &str) -> Result<()> {
    let b = sql.as_bytes();
    let mut i = 0;
    let mut tokens = Vec::new();
    while i < b.len() {
        if b[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if b[i..].starts_with(b"--") {
            while i < b.len() && b[i] != b'\n' {
                i += 1
            }
            continue;
        }
        if b[i..].starts_with(b"/*") {
            i += 2;
            let mut depth = 1;
            while i < b.len() && depth > 0 {
                if b[i..].starts_with(b"/*") {
                    depth += 1;
                    i += 2
                } else if b[i..].starts_with(b"*/") {
                    depth -= 1;
                    i += 2
                } else {
                    i += 1
                }
            }
            if depth != 0 {
                return Err(Error::Release);
            }
            continue;
        }
        if b[i] == b'\'' || b[i] == b'"' {
            let quote = b[i];
            let start = i;
            let escapes = quote == b'\''
                && i > 0
                && matches!(b[i - 1], b'e' | b'E')
                && (i < 2 || !(b[i - 2].is_ascii_alphanumeric() || b[i - 2] == b'_'));
            i += 1;
            let mut closed = false;
            while i < b.len() {
                if b[i] == quote {
                    if i + 1 < b.len() && b[i + 1] == quote {
                        i += 2
                    } else {
                        i += 1;
                        closed = true;
                        break;
                    }
                } else if escapes && b[i] == b'\\' {
                    i += 2
                } else {
                    i += 1
                }
            }
            if !closed {
                return Err(Error::Release);
            }
            if quote == b'"' && sql[start + 1..i - 1].eq_ignore_ascii_case("villow_setup") {
                return Err(Error::Release);
            }
            continue;
        }
        if b[i] == b'$' {
            let start = i;
            i += 1;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1
            }
            if i < b.len() && b[i] == b'$' {
                i += 1;
                let delimiter = &sql[start..i];
                let rest = &sql[i..];
                if let Some(end) = rest.find(delimiter) {
                    i += end + delimiter.len();
                    continue;
                } else {
                    return Err(Error::Release);
                }
            }
            continue;
        }
        if b[i].is_ascii_alphabetic() || b[i] == b'_' {
            let start = i;
            i += 1;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1
            }
            tokens.push(sql[start..i].to_ascii_uppercase());
        } else {
            i += 1
        }
    }
    // Conservatively deny transaction escapes and autocommit-only commands,
    // even if a keyword might merely be an unquoted identifier.
    let mut cases = 0usize;
    for token in &tokens {
        if token == "CASE" {
            cases += 1;
        }
        if token == "END" {
            if cases == 0 {
                return Err(Error::Release);
            }
            cases -= 1;
        }
        if [
            "BEGIN",
            "COMMIT",
            "ROLLBACK",
            "ABORT",
            "SAVEPOINT",
            "RELEASE",
            "VACUUM",
            "CONCURRENTLY",
            "VILLOW_SETUP",
            "STANDARD_CONFORMING_STRINGS",
        ]
        .contains(&token.as_str())
        {
            return Err(Error::Release);
        }
    }
    if cases != 0 {
        return Err(Error::Release);
    }
    for pair in tokens.windows(2) {
        if matches!(
            (pair[0].as_str(), pair[1].as_str()),
            ("START", "TRANSACTION")
                | ("PREPARE", "TRANSACTION")
                | ("CREATE", "DATABASE")
                | ("DROP", "DATABASE")
                | ("ALTER", "SYSTEM")
        ) {
            return Err(Error::Release);
        }
    }
    Ok(())
}
