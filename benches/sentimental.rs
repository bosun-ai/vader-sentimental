use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn run_vader_sentimental(sentence: &str) {
    let analyzer = vader_sentimental::SentimentIntensityAnalyzer::new();

    let _ = analyzer.polarity_scores(sentence);
}

static SENTENCES: &[&str] = &[
    "VADER is smart, handsome, and funny.", // positive sentence example
    "Sentiment analysis has never been good.",
];

fn bench_vader(c: &mut Criterion) {
    let mut group = c.benchmark_group("POSITIVE");

    for sentence in SENTENCES {
        group.throughput(Throughput::Bytes(sentence.len() as u64));
        group.significance_level(0.5).sample_size(10);
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
