use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;
use serde_json::Value;
use std::fs::File;
use std::io::{Read, Write};
use std::result::Result;
use tokio::io::AsyncReadExt;

pub async fn upload_to_s3(
    client: &S3Client,
    bucket: &str,
    key: &str,
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let byte_stream = ByteStream::from(buffer);

    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(byte_stream)
        .send()
        .await?;

    println!("File uploaded successfully to: {}/{}", bucket, key);
    Ok(())
}

pub async fn download_file(
    client: &S3Client,
    bucket: String,
    key: String,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.get_object().bucket(bucket).key(key).send().await?;

    let mut file = File::create(output_path)?;
    let stream = response.body;

    let mut buffer = Vec::new();
    stream.into_async_read().read_to_end(&mut buffer).await?;
    file.write_all(&buffer)?;
    println!("Arquivo salvo em: {}", output_path);

    Ok(())
}

pub async fn get_transcription_result(
    s3_client: &S3Client,
    bucket: &str,
    key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let response = s3_client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;

    let mut body = String::new();
    response
        .body
        .into_async_read()
        .read_to_string(&mut body)
        .await?;

    let v: Value = serde_json::from_str(&body)?;
    let transcription_text = v["results"]["transcripts"][0]["transcript"]
        .as_str()
        .ok_or("Transcription text not found")?
        .to_string();

    Ok(transcription_text)
}
