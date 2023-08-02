mod word;

use priority_queue::PriorityQueue;
use regex::{Match, Regex, RegexBuilder};
use std::ops::Range;

/// "crunch" a string. That is: make it as short as it can until it reaches the target length.
///
/// This uses an absolutely buckwild string shortening algorithm that tries to take away things
/// that don't mean as much first, growing more and more incoherent the more it has to compress. In
/// other words, it compresses strings like you'd name startups. It works well on strings you
/// already know; maybe not so well on things you're seeing for the first time.
///
/// It will try to:
///
/// - Shorten short/common words to shorter or more casual equivalents (e.g. replacing "one" with
///   "1" or "you" with "u".)
/// - Remove double letters from words
/// - Remove inner vowels from words
/// - Remove inner consonants from words
///
/// It does this longest-word-first, reasoning that a longer word is more likely to be readable
/// with a single character missing than a shorter word.
///
/// If all that fails, it makes one last-ditch attempt to get the string below the target size by
/// converting it to just the initials in the words.
///
/// Some fun examples:
///
/// ```rust
/// use crunch_str::crunch;
///
/// // assert_eq!(crunch("bookkeeper", 4), "bkpr");
/// // assert_eq!(crunch("how are your metrics?", 13), "ae yr mts?");
/// // assert_eq!(crunch("'Twas brillig, and the slithy toves", 20), "'Tws brlg, slthy tvs");
/// // assert_eq!(crunch("a very long string with a lot of words", 5), "VLSLW");
/// ```
pub fn crunch(input: &str, target: usize) -> String {
    Cruncher::default().crunch_words(input, target)
}

struct Cruncher {
    stop_words: Regex,
    double_letters: Regex,
    inner_word_vowels: Regex,
    inner_word_consonants: Regex,
}

impl Cruncher {
    fn crunch_words(&self, input: &str, target: usize) -> String {
        if input.len() <= target {
            return input.to_string();
        }

        let mut words = PriorityQueue::new();
        let mut total_size = input.len();

        input
            .split_inclusive(char::is_whitespace)
            .enumerate()
            .map(|(order, word)| word::Word::new(word.to_string(), order))
            .for_each(|word| {
                let priority = word.priority();
                words.push(word, priority);
            });

        let mut finished_words = Vec::with_capacity(words.len());

        while total_size > target {
            match words.pop() {
                Some((mut word, _)) => {
                    let removed = word.shorten();
                    if removed > 0 {
                        total_size -= removed;

                        let new_priority = word.priority();
                        words.push(word, new_priority);
                    } else if word.len() > 0 {
                        finished_words.push(word);
                    }
                }
                None => break,
            }
        }

        finished_words.append(&mut words.into_vec());
        finished_words.sort_by_key(|word| word.order);

        finished_words
            .drain(..)
            .map(|word| word.word)
            .collect::<Vec<String>>()
            .join("")
    }

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

        // last-ditch effort: make an acronym
        let mut initials = String::with_capacity(out.len() / 2);
        for word in out.split(char::is_whitespace) {
            if let Some(chr) = word.chars().next() {
                initials.push(chr.to_ascii_uppercase())
            }
        }

        initials
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
            inner_word_vowels: RegexBuilder::new(r"\b[a-z]+([aeiouy])[a-z]+\b")
                .case_insensitive(true)
                .build()
                .unwrap(),
            inner_word_consonants: RegexBuilder::new(r"\b[a-z]+([bcdfghjklmnpqrstvwxz])[a-z]+\b")
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

        assert_eq!(cruncher.crunch_words("foo", 30), "foo");
    }

    #[test]
    fn doesnt_shorten_strings_at_target_size() {
        let cruncher = Cruncher::default();

        assert_eq!(cruncher.crunch_words("foo", 3), "foo");
    }

    #[test]
    fn cruncher_crunches_longest_word_first() {
        let cruncher = Cruncher::default();

        assert_eq!(cruncher.crunch_words("a be creative", 10), "a be crtve");
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
    fn removes_inner_word_vowels_multiple_times() {
        let cruncher = Cruncher::default();

        assert_eq!(cruncher.crunch("band bundt bound", 12), "bnd bndt bnd");
    }

    #[test]
    fn removes_inner_word_consonants() {
        let cruncher = Cruncher::default();

        assert_eq!(cruncher.crunch("qwerty", 2), "qy");
    }

    #[test]
    fn makes_acronym() {
        let cruncher = Cruncher::default();

        assert_eq!(
            cruncher.crunch(
                "out of the night that covers me, black as the pit from pole to pole",
                11,
            ),
            "ONTCMBAPFPP"
        );
    }

    // #[test]
    // fn excellent() {
    //     for i in 0..40 {
    //         println!("{i}: {}", crunch("hello and good morning, Team Raven", i));
    //     }
    //
    //     assert!(false);
    // }
}
