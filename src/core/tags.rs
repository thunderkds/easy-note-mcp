//! `#word` tags in note text (FR-006): lower-case `[a-z0-9_/-]+`, each once, in text order.
//!
//! A tag starts at a `#` that begins the text or follows white space, so `a#b` and `##x` hold
//! none. The `#` itself stays in the note text; the tag is only copied to `tags`.

/// The characters of a tag after the `#`.
fn is_tag_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '/' | '-')
}

/// Lower-case a tag a person typed (with or without the leading `#`); `None` if it is not a tag.
pub fn normalize(tag: &str) -> Option<String> {
    let tag = tag.strip_prefix('#').unwrap_or(tag);
    (!tag.is_empty() && tag.chars().all(is_tag_char)).then(|| tag.to_ascii_lowercase())
}

/// Every `#word` in `text`, lower-cased, without repeats.
pub fn extract(text: &str) -> Vec<String> {
    let mut found: Vec<String> = vec![];
    for word in text.split_whitespace() {
        let Some(rest) = word.strip_prefix('#') else {
            continue;
        };
        let name: String = rest.chars().take_while(|c| is_tag_char(*c)).collect();
        if name.is_empty() {
            continue;
        }
        let name = name.to_ascii_lowercase();
        if !found.contains(&name) {
            found.push(name);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_takes_hash_words_lower_cased_once_in_order() {
        assert_eq!(
            extract("retry #Flaky it's! #v2/ui, #flaky a#b ##x # y #"),
            ["flaky", "v2/ui"]
        );
        assert_eq!(extract("#a\n#b\t#a"), ["a", "b"]);
        assert!(extract("no tags here").is_empty());
    }

    #[test]
    fn normalize_accepts_a_tag_with_or_without_hash() {
        assert_eq!(normalize("#Fix-1").as_deref(), Some("fix-1"));
        assert_eq!(normalize("a/b_c").as_deref(), Some("a/b_c"));
        for bad in ["", "#", "a b", "x!", "é"] {
            assert_eq!(normalize(bad), None, "{bad:?}");
        }
    }
}
