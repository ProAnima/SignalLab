//! Reading the interface's dictionaries (`src/lib/locales/<lang>.ts`) without
//! TypeScript. `build.rs` embeds what this returns; the tests feed it the real
//! files and odd ones. Each entry is one of
//!
//! ```text
//!   "key": "text",
//!   "key":
//!     "text",
//! ```
//!
//! with the text a JSON string. Anything else inside the object fails the
//! build, so a new way of writing an entry cannot be skipped silently.

/// The entries of a dictionary file, in file order.
pub fn entries(source: &str) -> Result<Vec<(String, String)>, String> {
    // A checkout with CRLF line ends (Windows) reads the same.
    let source = source.replace("\r\n", "\n");
    let mut lines = source.lines().enumerate();
    // The object starts at `… = {` (`export const en = {`, `export const ru: Dict = {`).
    if !lines.by_ref().any(|(_, line)| line.trim_end().ends_with("= {")) {
        return Err("no `export const … = {` in the dictionary".into());
    }
    let mut found = Vec::new();
    while let Some((index, line)) = lines.next() {
        let number = index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if line.starts_with('}') {
            return Ok(found);
        }
        let Some(rest) = line.strip_prefix("  \"") else {
            return Err(format!("line {number}: not an entry: {line}"));
        };
        let quoted = format!("\"{rest}");
        let (key, after) = string_literal(&quoted).ok_or_else(|| format!("line {number}: unreadable key"))?;
        let after = after.strip_prefix(':').ok_or_else(|| format!("line {number}: no `:` after the key {key}"))?.trim();
        let text = if after.is_empty() {
            // The text is on the next line.
            let (_, next) = lines.next().ok_or_else(|| format!("line {number}: {key} has no text"))?;
            next.trim().to_string()
        } else {
            after.to_string()
        };
        let (value, tail) = string_literal(&text).ok_or_else(|| format!("line {number}: the text of {key} is not one string"))?;
        if !matches!(tail.trim(), "," | "") {
            return Err(format!("line {number}: {key} is followed by `{}`", tail.trim()));
        }
        found.push((key, value));
    }
    Err("the dictionary object is not closed".into())
}

/// The JSON string literal at the start of `text`, decoded, and what follows it.
fn string_literal(text: &str) -> Option<(String, &str)> {
    if !text.starts_with('"') {
        return None;
    }
    let mut escaped = false;
    for (at, char) in text.char_indices().skip(1) {
        match char {
            _ if escaped => escaped = false,
            '\\' => escaped = true,
            '"' => {
                let value: String = serde_json::from_str(&text[..=at]).ok()?;
                return Some((value, &text[at + 1..]));
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_ways_of_writing_an_entry_and_json_escapes_are_read() {
        let source = "/** head */\nexport const en = {\n  \"a.b\": \"plain\",\n  \"c\":\n    \"on the \\\"next\\\" line — {n, plural, one {#} other {#}}\",\n  // a comment\n\n  \"d\": \"\\u00a0tab\\tend\\\\\",\n} as const;\n\nexport type X = 1;\n";
        assert_eq!(
            entries(source).unwrap(),
            [
                ("a.b".to_string(), "plain".to_string()),
                ("c".to_string(), "on the \"next\" line — {n, plural, one {#} other {#}}".to_string()),
                ("d".to_string(), "\u{a0}tab\tend\\".to_string()),
            ]
        );
    }

    #[test]
    fn crlf_line_ends_read_the_same() {
        let source = "export const ru: Dict = {\r\n  \"a\": \"один\",\r\n  \"b\":\r\n    \"два\",\r\n};\r\n";
        assert_eq!(entries(source).unwrap(), [("a".to_string(), "один".to_string()), ("b".to_string(), "два".to_string())]);
    }

    #[test]
    fn anything_unexpected_is_an_error_not_a_skipped_text() {
        for broken in [
            "const x = 1;\n",
            "export const en = {\n  \"a\": \"x\" + \"y\",\n};\n",
            "export const en = {\n  a: \"x\",\n};\n",
            "export const en = {\n  \"a\": `template`,\n};\n",
            "export const en = {\n  \"a\": \"unclosed,\n};\n",
            "export const en = {\n  \"a\": \"x\",\n",
        ] {
            assert!(entries(broken).is_err(), "{broken}");
        }
    }
}
