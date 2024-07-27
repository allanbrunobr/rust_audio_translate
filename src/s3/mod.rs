use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use std::fs::File;
use std::io::{Read, Write};
use std::result::Result;
use tokio::io::AsyncReadExt;

pub async fn upload_to_s3(client: &S3Client, bucket: &str, key: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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

    println!("File uploaded successfully to: {}/{}", bucket, key);
    Ok(())
}

pub async fn download_file(client: &S3Client, bucket: String, key: String, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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
