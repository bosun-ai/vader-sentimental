use clap::Parser;
use vader_sentimental::SentimentIntensityAnalyzer;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    text: String,
}

/// Simple CLI tool to analyze the sentiment of a given text.
fn main() {
    let args = Args::parse();

    let analyzer = SentimentIntensityAnalyzer::new();
    let scores = analyzer.polarity_scores(&args.text);

    println!("Polarity scores");
    println!("{scores:#?}");
}
