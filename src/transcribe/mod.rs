use aws_sdk_transcribe::Client as TranscribeClient;
use aws_sdk_transcribe::types::{Media, LanguageCode, TranscriptionJobStatus};
use aws_sdk_s3::Client as S3Client;
use std::result::Result;
use std::time::Duration;
use tokio::time::sleep;
use crate::s3::download_file;
use crate::utils::parse_s3_uri;

pub async fn transcribe_audio(client: &TranscribeClient, media_file_uri: &str, output_bucket: &str, output_key: &str, job_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let media = Media::builder()
        .media_file_uri(media_file_uri)
        .build();
    let _response = client
        .start_transcription_job()
        .transcription_job_name(job_name)
        .language_code(LanguageCode::EnUs)
        .media(media)
        .output_bucket_name(output_bucket)
        .output_key(output_key)
        .send()
        .await?;
    Ok(())
}

pub async fn check_transcription_job_status(client: &TranscribeClient, s3_client: &S3Client, job_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let response = client
            .get_transcription_job()
            .transcription_job_name(job_name)
            .send()
            .await?;

        if let Some(job) = response.transcription_job() {
            println!("Transcription job status : {:?}", job.transcription_job_status());

            match job.transcription_job_status() {
                Some(status) if *status == TranscriptionJobStatus::Completed => {
                    println!("Transcription job completed successfully!");

                    if let Some(transcript_uri) = job.transcript().and_then(|t| t.transcript_file_uri()) {
                        println!("Transcript URI: {:?}", transcript_uri);

                        let (bucket, key) = parse_s3_uri(&transcript_uri)?;
                        println!("Bucket: {:?}", bucket);
                        println!("Key: {:?}", key);
                        let output_path = "transcript.json";

                        download_file(s3_client, bucket, key, &output_path).await?;
                    }
                    break;
                }
                Some(status) if *status == TranscriptionJobStatus::Failed => {
                    println!("Transcription job failed.");
                    break;
                }
                _ => {
                    println!("Transcription job still in progress...");
                }
            }
        } else {
            println!("Transcription job not found.");
            break;
        }
        sleep(Duration::from_secs(10)).await;
    }
    Ok(())
}
