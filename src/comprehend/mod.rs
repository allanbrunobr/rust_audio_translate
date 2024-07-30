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
