use regex::{Match, Regex, RegexBuilder};
use std::ops::Range;

pub fn crunch(input: &str, target: usize) -> String {
    Cruncher::default().crunch(input, target)
}

struct Cruncher {
    stop_words: Regex,
    double_letters: Regex,
    inner_word_vowels: Regex,
    inner_word_consonants: Regex,
}

impl Cruncher {
    fn crunch(&self, input: &str, target: usize) -> String {
        let mut out = input.to_string();

        if out.len() <= target {
            return out;
        }

        // remove stopwords
        while let Some(stopword_range) = self.first_stopword(&out) {
            out.replace_range(stopword_range, "");

            if out.len() <= target {
                return out;
            }
        }

        // deduplicate double letters
        while let Some(double_match) = self.first_double_letter(&out) {
            assert!(!double_match.is_empty());
            let replacement: String = double_match.as_str().chars().take(1).collect();
            out.replace_range(double_match.range(), &replacement);

            if out.len() <= target {
                return out;
            }
        }

        // remove inner-word vowels
        while let Some(vowel) = self.first_inner_word_vowel(&out) {
            out.replace_range(vowel, "");

            if out.len() <= target {
                return out;
            }
        }

        // remove inner-word consonants
        while let Some(consonant) = self.first_inner_word_consonant(&out) {
            out.replace_range(consonant, "");

            if out.len() <= target {
                return out;
            }
        }

        out
    }

    fn first_stopword(&self, input: &str) -> Option<Range<usize>> {
        self.stop_words.find(input).map(|word| word.range())
    }

    fn first_double_letter<'input>(&self, input: &'input str) -> Option<Match<'input>> {
        self.double_letters.find(input)
    }

    fn first_inner_word_vowel(&self, input: &str) -> Option<Range<usize>> {
        self.inner_word_vowels
            .captures(input)
            .and_then(|captures| captures.get(1))
            .map(|vowel| vowel.range())
    }

    fn first_inner_word_consonant(&self, input: &str) -> Option<Range<usize>> {
        self.inner_word_consonants
            .captures(input)
            .and_then(|captures| captures.get(1))
            .map(|vowel| vowel.range())
    }
}

// List generated by looking at the most common words in my task list! Not representative of all
// English, even a little bit.
static STOP_WORDS: [&str; 25] = [
    "the", "in", "read", "a", "to", "for", "and", "with", "do", "of", "at", "check", "my", "on",
    "what", "up", "i", "how", "look", "is", "get", "this", "about", "could", "by",
];

impl Default for Cruncher {
    fn default() -> Self {
        let pattern = format!(r"\b({})\b\s*", STOP_WORDS.join("|"));

        Cruncher {
            stop_words: RegexBuilder::new(&pattern)
                .case_insensitive(true)
                .build()
                .unwrap(),
            double_letters: RegexBuilder::new(
                r"(aa|bb|cc|dd|ee|ff|gg|hh|ii|jj|kk|ll|mm|nn|oo|pp|qq|rr|ss|tt|uu|vv|ww|xx|yy|zz)",
            )
            .case_insensitive(true)
            .build()
            .unwrap(),
            inner_word_vowels: RegexBuilder::new(r"\b\w+([aeiouy])\w+\b")
                .case_insensitive(true)
                .build()
                .unwrap(),
            inner_word_consonants: RegexBuilder::new(r"\b\w+([^aeiouy])\w+\b")
                .case_insensitive(true)
                .build()
                .unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doesnt_shorten_string_less_than_target_size() {
        let cruncher = Cruncher::default();

        assert_eq!(cruncher.crunch("foo", 30), "foo");
    }

    #[test]
    fn doesnt_shorten_strings_at_target_size() {
        let cruncher = Cruncher::default();

        assert_eq!(cruncher.crunch("foo", 3), "foo");
    }

    #[test]
    fn cannot_find_stop_words_when_there_are_none() {
        let cruncher = Cruncher::default();

        assert_eq!(cruncher.first_stopword("eat five bananas"), None);
    }

    #[test]
    fn finds_first_stopword() {
        let cruncher = Cruncher::default();

        assert_eq!(
            cruncher.first_stopword("the old man and the sea"),
            Some(0..4)
        );
    }

    #[test]
    fn removes_only_as_many_stopwords_as_necessary() {
        let cruncher = Cruncher::default();

        assert_eq!(
            cruncher.crunch("the old man and the sea", 18),
            "old man the sea"
        );
    }

    #[test]
    fn removes_double_letters() {
        let cruncher = Cruncher::default();

        assert_eq!(cruncher.crunch("bookkeepers", 8), "bokepers");
    }

    #[test]
    fn removes_inner_word_vowels() {
        let cruncher = Cruncher::default();

        assert_eq!(cruncher.crunch("band", 3), "bnd");
    }

    #[test]
    fn removes_inner_word_consonants() {
        let cruncher = Cruncher::default();

        assert_eq!(cruncher.crunch("qwerty", 2), "qy");
    }
}
