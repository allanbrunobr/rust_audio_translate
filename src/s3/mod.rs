use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;
use serde_json::Value;
use std::fs::File;
use std::io::{Read, Write};
use std::result::Result;
use tokio::io::AsyncReadExt;

/// Uploads a file to an S3 bucket.
///
/// This function reads a file from the local filesystem and uploads it to the specified S3 bucket.
///
/// # Arguments
///
/// * `client` - A reference to an S3Client used to interact with Amazon S3.
/// * `bucket` - The name of the S3 bucket to upload the file to.
/// * `key` - The object key (path) under which the file will be stored in the S3 bucket.
/// * `file_path` - The local file system path of the file to be uploaded.
///
/// # Returns
///
/// Returns `Ok(())` if the upload is successful, or an error wrapped in a `Box<dyn std::error::Error>`
/// if any step of the process fails (e.g., file reading, network issues, S3 errors).
///
/// # Errors
///
/// This function will return an error if:
/// - The file cannot be opened or read
/// - The S3 client fails to upload the file
/// - Any other I/O or AWS SDK error occurs
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

/// Downloads a file from an S3 bucket and saves it locally.
///
/// This function retrieves an object from the specified S3 bucket and saves it to the local filesystem.
///
/// # Arguments
///
/// * `client` - A reference to an S3Client used to interact with Amazon S3.
/// * `bucket` - The name of the S3 bucket from which to download the file.
/// * `key` - The object key (path) of the file in the S3 bucket.
/// * `output_path` - The local file system path where the downloaded file will be saved.
///
/// # Returns
///
/// Returns `Ok(())` if the download and save operations are successful, or an error wrapped in a
/// `Box<dyn std::error::Error>` if any step of the process fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The S3 client fails to retrieve the object
/// - The local file cannot be created or written to
/// - Any other I/O or AWS SDK error occurs
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

/// Retrieves and extracts the transcription result from an S3 object.
///
/// This function fetches a JSON object from an S3 bucket, parses it, and extracts
/// the transcription text from a specific location in the JSON structure.
///
/// # Arguments
///
/// * `s3_client` - A reference to an S3Client used to interact with Amazon S3.
/// * `bucket` - The name of the S3 bucket containing the transcription result.
/// * `key` - The object key (path) of the transcription result file in the S3 bucket.
///
/// # Returns
///
/// Returns a `Result` which, on success, contains a `String` with the extracted
/// transcription text. On failure, it returns a boxed error (`Box<dyn std::error::Error>`).
///
/// # Errors
///
/// This function will return an error if:
/// - The S3 object cannot be retrieved
/// - The object's content cannot be read or parsed as JSON
/// - The expected transcription text is not found in the parsed JSON structure
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
