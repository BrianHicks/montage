pub fn crunch(input: &str, target: usize) -> String {
    input.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doesnt_shorten_string_less_than_target_size() {
        assert_eq!(crunch("foo", 30), "foo");
    }

    #[test]
    fn doesnt_shorten_strings_at_target_size() {
        assert_eq!(crunch("foo", 3), "foo");
    }
}
