//! Correction: replacing recognized words that are a close match for a
//! Custom word with that Custom word.
//!
//! Matching compares alphanumeric, lowercased keys with Levenshtein distance,
//! boosted when Double Metaphone says the two sound alike (so "Kaitlin" finds
//! "Caitlyn" even though the first letters differ). Runs of words are tried
//! too, so "Chat G P T" can become "ChatGPT" when the list says so. The
//! Engine splits words it does not know ("Bernal" comes back as "burn
//! all"), so a run may be one word longer than its entry, if it sounds
//! the same.

use rphonetic::{DoubleMetaphone, Encoder};

/// Scores below this are accepted. 0 is an exact match. Without phonetic
/// agreement this allows one edit in seven letters; "career" stays "career"
/// next to "Carver".
const ACCEPT_BELOW: f64 = 0.15;
/// Levenshtein score multiplier when the phonetic codes agree.
const PHONETIC_BOOST: f64 = 0.3;
/// Keys shorter than this must match exactly; "cat" must not become "Kat".
const MIN_FUZZY_LEN: usize = 4;
/// How many more recognized words than its own an entry may cover.
const SPLIT_ALLOWANCE: usize = 1;

#[derive(Clone, Debug)]
struct Entry {
    text: String,
    key: String,
    word_count: usize,
    primary: String,
    alternate: String,
}

/// The user's Custom words, prepared for matching.
#[derive(Clone, Debug)]
pub struct CustomWords {
    entries: Vec<Entry>,
    max_words: usize,
    metaphone: DoubleMetaphone,
}

impl Default for CustomWords {
    fn default() -> Self {
        Self::new(Vec::<String>::new())
    }
}

/// A Correction to apply at the cursor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Correction {
    /// How many recognized words the replacement covers.
    pub consumed: usize,
    /// The replacement, with the recognized words' edge punctuation kept.
    pub text: String,
}

/// A recognized word split into the part we match and the punctuation around it.
#[derive(Debug, PartialEq, Eq)]
struct Token<'a> {
    leading: &'a str,
    core: &'a str,
    trailing: &'a str,
}

fn tokenize(word: &str) -> Token<'_> {
    let start = word
        .find(|c: char| c.is_alphanumeric())
        .unwrap_or(word.len());
    let end = word
        .rfind(|c: char| c.is_alphanumeric())
        .map_or(start, |i| i + word[i..].chars().next().unwrap().len_utf8());
    let (leading, rest) = word.split_at(start);
    let (mut core, mut trailing) = rest.split_at(end - start);
    for possessive in ["'s", "\u{2019}s", "'S", "\u{2019}S"] {
        if let Some(stem) = core.strip_suffix(possessive) {
            let cut = core.len() - possessive.len();
            core = stem;
            trailing = &word[start + cut..];
            break;
        }
    }
    Token {
        leading,
        core,
        trailing,
    }
}

