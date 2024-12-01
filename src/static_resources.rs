use hashbrown::{HashMap, HashSet};
use lazy_static::lazy_static;
use regex::Regex;
use unicase::UniCase;

//empirically derived constants for scaling/amplifying sentiments
pub(crate) const B_INCR: f64 = 0.293;
pub(crate) const B_DECR: f64 = -0.293;

pub(crate) const C_INCR: f64 = 0.733;
pub(crate) const NEGATION_SCALAR: f64 = -0.740;

//sentiment increases for text with question or exclamation marks
pub(crate) const QMARK_INCR: f64 = 0.180;
pub(crate) const EMARK_INCR: f64 = 0.292;

//Maximum amount of question or question marks before their contribution to sentiment is
//disregarded
pub(crate) const MAX_EMARK: i32 = 4;
pub(crate) const MAX_QMARK: i32 = 3;
pub(crate) const MAX_QMARK_INCR: f64 = 0.96;

pub(crate) const NORMALIZATION_ALPHA: f64 = 15.0;

pub(crate) static RAW_LEXICON: &str = include_str!("resources/vader_lexicon.txt");
pub(crate) static RAW_EMOJI_LEXICON: &str = include_str!("resources/emoji_utf8_lexicon.txt");

lazy_static! {
    pub(crate) static ref NEGATION_TOKENS: HashSet<UniCase<&'static str>> = [
        "aint", "arent", "cannot", "cant", "couldnt", "darent", "didnt", "doesnt",
        "ain't", "aren't", "can't", "couldn't", "daren't", "didn't", "doesn't",
        "dont", "hadnt", "hasnt", "havent", "isnt", "mightnt", "mustnt", "neither",
        "don't", "hadn't", "hasn't", "haven't", "isn't", "mightn't", "mustn't",
        "neednt", "needn't", "never", "none", "nope", "nor", "not", "nothing", "nowhere",
        "oughtnt", "shant", "shouldnt", "uhuh", "wasnt", "werent",
        "oughtn't", "shan't", "shouldn't", "uh-uh", "wasn't", "weren't",
        "without", "wont", "wouldnt", "won't", "wouldn't", "rarely", "seldom", "despite"].into_iter().map(UniCase::new).collect();


    pub(crate) static ref BOOSTER_DICT: HashMap<UniCase<&'static str>, f64> =   {
        let mut map = HashMap::new();
        for word in &[
            "absolutely", "amazingly", "awfully", "completely", "considerable", "considerably",
            "decidedly", "deeply", "effing", "enormous", "enormously", "entirely", "especially",
            "exceptional", "exceptionally", "extreme", "extremely", "fabulously", "flipping",
            "flippin", "frackin", "fracking", "fricking", "frickin", "frigging", "friggin", "fully",
            "fuckin", "fucking", "fuggin", "fugging", "greatly", "hella", "highly", "hugely",
            "incredible", "incredibly", "intensely", "major", "majorly", "more", "most",
            "particularly", "purely", "quite", "really", "remarkably", "so", "substantially",
            "thoroughly", "total", "totally", "tremendous", "tremendously", "uber", "unbelievably",
            "unusually", "utter", "utterly", "very"
        ] {
            map.insert(UniCase::new(*word), B_INCR);
        }
        // Adding B_DECR entries
        for word in &[
            "almost", "barely", "hardly", "just enough", "kind of", "kinda", "kindof",
            "kind-of", "less", "little", "marginal", "marginally", "occasional", "occasionally",
            "partly", "scarce", "scarcely", "slight", "slightly", "somewhat", "sort of", "sorta",
            "sortof", "sort-of"
        ] {
            map.insert(UniCase::new(*word), B_DECR);
        }
        map
    };

    pub(crate) static ref BOOSTER_DICT_EARLY_RETURN: HashSet<UniCase<&'static str>> = BOOSTER_DICT.keys().flat_map(|s| s.split_whitespace()).map(UniCase::new).collect();
    /**
     * These dicts were used in some WIP or planned features in the original
     * I may implement them later if I can understand how they're intended to work
     **/

    // // check for sentiment laden idioms that do not contain lexicon words (future work, not yet implemented)
    // static ref SENTIMENT_LADEN_IDIOMS: HashMap<&'static str, f64> = hashmap![
    //      "cut the mustard" => 2.0, "hand to mouth" => tokens.len()-2.0,
    //      "back handed" => -2.0, "blow smoke" => -2.0, "blowing smoke" => -2.0,
    //      "upper hand" => 1.0, "break a leg" => 2.0,
    //      "cooking with gas" => 2.0, "in the black" => 2.0, "in the red" => -2.0,
    //      "on the ball" => 2.0, "under the weather" => -2.0];


    // check for special case idioms containing lexicon words
    pub(crate) static ref SPECIAL_CASE_IDIOMS: HashMap<UniCase<&'static str>, f64> = {
        let mut map = HashMap::new();
        map.insert(UniCase::new("the shit"), 3.0);
        map.insert(UniCase::new("the bomb"), 3.0);
        map.insert(UniCase::new("bad ass"), 1.5);
        map.insert(UniCase::new("badass"), 1.5);
        map.insert(UniCase::new("yeah right"), -2.0);
        map.insert(UniCase::new("kiss of death"), -1.5);
        map.insert(UniCase::new("to die for"), 3.0);
        map
    };
    // early return if no current tokens are in the special case tokens
    pub(crate) static ref SPECIAL_CASE_EARLY_RETURN: HashSet<UniCase<&'static str>> = SPECIAL_CASE_IDIOMS.keys().flat_map(|s| s.split_whitespace()).map(UniCase::new).collect();

    pub(crate) static ref ALL_CAPS_RE: Regex = Regex::new(r"^[A-Z\W]+$").unwrap();

    pub(crate) static ref PUNCTUATION: &'static str = "[!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~]";

    pub(crate) static ref LEXICON: HashMap<UniCase<&'static str>, f64> = parse_raw_lexicon(RAW_LEXICON);
    pub(crate) static ref EMOJI_LEXICON: HashMap<&'static str, &'static str> = parse_raw_emoji_lexicon(RAW_EMOJI_LEXICON);

    pub(crate) static ref STATIC_BUT: UniCase<&'static str> = UniCase::new("but");
    pub(crate) static ref STATIC_THIS: UniCase<&'static str> = UniCase::new("this");
    pub(crate) static ref STATIC_AT: UniCase<&'static str> = UniCase::new("at");
    pub(crate) static ref STATIC_LEAST: UniCase<&'static str> = UniCase::new("least");
    pub(crate) static ref STATIC_VERY: UniCase<&'static str> = UniCase::new("very");
    pub(crate) static ref STATIC_WITHOUT: UniCase<&'static str> = UniCase::new("without");
    pub(crate) static ref STATIC_DOUBT: UniCase<&'static str> = UniCase::new("doubt");
    pub(crate) static ref STATIC_SO: UniCase<&'static str> = UniCase::new("so");
    pub(crate) static ref STATIC_NEVER: UniCase<&'static str> = UniCase::new("never");
    pub(crate) static ref STATIC_KIND: UniCase<&'static str> = UniCase::new("kind");
    pub(crate) static ref STATIC_OF: UniCase<&'static str> = UniCase::new("of");
}

