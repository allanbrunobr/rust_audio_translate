use crate::s3::download_file;
use crate::utils::parse_s3_uri;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_transcribe::types::{LanguageCode, Media, TranscriptionJobStatus};
use aws_sdk_transcribe::Client as TranscribeClient;
use std::result::Result;
use std::time::Duration;
use tokio::time::sleep;

/// Initiates an audio transcription job using Amazon Transcribe.
///
/// This function starts a transcription job for the specified audio file and configures
/// the output location in an S3 bucket.
///
/// # Arguments
///
/// * `client` - A reference to the TranscribeClient used to start the transcription job.
/// * `media_file_uri` - The URI of the audio file to be transcribed.
/// * `output_bucket` - The name of the S3 bucket where the transcription output will be stored.
/// * `output_key` - The key (path) within the S3 bucket where the transcription output will be saved.
/// * `job_name` - A unique name for the transcription job.
///
/// # Returns
///
/// Returns `Ok(())` if the transcription job is successfully initiated, otherwise returns
/// an error wrapped in a `Box<dyn std::error::Error>`.
pub async fn transcribe_audio(
    client: &TranscribeClient,
    media_file_uri: &str,
    output_bucket: &str,
    output_key: &str,
    job_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let media = Media::builder().media_file_uri(media_file_uri).build();
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

/// Checks the status of a transcription job and downloads the transcript if completed.
///
/// This function continuously polls the status of a transcription job until it's either
/// completed or failed. If completed successfully, it downloads the transcript file from S3.
///
/// # Arguments
///
/// * `client` - A reference to the TranscribeClient used to check the job status.
/// * `s3_client` - A reference to the S3Client used to download the transcript file.
/// * `job_name` - The name of the transcription job to check.
///
/// # Returns
///
/// Returns `Ok(())` if the function completes without errors, otherwise returns an error wrapped in a `Box<dyn std::error::Error>`.
pub async fn check_transcription_job_status(
    client: &TranscribeClient,
    s3_client: &S3Client,
    job_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let response = client
            .get_transcription_job()
            .transcription_job_name(job_name)
            .send()
            .await?;

        if let Some(job) = response.transcription_job() {
            println!(
                "Transcription job status : {:?}",
                job.transcription_job_status()
            );

            match job.transcription_job_status() {
                Some(status) if *status == TranscriptionJobStatus::Completed => {
                    println!("Transcription job completed successfully!");

                    if let Some(transcript_uri) =
                        job.transcript().and_then(|t| t.transcript_file_uri())
                    {
                        println!("Transcript URI: {:?}", transcript_uri);

                        let (bucket, key) = parse_s3_uri(&transcript_uri)?;
                        println!("Bucket: {:?}", bucket);
                        println!("Key: {:?}", key);
                        let output_path = format!("src/transcripts/{}.json", job_name);

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
