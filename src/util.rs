use unicase::UniCase;

use crate::static_resources::*;

//Checks if all letters in token are capitalized
pub(crate) fn is_all_caps<S: AsRef<str>>(token: S) -> bool {
    let token_ref = token.as_ref();
    ALL_CAPS_RE.is_match(token_ref) && token_ref.len() > 1
}

//Checks if token is in the list of NEGATION_SCALAR
pub(crate) fn is_negated(token: &UniCase<&str>) -> bool {
    if NEGATION_TOKENS.contains(token) {
        return true;
    }
    token.contains("n't")
}

//Normalizes score between -1.0 and 1.0. Alpha value is expected upper limit for a score
pub(crate) fn normalize_score(score: f64) -> f64 {
    let norm_score = score / (score * score + NORMALIZATION_ALPHA).sqrt();
    if norm_score < -1.0 {
        return -1.0;
    } else if norm_score > 1.0 {
        return 1.0;
    }
    norm_score
}

//Checks how previous tokens affect the valence of the current token
pub(crate) fn scalar_inc_dec(token: &UniCase<&str>, valence: f64, has_mixed_caps: bool) -> f64 {
    let mut scalar = 0.0;
    if BOOSTER_DICT.contains_key(token) {
        scalar = *BOOSTER_DICT.get(token).unwrap();
        if valence < 0.0 {
            scalar *= -1.0;
        }
        if is_all_caps(token) && has_mixed_caps {
            if valence > 0.0 {
                scalar += C_INCR;
            } else {
                scalar -= C_INCR;
            }
        }
    }
    scalar
}

pub(crate) fn sum_sentiment_scores(scores: Vec<f64>) -> (f64, f64, u32) {
    let (mut pos_sum, mut neg_sum, mut neu_count) = (0f64, 0f64, 0);
    for score in scores {
        if score > 0f64 {
            pos_sum += score + 1.0;
        } else if score < 0f64 {
            neg_sum += score - 1.0;
        } else {
            neu_count += 1;
        }
    }
    (pos_sum, neg_sum, neu_count)
}
