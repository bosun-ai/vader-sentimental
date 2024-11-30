use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn run_vader_sentiment(sentence: &str) {
    let analyzer = vader_sentiment::SentimentIntensityAnalyzer::new();

    analyzer.polarity_scores(sentence);
}

fn run_vader_sentimental(sentence: &str) {
    let analyzer = vader_sentimental::SentimentIntensityAnalyzer::new();

    analyzer.polarity_scores(sentence);
}

static SENTENCES: &[&str] = &[
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
    "Make sure you :) or :D today!",              // emoticons handled
    "Catch utf-8 emoji such as 💘 and 💋 and 😁", // emojis handled
    "Not bad at all",
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

fn bench_vader(c: &mut Criterion) {
    let mut group = c.benchmark_group("SENTENCES");

    for sentence in SENTENCES {
        group.throughput(Throughput::Elements(sentence.len() as u64));
        group
            .significance_level(0.001)
            .sample_size(10)
            .measurement_time(Duration::from_secs(1))
            .warm_up_time(Duration::from_millis(10));
        group.bench_with_input(
            BenchmarkId::new("vader_sentiment", sentence),
            sentence,
            |b, i| b.iter(|| run_vader_sentiment(black_box(*i))),
        );
        group.bench_with_input(
            BenchmarkId::new("vader_sentimental", sentence),
            sentence,
            |b, i| b.iter(|| run_vader_sentimental(black_box(*i))),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_vader);
criterion_main!(benches);
