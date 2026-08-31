//! Pronunciation lookups in the CMU Pronouncing Dictionary, vendored at
//! `data/cmudict.dict`. Unlike Double Metaphone this keeps vowels, so it can
//! tell "clawed" (K L AO D) from "cloud" (K L AW D). Stress digits are
//! stripped: "clawed" and "Claude" differ only in stress marks and are the
//! same word to us.
//!
//! The dictionary parses once, on first lookup, and only code that hits
//! this module pays for it (about 135k entries).

use std::collections::HashMap;
use std::sync::OnceLock;

/// One way to say a word, as stress-free ARPAbet phonemes.
pub type Pronunciation = Vec<&'static str>;

static RAW: &str = include_str!("../data/cmudict.dict");
static DICT: OnceLock<HashMap<&'static str, Vec<Pronunciation>>> = OnceLock::new();

fn dict() -> &'static HashMap<&'static str, Vec<Pronunciation>> {
    DICT.get_or_init(|| {
        let mut map: HashMap<&'static str, Vec<Pronunciation>> = HashMap::new();
        for line in RAW.lines() {
            let mut parts = line.split_whitespace();
            let Some(label) = parts.next() else { continue };
            // Alternates are labeled "word(2)"; comments trail after "#".
            let word = label.split('(').next().unwrap_or(label);
            let phonemes: Pronunciation = parts
                .take_while(|p| !p.starts_with('#'))
                .map(|p| p.trim_end_matches(|c: char| c.is_ascii_digit()))
                .collect();
            if word.is_empty() || phonemes.is_empty() {
                continue;
            }
            map.entry(word).or_default().push(phonemes);
        }
        map
    })
}

/// Every known way to say `word`. The dictionary's words are lowercase and
/// keep internal punctuation ("don't"), so pass the word that way.
pub fn pronunciations(word: &str) -> Option<&'static [Pronunciation]> {
    dict().get(word).map(Vec::as_slice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_vowels_metaphone_drops() {
        let say = |w| pronunciations(w).unwrap();
        assert_eq!(say("clawed"), say("claude"));
        assert_ne!(say("cloud"), say("clawed"));
        assert_eq!(say("colonel"), say("kernel"));
        // Not every real word is in: "clod" falls back to Metaphone.
        assert!(pronunciations("clod").is_none());
    }

    #[test]
    fn alternates_comments_and_contractions_parse() {
        assert_eq!(pronunciations("a").unwrap().len(), 2);
        // "gdp G IY1 D IY1 P IY1 # abbrev" must not swallow the comment.
        assert_eq!(pronunciations("gdp").unwrap()[0].len(), 6);
        assert!(pronunciations("don't").is_some());
        assert!(pronunciations("burnall").is_none());
        assert!(dict().len() > 100_000);
    }
}
