#[inline]
pub fn trim(input: &str) -> Option<&str> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}