fn key_of(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

impl CustomWords {
    pub fn new<I, S>(words: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        // Uncapped: the default 4-character codes make every key sound like
        // any other that shares its first four sounds.
        let metaphone = DoubleMetaphone::new(None);
        let entries: Vec<Entry> = words
            .into_iter()
            .map(|w| w.as_ref().trim().to_string())
            .filter(|w| !w.is_empty())
            .filter_map(|text| {
                let key = key_of(&text);
                if key.is_empty() {
                    return None;
                }
                Some(Entry {
                    primary: metaphone.encode(&key),
                    alternate: metaphone.encode_alternate(&key),
                    word_count: text.split_whitespace().count(),
                    key,
                    text,
                })
            })
            .collect();
        let max_words = entries
            .iter()
            .map(|e| e.word_count)
            .max()
            .unwrap_or(1)
            .max(1);
        Self {
            entries,
            max_words,
            metaphone,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The longest run of recognized words a Correction can cover, and so
    /// the Hold-back. 1 when there is nothing to correct.
    pub fn window(&self) -> usize {
        if self.entries.is_empty() {
            1
        } else {
            self.max_words + SPLIT_ALLOWANCE
        }
    }

    /// The Custom words as written, in order.
    pub fn texts(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|e| e.text.as_str())
    }

    /// The best Correction starting at `words[0]`, looking at up to
    /// [`window`](Self::window) of them. Among equal scores the shortest
    /// run wins, so a word that is already right is never merged with its
    /// neighbour.
    pub fn correct_at(&self, words: &[&str]) -> Option<Correction> {
        if self.entries.is_empty() || words.is_empty() {
            return None;
        }
        let tokens: Vec<Token<'_>> = words
            .iter()
            .take(self.window())
            .map(|w| tokenize(w))
            .collect();
        let mut best: Option<(f64, Correction)> = None;
        let mut key = String::new();
        for (n, token) in tokens.iter().enumerate() {
            if n > 0 && !tokens[n - 1].trailing.is_empty() {
                break;
            }
            key.push_str(&key_of(token.core));
            if key.is_empty() {
                continue;
            }
            for entry in &self.entries {
                let Some(score) = self.score(&key, n + 1, entry) else {
                    continue;
                };
                if best.as_ref().is_none_or(|(s, _)| score < *s) {
                    let text = format!("{}{}{}", tokens[0].leading, entry.text, token.trailing);
                    best = Some((
                        score,
                        Correction {
                            consumed: n + 1,
                            text,
                        },
                    ));
                }
            }
        }
        best.map(|(_, c)| c)
    }

    /// How far `candidate`, the key of a run of `run_words` recognized
    /// words, is from `entry`; `None` when too far to be a Correction.
    fn score(&self, candidate: &str, run_words: usize, entry: &Entry) -> Option<f64> {
        if candidate == entry.key {
            return Some(0.0);
        }
        let candidate_len = candidate.chars().count();
        let entry_len = entry.key.chars().count();
        if candidate_len < MIN_FUZZY_LEN || entry_len < MIN_FUZZY_LEN {
            return None;
        }
        let max_len = candidate_len.max(entry_len) as f64;
        let len_diff = candidate_len.abs_diff(entry_len) as f64;
        if len_diff > (max_len * 0.25).max(2.0) {
            return None;
        }
        let lev = strsim::levenshtein(candidate, &entry.key) as f64 / max_len;
        let primary = self.metaphone.encode(candidate);
        let alternate = self.metaphone.encode_alternate(candidate);
        let sounds_alike = [&primary, &alternate]
            .iter()
            .any(|c| !c.is_empty() && (**c == entry.primary || **c == entry.alternate));
        let split_further = run_words > entry.word_count;
        let score = if sounds_alike {
            lev * PHONETIC_BOOST
        } else if split_further {
            return None;
        } else {
            lev
        };
        (score < ACCEPT_BELOW).then_some(score)
    }

    /// Corrects a whole text at once. Used for tests and offline runs; the
    /// streaming path is [`crate::dictation::Dictation`].
    pub fn correct_all(&self, text: &str) -> String {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut out: Vec<String> = Vec::new();
        let mut i = 0;
        while i < words.len() {
            match self.correct_at(&words[i..]) {
                Some(c) => {
                    out.push(c.text);
                    i += c.consumed;
                }
                None => {
                    out.push(words[i].to_string());
                    i += 1;
                }
            }
        }
        out.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn words(list: &[&str]) -> CustomWords {
        CustomWords::new(list.iter().copied())
    }

    #[test]
    fn tokenizes_edge_punctuation_and_possessives() {
        assert_eq!(
            tokenize("Caitlyn,"),
            Token {
                leading: "",
                core: "Caitlyn",
                trailing: ","
            }
        );
        assert_eq!(
            tokenize("\"Caitlyn\""),
            Token {
                leading: "\"",
                core: "Caitlyn",
                trailing: "\""
            }
        );
        assert_eq!(
            tokenize("Jason's"),
            Token {
                leading: "",
                core: "Jason",
                trailing: "'s"
            }
        );
        assert_eq!(
            tokenize("Jason\u{2019}s."),
            Token {
                leading: "",
                core: "Jason",
                trailing: "\u{2019}s."
            }
        );
        assert_eq!(
            tokenize("..."),
            Token {
                leading: "...",
                core: "",
                trailing: ""
            }
        );
        assert_eq!(
            tokenize("it's"),
            Token {
                leading: "",
                core: "it",
                trailing: "'s"
            }
        );
    }

    #[test]
    fn sound_alike_names_are_corrected() {
        let w = words(&["Caitlyn", "Carver"]);
        assert_eq!(
            w.correct_all("I met Kaitlin and Mr. Karver today."),
            "I met Caitlyn and Mr. Carver today."
        );
        assert_eq!(w.correct_all("Kaitlin's book"), "Caitlyn's book");
    }

    #[test]
    fn common_words_stay_put() {
        let w = words(&["Kat", "Carver", "Caitlyn"]);
        for text in [
            "the cat sat",
            "his career",
            "a kitten",
            "a carving",
            "cover it",
        ] {
            assert_eq!(w.correct_all(text), text);
        }
    }

    #[test]
    fn split_terms_merge_when_the_entry_says_so() {
        let w = words(&["ChatGPT", "Jason Carver"]);
        assert_eq!(w.window(), 3);
        assert_eq!(w.correct_all("ask chat g p t"), "ask chat g p t");
        assert_eq!(w.correct_all("ask chat gpt"), "ask ChatGPT");
        let w3 = CustomWords::new(["Chat G P T"]);
        assert_eq!(w3.correct_all("ask chat g p t now"), "ask Chat G P T now");
        assert_eq!(w.correct_all("jason carver said"), "Jason Carver said");
        assert_eq!(w.correct_all("jason, carver said"), "jason, carver said");
    }

    #[test]
    fn sharing_the_first_four_sounds_is_not_sounding_alike() {
        let w = words(&["Jason Carver"]);
        assert_eq!(w.correct_all("jason carlton said"), "jason carlton said");
    }

    #[test]
    fn a_name_the_engine_split_is_merged_when_it_sounds_right() {
        let w = words(&["Bernal Heights", "Caitlyn"]);
        assert_eq!(w.window(), 3);
        assert_eq!(
            w.correct_all("possibly burn all heights onto Linux"),
            "possibly Bernal Heights onto Linux"
        );
        assert_eq!(w.correct_all("kait lin came"), "Caitlyn came");
        // One word too many that only looks close stays as heard.
        assert_eq!(w.correct_all("burn all hopes"), "burn all hopes");
    }

    #[test]
    fn a_word_that_is_already_right_is_not_merged_with_its_neighbour() {
        let w = words(&["Caitlyn"]);
        let c = w.correct_at(&["Caitlyn", "is"]).unwrap();
        assert_eq!(
            c,
            Correction {
                consumed: 1,
                text: "Caitlyn".into()
            }
        );
    }

    #[test]
    fn empty_list_never_corrects() {
        let w = CustomWords::default();
        assert!(w.is_empty());
        assert_eq!(w.window(), 1);
        assert_eq!(w.correct_at(&["anything"]), None);
    }

    proptest! {
        #[test]
        fn correcting_twice_is_the_same_as_once(text in "[a-zA-Z' ,.]{0,60}") {
            let w = words(&["Caitlyn", "Carver", "ChatGPT"]);
            let once = w.correct_all(&text);
            prop_assert_eq!(w.correct_all(&once), once.clone());
        }

        #[test]
        fn output_has_one_word_per_consumed_run(text in "[a-zA-Z ]{0,40}") {
            let w = words(&["Caitlyn"]);
            let out = w.correct_all(&text);
            prop_assert!(out.split_whitespace().count() <= text.split_whitespace().count());
        }
    }
}
