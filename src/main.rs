use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client as S3Client};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_transcribe::{Client as TranscribeClient};
use aws_sdk_transcribe::types::{Media, LanguageCode, TranscriptionJobStatus};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::result::Result;
use std::time::Duration;
use tokio::main;
use tokio::time::sleep;
use tokio::io::AsyncReadExt; // Importar o trait AsyncReadExt




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

    let random_job_name = generate_random_job_name();
    println!("Nome do trabalho de transcrição: {}", random_job_name);

    // Step 2: Send the transcription request to AWS Transcribe
    let media_file_uri = format!("s3://{}/{}", bucket, key);
    transcribe_audio(&transcribe_client, &media_file_uri, output_bucket, output_key, &random_job_name).await?;
    check_transcription_job_status(&transcribe_client, &s3_client, &random_job_name).await?;

    Ok(())
}

fn generate_random_job_name() -> String {
    let mut rng = thread_rng();
    let job_name: String = (0..10)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();
    job_name
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

async fn transcribe_audio(client: &TranscribeClient, media_file_uri: &str, output_bucket: &str, output_key: &str, job_name: &str)
                          -> Result<(), Box<dyn std::error::Error>> {
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
async fn check_transcription_job_status(
    client: &TranscribeClient,
    s3_client: &S3Client,
    job_name: &str
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
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

                    if let Some(transcript_uri) = job.transcript().and_then(|t| t.transcript_file_uri()) {
                        println!("Transcript URI: {:?}", transcript_uri);

                        // Extraia o bucket e a chave da URI
                        let (bucket, key) = parse_s3_uri(transcript_uri)?;
                        println!("Bucket: {:?}", bucket);
                        println!("Key: {:?}", key);
                        let output_path = "transcript.json"; // Substitua pelo caminho desejado

                        // Baixe e salve o arquivo
                        download_file(s3_client, bucket, key, &output_path).await?;
                    }
                    break
                }
                Some(status) if *status == TranscriptionJobStatus::Failed => {
                    println!("Transcription job failed.");
                    break
                }
                _ => {
                    println!("Transcription job still in progress...");
                }
            }
        } else {
            println!("Transcription job not found.");
            break
        }
        sleep(Duration::from_secs(10)).await;
    }
    Ok(())
}



async fn download_file( client: &S3Client, bucket: String, key: String, output_path: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;

        let mut file = File::create(output_path)?;
        let stream = response.body;

        let mut buffer = Vec::new();
        stream.into_async_read().read_to_end(&mut buffer).await?;
        file.write_all(&buffer)?;
        println!("Arquivo salvo em: {}", output_path);


    Ok(())
}

fn parse_s3_uri(uri: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let uri = uri.trim_start_matches("s3://").trim_start_matches("https://s3.us-east-1.amazonaws.com/");
    println!("URI: {}", uri);
    // Divida o URI em partes separadas por '/'
    let parts: Vec<&str> = uri.splitn(2, '/').collect();

    if parts.len() != 2 {
        return Err("Invalid S3 URI format".into());
    }

    let bucket = parts[0].to_string();
    let key = parts[1].to_string();

    Ok((bucket, key))
}


