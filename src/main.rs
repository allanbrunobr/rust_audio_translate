use std;
use std::result::Result;
use std::fs::File;
use std::io::Read;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client as S3Client};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_transcribe::{Client as TranscribeClient};
use aws_sdk_transcribe::types::{Media, LanguageCode, TranscriptionJobStatus};
use tokio::main;
use tokio::time::sleep;
use std::time::Duration;



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
    let file_path = "/Users/bruno/RustroverProjects/rust/rust_audio_translate/src/test.wav";
    let output_bucket = "t1bkt";
    let output_key = "my-output-files/";

    // Step 1: Upload the audio file to S3
    upload_to_s3(&s3_client, bucket, key, file_path).await?;

    // Step 2: Send the transcription request to AWS Transcribe
    let media_file_uri = format!("s3://{}/{}", bucket, key);
    transcribe_audio(&transcribe_client, &media_file_uri, output_bucket, output_key).await?;
    check_transcription_job_status(&transcribe_client, "my-first-14transcription-job").await?;

    Ok(())
}

async fn upload_to_s3(client: &S3Client, bucket: &str, key: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let byte_stream = ByteStream::from(buffer);

    client.put_object()
        .bucket(bucket)
        .key(key)
        .body(byte_stream)
        .send()
        .await?;

    println!("File uploaded successfully to {}/{}", bucket, key);
    Ok(())
}

async fn transcribe_audio(client: &TranscribeClient, media_file_uri: &str, output_bucket: &str, output_key: &str)
                          -> Result<(), Box<dyn std::error::Error>> {
    let media = Media::builder()
        .media_file_uri(media_file_uri)
        .build();
    let response = client
        .start_transcription_job()
        .transcription_job_name("my-first-14transcription-job")
        .language_code(LanguageCode::EnUs)
        .media(media)
        .output_bucket_name(output_bucket)
        .output_key(output_key)
        .send()
        .await?;
    Ok(())
}

async fn check_transcription_job_status(client: &TranscribeClient, job_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = client
        .get_transcription_job()
        .transcription_job_name(job_name)
        .send()
        .await?;

    if let Some(job) = response.transcription_job() {
        println!("Transcription job status: {:?}", job.transcription_job_status());

        match job.transcription_job_status() {
            Some(status) if *status == TranscriptionJobStatus::Completed => {
                println!("Transcription job completed successfully!");
                // Você pode acessar o resultado da transcrição aqui
                if let Some(transcript_uri) = job.transcript().map(|t| t.transcript_file_uri()) {
                    println!("Transcript URI: {:?}", transcript_uri);
                }
            }
            Some(status) if *status == TranscriptionJobStatus::Failed => {
                println!("Transcription job failed.");
            }
            _ => {
                println!("Transcription job still in progress...");

            }
        }
    } else {
        println!("Transcription job not found.");
    }

    Ok(())
}
