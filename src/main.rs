// extern crate vader_sentiment;

// fn main() {
//     vader_sentiment::demo::run_demo();
// }

use clap::Parser;
use vader_sentiment::SentimentIntensityAnalyzer;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    text: String,
}

fn main() {
    let args = Args::parse();

    let analyzer = SentimentIntensityAnalyzer::new();
    let scores = analyzer.polarity_scores(&args.text);

    println!("Polarity scores");
    println!("{:#?}", scores);
}