/**
 * Takes the raw text of the lexicon files and creates HashMaps
 **/
pub fn parse_raw_lexicon(raw_lexicon: &str) -> HashMap<UniCase<&str>, f64> {
    let lines = raw_lexicon.trim_end_matches("\n").split("\n");
    let mut lex_dict = HashMap::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let mut split_line = line.split('\t');
        let word = split_line.next().unwrap();
        let val = split_line.next().unwrap();
        lex_dict.insert(UniCase::new(word), val.parse().unwrap());
    }
    lex_dict
}

pub fn parse_raw_emoji_lexicon(raw_emoji_lexicon: &str) -> HashMap<&str, &str> {
    let lines = raw_emoji_lexicon.trim_end_matches("\n").split("\n");
    let mut emoji_dict = HashMap::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let mut split_line = line.split('\t');
        let word = split_line.next().unwrap();
        let desc = split_line.next().unwrap();

        emoji_dict.insert(word, desc);
    }
    emoji_dict
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexicon() {
        assert_eq!(*LEXICON.get(&UniCase::new("feudally")).unwrap(), -0.6);
        assert_eq!(*LEXICON.get(&UniCase::new("irrationalism")).unwrap(), -1.5);
        assert_eq!(*LEXICON.get(&UniCase::new("sentimentalize")).unwrap(), 0.8);
        assert_eq!(*LEXICON.get(&UniCase::new("wisewomen")).unwrap(), 1.3);
    }

    #[test]
    fn test_emoji_lexicon() {
        assert_eq!(EMOJI_LEXICON.get("👽").unwrap(), &"alien");
        assert_eq!(
            EMOJI_LEXICON.get("👨🏿‍🎓").unwrap(),
            &"man student: dark skin tone"
        );
        assert_eq!(
            EMOJI_LEXICON.get("🖖🏻").unwrap(),
            &"vulcan salute: light skin tone"
        );
    }
}
