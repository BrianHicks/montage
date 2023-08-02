use regex::{Regex, RegexBuilder};
use std::ops::Range;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Word {
    pub word: String,
    pub order: usize,
}

lazy_static::lazy_static! {
    static ref DOUBLE_LETTER_RE: Regex = RegexBuilder::new(
        r"(aa|bb|cc|dd|ee|ff|gg|hh|ii|jj|kk|ll|mm|nn|oo|pp|qq|rr|ss|tt|uu|vv|ww|xx|yy|zz)",
    )
    .case_insensitive(true)
    .build()
    .unwrap();

    static ref INNER_WORD_VOWELS_RE: Regex =
        RegexBuilder::new(r"\b[a-z]+?([aeiouy])[a-z]+\b")
            .case_insensitive(true)
            .build()
            .unwrap();

    static ref INNER_WORD_CONSONANTS_RE: Regex =
        RegexBuilder::new(r"\b[a-z]+?([bcdfghjklmnpqrstvwxz])[a-z]+\b")
            .case_insensitive(true)
            .build()
            .unwrap();

    static ref PUNCTUATION_RE: Regex = Regex::new(r"[\.,!?:]").unwrap();
}

impl Word {
    pub fn new(word: String, order: usize) -> Self {
        Self { word, order }
    }

    pub fn len(&self) -> usize {
        self.word.len()
    }

    pub fn priority(&self) -> usize {
        if self.word.ends_with(' ') {
            self.word.len() - 1
        } else {
            self.word.len()
        }
    }

    pub fn shorten(&mut self) -> usize {
        if let Some(double_match) = DOUBLE_LETTER_RE.find(&self.word) {
            assert!(!double_match.is_empty());
            let replacement: String = double_match.as_str().chars().take(1).collect();
            self.word.replace_range(double_match.range(), &replacement);

            return 1;
        }

        if let Some(vowel) = self.first_inner_word_vowel() {
            self.word.replace_range(vowel, "");

            return 1;
        }

        if let Some(consonant) = self.first_inner_word_consonant() {
            self.word.replace_range(consonant, "");

            return 1;
        }

        if let Some(punc_match) = PUNCTUATION_RE.find(&self.word) {
            self.word.replace_range(punc_match.range(), "");

            return 1;
        }

        if self.priority() <= 3 {
            self.word = self
                .word
                .chars()
                .take(1)
                .map(|c| c.to_ascii_uppercase())
                .collect();
        }

        0
    }

    fn first_inner_word_vowel(&self) -> Option<Range<usize>> {
        INNER_WORD_VOWELS_RE
            .captures(&self.word)
            .and_then(|captures| captures.get(1))
            .map(|vowel| vowel.range())
    }

    fn first_inner_word_consonant(&self) -> Option<Range<usize>> {
        INNER_WORD_CONSONANTS_RE
            .captures(&self.word)
            .and_then(|captures| captures.get(1))
            .map(|vowel| vowel.range())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_word(s: &str) -> Word {
        Word::new(s.to_string(), 0)
    }

    #[test]
    fn score_is_length() {
        assert_eq!(new_word("abcd").priority(), 4);
    }

    #[test]
    fn removes_double_letters() {
        let mut word = new_word("bookkeeper");

        assert_eq!(word.shorten(), 1);
        assert_eq!(word.shorten(), 1);
        assert_eq!(word.shorten(), 1);
        assert_eq!(word.word, "bokeper");
    }

    #[test]
    fn removes_inner_vowels() {
        let mut word = new_word("delicious");

        assert_eq!(word.shorten(), 1);
        assert_eq!(word.word, "dlicious");
    }

    #[test]
    fn removes_punctuation() {
        let mut word = new_word("!?.,:");

        assert_eq!(word.shorten(), 1, "{}", word.word);
        assert_eq!(word.shorten(), 1, "{}", word.word);
        assert_eq!(word.shorten(), 1, "{}", word.word);
        assert_eq!(word.shorten(), 1, "{}", word.word);
        assert_eq!(word.shorten(), 1, "{}", word.word);
        assert_eq!(word.word, "");
    }

    #[test]
    fn removes_inner_consonants() {
        let mut word = new_word("qwerty");

        assert_eq!(word.shorten(), 1); // the vowel
        assert_eq!(word.shorten(), 1);
        assert_eq!(word.shorten(), 1);
        assert_eq!(word.word, "qty");
    }
}
