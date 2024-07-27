mod s3;
mod transcribe;
mod utils;

use aws_config::meta::region::RegionProviderChain;
use aws_config::Region;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_transcribe::Client as TranscribeClient;
use std::result::Result;
use tokio::main;
use utils::generate_random_job_name;
use s3::upload_to_s3;
use transcribe::{transcribe_audio, check_transcription_job_status};

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up AWS S3 client
    let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let s3_client = S3Client::new(&shared_config);

    // Set up Transcribe client
    let transcribe_client = TranscribeClient::new(&shared_config);

    // File and bucket details
    let bucket = "audio-wav-rust";
    let key = "test.wav";
    let file_path = "/Users/bruno/RustroverProjects/rust/rust_audio_translate/src/audios/test.wav";
    let output_bucket = "t1bkt";
    let output_key = "my-output-files/";

    // Step 1: Upload the audio file to S3
    upload_to_s3(&s3_client, bucket, key, file_path).await?;

    let random_job_name = generate_random_job_name();
    println!("Nome do trabalho de transcrição: {}", random_job_name);

    // Step 2: Send the transcription request to AWS Transcribe
    let media_file_uri = format!("s3://{}/{}", bucket, key);
    transcribe_audio(&transcribe_client, &media_file_uri, output_bucket, output_key, &random_job_name).await?;
    check_transcription_job_status(&transcribe_client, &s3_client, &random_job_name).await?;

    Ok(())
}