pub fn contains_pattern_escaped(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len() + 2);
    escaped.push('%');
    for ch in input.chars() {
        match ch {
            '\\' | '%' | '_' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    escaped.push('%');
    escaped
}

#[cfg(test)]
mod tests {
    use super::contains_pattern_escaped;

    #[test]
    fn escapes_like_wildcards() {
        assert_eq!(contains_pattern_escaped(r"a%b_c\d"), r"%a\%b\_c\\d%");
    }
}
