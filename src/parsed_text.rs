use std::cmp::min;

use unicase::UniCase;

use crate::static_resources::{
    EMARK_INCR, MAX_EMARK, MAX_QMARK, MAX_QMARK_INCR, PUNCTUATION, QMARK_INCR,
};
use crate::util::is_all_caps;

/**
 *  Stores tokens and useful info about text
 **/
pub struct ParsedText<'a> {
    pub tokens: Vec<UniCase<&'a str>>,
    pub has_mixed_caps: bool,
    pub punc_amplifier: f64,
}

impl ParsedText<'_> {
    //Tokenizes and extracts useful properties of input text
    pub fn from_text(text: &str) -> ParsedText<'_> {
        let tokens = ParsedText::tokenize(text);
        let has_mixed_caps = ParsedText::has_mixed_caps(&tokens);
        let punc_amplifier = ParsedText::get_punctuation_emphasis(text);
        ParsedText {
            tokens,
            has_mixed_caps,
            punc_amplifier,
        }
    }

    pub fn tokenize(text: &str) -> Vec<UniCase<&str>> {
        let tokens = text
            .split_whitespace()
            .filter_map(|s| {
                if s.len() <= 1 {
                    None
                } else {
                    Some(UniCase::new(ParsedText::strip_punc_if_word(s)))
                }
            })
            .collect();
        tokens
    }

    // Removes punctuation from words, ie "hello!!!" -> "hello" and ",don't??" -> "don't"
    // Keeps most emoticons, ie ":^)" -> ":^)"\
    fn strip_punc_if_word(token: &str) -> &str {
        let stripped = token.trim_matches(|c| PUNCTUATION.contains(c));
        if stripped.len() <= 1 {
            return token;
        }
        stripped
    }

    // Determines if message has a mix of both all caps and non all caps words
    pub fn has_mixed_caps<S: AsRef<str>>(tokens: &[S]) -> bool {
        let (mut has_caps, mut has_non_caps) = (false, false);
        for token in tokens {
            if is_all_caps(token.as_ref()) {
                has_caps = true;
            } else {
                has_non_caps = true;
            }
            if has_non_caps && has_caps {
                return true;
            }
        }
        false
    }

    //uses empirical values to determine how the use of '?' and '!' contribute to sentiment
    // TODO: Floating points here is a concern, use `rust_decimal` instead.
    // TODO: Naive way of counting bytes, use `bytecount` crate
    // TODO: usize to i32 _can_ overflow
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::naive_bytecount)]
    fn get_punctuation_emphasis(text: &str) -> f64 {
        let emark_count = text.as_bytes().iter().filter(|b| **b == b'!').count() as i32;
        let qmark_count = text.as_bytes().iter().filter(|b| **b == b'?').count() as i32;

        let emark_emph = f64::from(min(emark_count, MAX_EMARK)) * EMARK_INCR;
        let mut qmark_emph = f64::from(qmark_count) * QMARK_INCR;
        if qmark_count > MAX_QMARK {
            qmark_emph = MAX_QMARK_INCR;
        }
        qmark_emph + emark_emph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_parsed_text() {
        let messy_text = "WOAH!!! ,Who? DO u Think you're?? :) :D :^(";
        let parsed_messy = ParsedText::from_text(messy_text);
        let expected_text: Vec<UniCase<&str>> =
            ["WOAH", "Who", "DO", "Think", "you\'re", ":)", ":D", ":^("]
                .iter()
                .map(|r| UniCase::new(*r))
                .collect();
        assert_eq!(parsed_messy.tokens, expected_text);
        assert!(parsed_messy.has_mixed_caps);
        assert_eq!(parsed_messy.punc_amplifier, 1.416);

        assert!(!ParsedText::has_mixed_caps(&ParsedText::tokenize(
            "yeah!!! I'm aLLERGIC to ShouTING."
        )));
        assert!(!ParsedText::has_mixed_caps(&ParsedText::tokenize(
            "OH MAN I LOVE SHOUTING!"
        )));
        assert!(ParsedText::has_mixed_caps(&ParsedText::tokenize(
            "I guess I CAN'T MAKE UP MY MIND"
        )));
        assert!(ParsedText::has_mixed_caps(&ParsedText::tokenize(
            "Hmm, yeah ME NEITHER"
        )));
    }
}
