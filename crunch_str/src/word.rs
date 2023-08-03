use regex::{Regex, RegexBuilder};
use std::collections::HashMap;
use std::ops::Range;

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

    static ref TRAILING_VOWELS_RE: Regex =
        RegexBuilder::new(r"\b[a-z]+?([aeiou])\b")
            .case_insensitive(true)
            .build()
            .unwrap();

    static ref INNER_WORD_CONSONANTS_RE: Regex =
        RegexBuilder::new(r"\b[a-z]+?([bcdfghjklmnpqrstvwxz])[a-z]+\b")
            .case_insensitive(true)
            .build()
            .unwrap();

    static ref PUNCTUATION_RE: Regex = Regex::new(r"[\.,!?:]").unwrap();

    static ref SUBSTITUTIONS: HashMap<String, &'static str> = HashMap::from([
        // numbers
        ("one".to_string(), "1"),
        ("two".to_string(), "2"),
        ("three".to_string(), "3"),
        ("four".to_string(), "4"),
        ("five".to_string(), "5"),
        ("six".to_string(), "6"),
        ("seven".to_string(), "7"),
        ("eight".to_string(), "8"),
        ("nine".to_string(), "9"),
        ("ten".to_string(), "10"),

        // text speak
        ("and".to_string(), "&"),
        ("are".to_string(), "r"),
        ("be".to_string(), "b"),
        ("for".to_string(), "4"),
        ("our".to_string(), "r"),
        ("to".to_string(), "2"),
        ("why".to_string(), "y"),
        ("you".to_string(), "u"),
        ("your".to_string(), "ur"),

        // misc
        ("make".to_string(), "mk"),
    ]);
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Word {
    word: String,
    trailing_whitespace: usize,
    pub order: usize,
}

impl Word {
    pub fn new(mut word: String, order: usize) -> Self {
        let mut trailing_whitespace = 0;
        while word.ends_with(char::is_whitespace) {
            trailing_whitespace += 1;
            word.pop();
        }

        Self {
            word,
            trailing_whitespace,
            order,
        }
    }

    pub fn len(&self) -> usize {
        self.word.len()
    }

    pub fn priority(&self) -> usize {
        let base = self.len();

        if self.can_substitute() {
            base + 10
        } else {
            base
        }
    }

    fn can_substitute(&self) -> bool {
        SUBSTITUTIONS.contains_key(&self.word)
    }

    pub fn shorten(&mut self) -> usize {
        if let Some(substitution) = SUBSTITUTIONS.get(&self.word) {
            let saved = self.word.len() - substitution.len();

            self.word = substitution.to_string();

            return saved;
        }

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

        if let Some(vowel) = self.trailing_vowel() {
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

        if self.len() <= 2 {
            self.word = self
                .word
                .chars()
                .take(1)
                .map(|c| c.to_ascii_uppercase())
                .collect();
            self.trailing_whitespace = 0;
        }

        0
    }

    fn first_inner_word_vowel(&self) -> Option<Range<usize>> {
        INNER_WORD_VOWELS_RE
            .captures(&self.word)
            .and_then(|captures| captures.get(1))
            .map(|vowel| vowel.range())
    }

    fn trailing_vowel(&self) -> Option<Range<usize>> {
        TRAILING_VOWELS_RE
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

    pub fn to_string(&self) -> String {
        let mut out = String::with_capacity(self.word.len() + self.trailing_whitespace);

        out.push_str(&self.word);
        for _ in 0..self.trailing_whitespace {
            out.push(' ');
        }

        out
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
    fn removes_trailing_vowels() {
        let mut word = new_word("the");

        assert_eq!(word.shorten(), 1);
        assert_eq!(word.word, "th");
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

    #[test]
    fn substitutes_shorter_meanings() {
        SUBSTITUTIONS.iter().for_each(|(key, sub)| {
            let mut word = new_word(key);

            assert_eq!(word.shorten(), key.len() - sub.len());
            assert_eq!(word.word, sub.to_string());
        });
    }
}
