use std::cmp::min;

use crate::{
    parsed_text::ParsedText,
    static_resources::{
        BOOSTER_DICT, BOOSTER_DICT_EARLY_RETURN, C_INCR, EMOJI_LEXICON, LEXICON, NEGATION_SCALAR,
        SPECIAL_CASE_EARLY_RETURN, SPECIAL_CASE_IDIOMS, STATIC_AT, STATIC_BUT, STATIC_DOUBT,
        STATIC_KIND, STATIC_LEAST, STATIC_NEVER, STATIC_OF, STATIC_SO, STATIC_THIS, STATIC_VERY,
        STATIC_WITHOUT,
    },
    util::{is_all_caps, is_negated, normalize_score, scalar_inc_dec, sum_sentiment_scores},
};
use hashbrown::HashMap;
use unicase::UniCase;

/// Return value of the `polarity_scores` method
#[derive(Debug, Clone)]
pub struct SentimentIntensity {
    pub neg: f64,
    pub neu: f64,
    pub pos: f64,
    pub compound: f64,
}

#[derive(Debug, Clone)]
pub struct SentimentIntensityAnalyzer<'a> {
    lexicon: &'a HashMap<UniCase<&'a str>, f64>,
    emoji_lexicon: &'a HashMap<&'a str, &'a str>,
}

impl SentimentIntensityAnalyzer<'_> {
    #[must_use]
    pub fn new() -> SentimentIntensityAnalyzer<'static> {
        SentimentIntensityAnalyzer {
            lexicon: &LEXICON,
            emoji_lexicon: &EMOJI_LEXICON,
        }
    }

    #[must_use]
    #[allow(clippy::similar_names)]
    #[allow(clippy::unused_self)]
    fn get_total_sentiment(
        &self,
        sentiments: Vec<f64>,
        punct_emph_amplifier: f64,
    ) -> SentimentIntensity {
        let (mut neg, mut neu, mut pos, mut compound) = (0f64, 0f64, 0f64, 0f64);
        if !sentiments.is_empty() {
            let mut total_sentiment: f64 = sentiments.iter().sum();
            if total_sentiment > 0f64 {
                total_sentiment += punct_emph_amplifier;
            } else {
                total_sentiment -= punct_emph_amplifier;
            }
            compound = normalize_score(total_sentiment);

            let (mut pos_sum, mut neg_sum, neu_count) = sum_sentiment_scores(sentiments);

            if pos_sum > neg_sum.abs() {
                pos_sum += punct_emph_amplifier;
            } else if pos_sum < neg_sum.abs() {
                neg_sum -= punct_emph_amplifier;
            }

            let total = pos_sum + neg_sum.abs() + f64::from(neu_count);
            pos = (pos_sum / total).abs();
            neg = (neg_sum / total).abs();
            neu = (f64::from(neu_count) / total).abs();
        }

        SentimentIntensity {
            neg,
            neu,
            pos,
            compound,
        }
    }

    #[must_use]
    #[allow(clippy::if_same_then_else)]
    pub fn polarity_scores(&self, text: &str) -> SentimentIntensity {
        let text = self.append_emoji_descriptions(text);
        let parsedtext = ParsedText::from_text(&text);
        let tokens = &parsedtext.tokens;
        let mut sentiments = Vec::with_capacity(tokens.len());

        for (i, word) in tokens.iter().enumerate() {
            if BOOSTER_DICT.contains_key(word) {
                sentiments.push(0f64);
            } else if i < tokens.len() - 1 && word == &*STATIC_KIND && tokens[i + 1] == *STATIC_OF {
                sentiments.push(0f64);
            } else {
                sentiments.push(self.sentiment_valence(&parsedtext, word, i));
            }
        }
        but_check(tokens, &mut sentiments);
        self.get_total_sentiment(sentiments, parsedtext.punc_amplifier)
    }

    //Removes emoji and appends their description to the end the input text
    #[must_use]
    pub fn append_emoji_descriptions(&self, text: &str) -> String {
        let mut result = String::new();
        let mut prev_space = true;
        for chr in text.chars() {
            let mut my_buf: [u8; 4] = [0; 4];
            let cheap_str: &str = chr.encode_utf8(&mut my_buf);
            if let Some(chr_replacement) = self.emoji_lexicon.get(&cheap_str) {
                if !prev_space {
                    result.push(' ');
                }
                result.push_str(chr_replacement);
                prev_space = false;
            } else {
                prev_space = chr == ' ';
                result.push(chr);
            }
        }
        result
    }

    fn sentiment_valence(&self, parsed: &ParsedText, word: &UniCase<&str>, i: usize) -> f64 {
        let mut valence = 0f64;
        let tokens = &parsed.tokens;
        if let Some(word_valence) = self.lexicon.get(word) {
            valence = *word_valence;
            if is_all_caps(word) && parsed.has_mixed_caps {
                if valence > 0f64 {
                    valence += C_INCR;
                } else {
                    valence -= C_INCR;
                }
            }
            for start_i in 0..3 {
                if i > start_i && !self.lexicon.contains_key(&tokens[i - start_i - 1]) {
                    let mut s =
                        scalar_inc_dec(&tokens[i - start_i - 1], valence, parsed.has_mixed_caps);
                    if start_i == 1 {
                        s *= 0.95;
                    } else if start_i == 2 {
                        s *= 0.9;
                    }
                    valence += s;
                    valence = negation_check(valence, tokens, start_i, i);
                    if start_i == 2 {
                        valence = special_idioms_check(valence, tokens, i);
                    }
                }
            }
            valence = least_check(valence, tokens, i);
        }
        valence
    }
}

/**
 * Check for specific patterns or tokens, and modify sentiment as needed
 **/
