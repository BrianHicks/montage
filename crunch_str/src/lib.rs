mod word;

use priority_queue::PriorityQueue;

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
/// assert_eq!(crunch("bookkeeper", 4), "bkpr");
/// assert_eq!(crunch("how are your metrics?", 15), "how r ur mtrcs?");
/// assert_eq!(crunch("'Twas brillig, and the slithy toves", 23), "'Ts blg, & the sthy tvs");
/// assert_eq!(crunch("a very long string with a lot of words", 9), "AVLSWALOW");
/// ```
pub fn crunch(input: &str, target: usize) -> String {
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
        .map(|word| word.to_string())
        .collect::<Vec<String>>()
        .join("")
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

    #[test]
    fn cruncher_crunches_longest_word_first() {
        assert_eq!(crunch("a be creative", 9), "a b crtve");
    }

    #[test]
    fn removes_double_letters() {
        assert_eq!(crunch("bookkeepers", 8), "bokepers");
    }

    #[test]
    fn removes_inner_word_vowels() {
        assert_eq!(crunch("band", 3), "bnd");
    }

    #[test]
    fn removes_inner_word_vowels_multiple_times() {
        assert_eq!(crunch("band bundt bound", 11), "bnd bdt bnd");
    }

    #[test]
    fn removes_inner_word_consonants() {
        assert_eq!(crunch("qwerty", 2), "qy");
    }

    #[test]
    fn makes_acronym() {
        assert_eq!(
            crunch(
                "out of the night that covers me, black as the pit from pole to pole",
                15,
            ),
            "OOTNTCMBATPFP2P"
        );
    }
}
