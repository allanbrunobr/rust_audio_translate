mod s3;
mod transcribe;
mod utils;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_transcribe::Client as TranscribeClient;
use s3::upload_to_s3;
use std::result::Result;
use tokio::main;
use transcribe::{check_transcription_job_status, transcribe_audio};
use utils::generate_random_job_name;

/// Uploads an audio file to an S3 bucket.
///
/// This function takes an `S3Client` instance, a `bucket` name, a `key` for the file, and a `file_path` to the audio file. It uploads the audio file to the specified S3 bucket and key.
///
/// # Arguments
///
/// * `s3_client` - An instance of the AWS S3 client.
/// * `bucket` - The name of the S3 bucket where the audio file will be uploaded.
/// * `key` - The key or filename for the audio file in the specified S3 bucket.
/// * `file_path` - The path to the audio file on the local filesystem.
///
/// # Returns
///
/// This function returns a `Result` type. On success, it returns `Ok(())`, indicating that the audio file has been successfully uploaded to the S3 bucket. If an error occurs during the upload process, it returns an `Err` variant containing the error.
///
/// # Examples
///
/// ```rust
/// use aws_sdk_s3::Client;
/// use s3::upload_to_s3;
///
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let s3_client = Client::new(shared_config);
///     let bucket = "audio-wav-rust";
///     let key = "test.wav";
///     let file_path = "/Users/bruno/RustroverProjects/rust/rust_audio_translate/src/audios/test.wav";
///     upload_to_s3(&s3_client, bucket, key, file_path).await?;
/// #     Ok(())
/// # }
/// ```
///
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
    transcribe_audio(
        &transcribe_client,
        &media_file_uri,
        output_bucket,
        output_key,
        &random_job_name,
    )
    .await?;
    check_transcription_job_status(&transcribe_client, &s3_client, &random_job_name).await?;

    Ok(())
}
