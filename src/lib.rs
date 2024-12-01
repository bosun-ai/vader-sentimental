//! * If you use the VADER sentiment analysis tools, please cite:
//!  * Hutto, C.J. & Gilbert, E.E. (2014). VADER: A Parsimonious Rule-based Model for
//!  * Sentiment Analysis of Social Media Text. Eighth International Conference on
//!  * Weblogs and Social Media (ICWSM-14). Ann Arbor, MI, June 2014.
mod parsed_text;
mod sentiment_intensity_analyzer;
mod static_resources;
mod util;

pub use crate::sentiment_intensity_analyzer::SentimentIntensityAnalyzer;
