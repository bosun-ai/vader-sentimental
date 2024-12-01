use vader_sentimental::SentimentIntensityAnalyzer;

#[test]
fn test_positive() {
    let sentences = vec![
        "VADER is smart, handsome, and funny.", // positive sentence example
        "VADER is smart, handsome, and funny!",
        // punctuation emphasis handled correctly (sentiment intensity adjusted)
        "VADER is very smart, handsome, and funny.",
        // booster words handled correctly (sentiment intensity adjusted)
        "VADER is VERY SMART, handsome, and FUNNY.", // emphasis for ALLCAPS handled
        "VADER is VERY SMART, handsome, and FUNNY!!!",
        // combination of signals - VADER appropriately adjusts intensity
        "VADER is VERY SMART, uber handsome, and FRIGGIN FUNNY!!!",
        // booster words & punctuation make this close to ceiling for score
        "VADER is not smart, handsome, nor funny.", // negation sentence example
        "The book was good.",                       // positive sentence
        "At least it isn't a horrible book.",       // negated negative sentence with contraction
        "The book was only kind of good.",
        // qualified positive sentence is handled correctly (intensity adjusted)
        "The plot was good, but the characters are uncompelling and the dialog is not great.",
        // mixed negation sentence
        "Today SUX!", // negative slang with capitalization emphasis
        "Today only kinda sux! But I'll get by, lol",
        // mixed sentiment example with slang and constrastive conjunction "but"
        "Make sure you :) or :D today!", // emoticons handled
        "Catch utf-8 emoji such as 💘 and 💋 and 😁", // emojis handled
        "Not bad at all",
    ];

    let analyzer = SentimentIntensityAnalyzer::new();
    for sentence in sentences {
        let mut scores = analyzer
            .polarity_scores(sentence)
            .into_values()
            .collect::<Vec<_>>();

        scores.sort_by(f64::total_cmp);

        insta::assert_snapshot!(format!("{:-<65} {:#?}", sentence, scores));
    }
}

#[test]
fn test_negative() {
    let tricky_sentences = vec![
        "Sentiment analysis has never been good.",
        "Sentiment analysis has never been this good!",
        "Most automated sentiment analysis tools are shit.",
        "With VADER, sentiment analysis is the shit!",
        "Other sentiment analysis tools can be quite bad.",
        "On the other hand, VADER is quite bad ass",
        "VADER is such a badass!", // slang with punctuation emphasis
        "Without a doubt, excellent idea.",
        "Roger Dodger is one of the most compelling variations on this theme.",
        "Roger Dodger is at least compelling as a variation on the theme.",
        "Roger Dodger is one of the least compelling variations on this theme.",
        "Not such a badass after all.", // Capitalized negation with slang
        "Without a doubt, an excellent idea.",
    ];

    let analyzer = SentimentIntensityAnalyzer::new();
    for sentence in tricky_sentences {
        let mut scores = analyzer
            .polarity_scores(sentence)
            .into_values()
            .collect::<Vec<_>>();

        scores.sort_by(f64::total_cmp);

        insta::assert_snapshot!(format!("{:-<65} {:#?}", sentence, scores));
    }
}
