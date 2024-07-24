use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Serialize)]
struct SpeechToTextRequest {
    audio: AudioContent,
    config: RecognitionConfig,
}

#[derive(Serialize)]
struct AudioContent {
    content: String,
}

#[derive(Serialize)]
struct RecognitionConfig {
    encoding: String,
    sample_rate_hertz: i32,
    language_code: String,
}

#[derive(Deserialize)]
struct SpeechToTextResponse {
    results: Vec<SpeechRecognitionResult>,
}

#[derive(Deserialize)]
struct SpeechRecognitionResult {
    alternatives: Vec<SpeechRecognitionAlternative>,
}

#[derive(Deserialize)]
struct SpeechRecognitionAlternative {
    transcript: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set your Google Cloud API key and the path to your audio file.
    let api_key = "YOUR_API_KEY";
    let audio_path = Path::new("path/to/your/audio/file.wav");

    // Read the audio data from the file.
    let mut file = File::open(&audio_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    // Encode the audio data as base64.
    let audio_content = base64::encode(data);

    // Create a new recognition request.
    let request_body = SpeechToTextRequest {
        audio: AudioContent {
            content: audio_content,
        },
        config: RecognitionConfig {
            encoding: "LINEAR16".to_string(),
            sample_rate_hertz: 16000,
            language_code: "en-US".to_string(),
        },
    };

    // Create a new HTTP client.
    let client = Client::new();

    // Send the recognition request to the API.
    let response_text = client
        .post(format!(
            "https://speech.googleapis.com/v1p1beta1/speech:recognize?key={}",
            api_key
        ))
        .json(&request_body)
        .send()
        .await?
        .text()
        .await?;

    // Deserialize the response into a SpeechToTextResponse struct.
    let response: SpeechToTextResponse = serde_json::from_str(&response_text)?;

    // Get the first transcription result.
    let result = response.results.first().unwrap();

    // Get the first transcription alternative.
    let alternative = result.alternatives.first().unwrap();

    // Print the transcription.
    println!("Transcription: {}", alternative.transcript);

    Ok(())
}
