/// Truncate the string `s` to at most `max_chars` Unicode characters, appending an
/// ellipsis (`…`) only when the string was actually shortened.
pub(crate) fn truncate_with_ellipsis(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let mut output_string: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        output_string.push('…');
    }
    output_string
}