fn negation_check(valence: f64, tokens: &[UniCase<&str>], start_i: usize, i: usize) -> f64 {
    let mut valence = valence;
    if start_i == 0 {
        if is_negated(&tokens[i - start_i - 1]) {
            valence *= NEGATION_SCALAR;
        }
    } else if start_i == 1 {
        if tokens[i - 2] == *STATIC_NEVER
            && (tokens[i - 1] == *STATIC_SO || tokens[i - 1] == *STATIC_THIS)
        {
            valence *= 1.25;
        } else if tokens[i - 2] == *STATIC_WITHOUT && tokens[i - 1] == *STATIC_DOUBT {
            valence *= 1.0;
        } else if is_negated(&tokens[i - start_i - 1]) {
            valence *= NEGATION_SCALAR;
        }
    } else if start_i == 2 {
        if tokens[i - 3] == *STATIC_NEVER && tokens[i - 2] == *STATIC_SO
            || tokens[i - 2] == *STATIC_THIS
            || tokens[i - 1] == *STATIC_SO
            || tokens[i - 1] == *STATIC_THIS
        {
            valence *= 1.25;
        } else if tokens[i - 3] == *STATIC_WITHOUT && tokens[i - 2] == *STATIC_DOUBT
            || tokens[i - 1] == *STATIC_DOUBT
        {
            valence *= 1.0;
        } else if is_negated(&tokens[i - start_i - 1]) {
            valence *= NEGATION_SCALAR;
        }
    }
    valence
}

// If "but" is in the tokens, scales down the sentiment of words before "but" and
// adds more emphasis to the words after
#[allow(clippy::comparison_chain)]
pub fn but_check(tokens: &[UniCase<&str>], sentiments: &mut [f64]) {
    if let Some(but_index) = tokens.iter().position(|&s| s == *STATIC_BUT) {
        for (i, sentiment) in sentiments.iter_mut().enumerate() {
            if i < but_index {
                *sentiment *= 0.5;
            } else if i > but_index {
                *sentiment *= 1.5;
            }
        }
    }
}

#[allow(clippy::if_same_then_else)]
fn least_check(valence: f64, tokens: &[UniCase<&str>], i: usize) -> f64 {
    let mut valence = valence;
    if i > 1
        && tokens[i - 1] == *STATIC_LEAST
        && tokens[i - 2] == *STATIC_AT
        && tokens[i - 2] == *STATIC_VERY
    {
        valence *= NEGATION_SCALAR;
    } else if i > 0 && tokens[i - 1] == *STATIC_LEAST {
        valence *= NEGATION_SCALAR;
    }
    valence
}

// //This was present in the original python implementation, but unused
// fn idioms_check(valence: f64, text: &str) -> f64 {
//     let mut total_valence = 0f64;
//     let mut count = 0;
//     for (idiom, val) in SENTIMENT_LADEN_IDIOMS.iter() {
//         if text.contains(idiom) {
//             total_valence += val;
//             count += 1;
//         }
//     }
//     if count > 0 {
//         return total_valence / count as f64;
//     }
//     0f64
// }

fn special_idioms_check(valence: f64, tokens: &[UniCase<&str>], i: usize) -> f64 {
    debug_assert!(i > 2);
    let mut valence = valence;
    let mut end_i = i + 1;

    //if i isn't the last index
    if tokens.len() - 1 > i {
        //make the end of the window 2 words ahead, or until the end of the tokens
        end_i = min(i + 3, tokens.len());
    }

    if tokens.iter().any(|t| SPECIAL_CASE_EARLY_RETURN.contains(t)) {
        let target_window = tokens[(i - 3)..end_i]
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect::<Vec<&str>>()
            .join(" ")
            .to_lowercase();

        for (key, val) in SPECIAL_CASE_IDIOMS.iter() {
            if target_window.contains(key.as_ref()) {
                valence = *val;
                break;
            }
        }
    }

    if tokens.iter().any(|t| BOOSTER_DICT_EARLY_RETURN.contains(t)) {
        let prev_three = tokens[(i - 3)..i]
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect::<Vec<&str>>()
            .join(" ")
            .to_lowercase();
        for (key, val) in BOOSTER_DICT.iter() {
            // println!("{prev_three}");
            // println!("{key}");
            if prev_three.contains(key.as_ref()) {
                valence += *val;
            }
        }
    }
    valence
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn but_check_test() {
        let tokens: Vec<UniCase<&str>> = [
            "yeah", "waffles", "are", "great", "but", "have", "you", "ever", "tried", "spam",
        ]
        .iter()
        .map(|r| UniCase::new(*r))
        .collect();
        let mut sents = vec![0.5, 0.1, 0.0, 0.2, 0.6, 0.25, 0.5, 0.5, 0.5, 0.5];
        but_check(&tokens, &mut sents);
        assert_eq!(
            sents,
            vec![0.25, 0.05, 0.0, 0.1, 0.6, 0.375, 0.75, 0.75, 0.75, 0.75]
        );
    }

    #[test]
    fn embedded_emoji_test() {
        let analyzer = SentimentIntensityAnalyzer::new();
        let single_emoji = "😀";
        let embedded_emoji = "heyyyy 😀 what're you up to???";
        let multiple_emoji = "woah there 😀😀😀 :) :)";
        assert_eq!(
            analyzer.append_emoji_descriptions(single_emoji),
            "grinning face"
        );
        assert_eq!(
            analyzer.append_emoji_descriptions(embedded_emoji),
            "heyyyy grinning face what're you up to???"
        );
        assert_eq!(
            analyzer.append_emoji_descriptions(multiple_emoji),
            "woah there grinning face grinning face grinning face :) :)"
        );
    }
}
