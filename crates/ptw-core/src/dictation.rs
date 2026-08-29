//! One Dictation's text path: Committed text in, text to type out.
//!
//! Words are released in order and never retracted (ADR 0002). A word is
//! released once it is complete (followed by whitespace) and enough words
//! follow it to decide whether a Correction applies: the Hold-back.

use tracing::warn;

use crate::correction::CustomWords;

#[derive(Debug)]
pub struct Dictation {
    words: CustomWords,
    /// Recognized words already released, counted from the start of the text.
    released: usize,
}

impl Dictation {
    pub fn new(words: CustomWords) -> Self {
        Self { words, released: 0 }
    }

    /// Takes the whole Committed text so far and returns what to type now,
    /// each word followed by a space. Empty when nothing new is releasable.
    pub fn update(&mut self, committed: &str) -> String {
        let ends_in_whitespace = committed.chars().last().is_some_and(char::is_whitespace);
        let tokens: Vec<&str> = committed.split_whitespace().collect();
        let complete = if ends_in_whitespace {
            tokens.len()
        } else {
            tokens.len().saturating_sub(1)
        };
        self.release(&tokens[..complete], false)
    }

    /// Flush: takes the final text and returns everything not yet typed.
    pub fn finish(&mut self, final_text: &str) -> String {
        let tokens: Vec<&str> = final_text.split_whitespace().collect();
        self.release(&tokens, true)
    }

    fn release(&mut self, complete: &[&str], flushing: bool) -> String {
        if complete.len() < self.released {
            warn!(
                released = self.released,
                now = complete.len(),
                "committed text shrank; keeping what was typed"
            );
            return String::new();
        }
        let window = self.words.max_words();
        let mut out = String::new();
        while self.released < complete.len() {
            let decidable = flushing || self.released + window <= complete.len();
            if !decidable {
                break;
            }
            let end = (self.released + window).min(complete.len());
            match self.words.correct_at(&complete[self.released..end]) {
                Some(c) => {
                    out.push_str(&c.text);
                    self.released += c.consumed;
                }
                None => {
                    out.push_str(complete[self.released]);
                    self.released += 1;
                }
            }
            out.push(' ');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn dictation(list: &[&str]) -> Dictation {
        Dictation::new(CustomWords::new(list.iter().copied()))
    }

    #[test]
    fn releases_complete_words_and_flushes_the_rest() {
        let mut d = dictation(&[]);
        assert_eq!(d.update("Hel"), "");
        assert_eq!(d.update("Hello"), "");
        assert_eq!(d.update("Hello wor"), "Hello ");
        assert_eq!(d.update("Hello world"), "");
        assert_eq!(d.finish("Hello world."), "world. ");
    }

    #[test]
    fn silence_types_nothing() {
        let mut d = dictation(&["Caitlyn"]);
        assert_eq!(d.update(""), "");
        assert_eq!(d.finish("  "), "");
    }

    #[test]
    fn holds_back_enough_words_for_multi_word_corrections() {
        let mut d = dictation(&["Jason Carver"]);
        assert_eq!(d.update("I asked jason "), "I asked ");
        assert_eq!(d.update("I asked jason carver "), "Jason Carver ");
        assert_eq!(d.update("I asked jason carver about "), "");
        assert_eq!(d.finish("I asked jason carver about it"), "about it ");
    }

    #[test]
    fn single_word_lists_release_each_word_as_the_next_begins() {
        let mut d = dictation(&["Caitlyn"]);
        assert_eq!(d.update("Kaitlin said"), "Caitlyn ");
        assert_eq!(d.update("Kaitlin said hi"), "said ");
        assert_eq!(d.finish("Kaitlin said hi"), "hi ");
    }

    #[test]
    fn shrinking_committed_text_is_tolerated() {
        let mut d = dictation(&[]);
        assert_eq!(d.update("one two three "), "one two three ");
        assert_eq!(d.update("one "), "");
        assert_eq!(d.finish("one two three four"), "four ");
    }

    fn cuts(text: &str, points: &[usize]) -> Vec<String> {
        let mut prefixes: Vec<String> = points
            .iter()
            .map(|p| text.chars().take(*p).collect())
            .collect();
        prefixes.sort_by_key(String::len);
        prefixes
    }

    proptest! {
        /// Typing the released pieces in order gives the same text as
        /// correcting the final text in one go: the Hold-back never releases
        /// a word whose Correction could still change.
        #[test]
        fn streamed_output_equals_one_shot_output(
            text in "([a-zA-Z]{1,7}[ ,.]{0,2} ){0,12}",
            points in proptest::collection::vec(0usize..80, 0..12),
            list in proptest::sample::subsequence(vec!["Caitlyn", "Jason Carver", "ChatGPT", "Kat"], 0..=4),
        ) {
            let one_shot = {
                let mut d = Dictation::new(CustomWords::new(list.iter().copied()));
                d.finish(&text)
            };
            let mut d = Dictation::new(CustomWords::new(list.iter().copied()));
            let mut streamed = String::new();
            for prefix in cuts(&text, &points) {
                streamed.push_str(&d.update(&prefix));
            }
            streamed.push_str(&d.finish(&text));
            prop_assert_eq!(streamed, one_shot);
        }
    }
}
