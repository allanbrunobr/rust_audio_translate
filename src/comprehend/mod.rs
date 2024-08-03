use crate::AppState;
use aws_sdk_comprehend::types::LanguageCode;
use tokio::sync::MutexGuard;

#[derive(Debug)]
pub enum ComprehendError {
    AwsError(String),
}

impl std::fmt::Display for ComprehendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ComprehendError::AwsError(ref err) => write!(f, "AWS Error: {}", err),
        }
    }
}

impl std::error::Error for ComprehendError {}

/// Performs sentiment analysis on the given transcription text using AWS Comprehend.
///
/// This function takes a transcription text, sends it to AWS Comprehend for sentiment analysis,
/// and prints the detected sentiment and sentiment scores.
///
/// # Parameters
///
/// * `guard`: A `MutexGuard` holding the `AppState`, which contains the AWS Comprehend client.
/// * `transcription_text`: A string slice containing the text to be analyzed for sentiment.
///
/// # Returns
///
/// Returns `Ok(())` if the sentiment analysis is successful and the results are printed.
/// Returns a `ComprehendError` if there's an error during the AWS Comprehend API call.
///
/// # Errors
///
/// This function will return an error if the AWS Comprehend API call fails.
pub async fn perform_sentiment_analysis(
    guard: MutexGuard<'_, AppState>,
    transcription_text: &str,
) -> Result<(), ComprehendError> {
    let sentiment_result = guard
        .comprehend_client
        .detect_sentiment()
        .text(transcription_text)
        .language_code(LanguageCode::En)
        .send()
        .await
        .map_err(|e| ComprehendError::AwsError(e.to_string()))?;

    println!("Sentimento detectado: {:?}", sentiment_result.sentiment());
    println!("Pontuações: {:?}", sentiment_result.sentiment_score());

    Ok(())
}
