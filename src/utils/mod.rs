use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::result::Result;

/// Generates a random job name consisting of 10 alphanumeric characters.
///
/// This function uses the `rand` crate to generate a random job name of length 10,
/// composed of alphanumeric characters.
///
/// # Arguments
///
/// This function takes no arguments.
///
/// # Return Value
///
/// The function returns a `String` containing a random job name.
///
/// # Examples
///
/// ```
/// use s3_utils::generate_random_job_name;
///
/// let job_name = generate_random_job_name();
/// assert_eq!(job_name.len(), 10);
/// assert!(job_name.chars().all(|c| c.is_alphanumeric()));
/// ```
pub fn generate_random_job_name() -> String {
    let mut rng = thread_rng();
    let job_name: String = (0..10).map(|_| rng.sample(Alphanumeric) as char).collect();
    job_name
}

/// Parses an S3 URI and returns the bucket and key as a tuple.
///
/// # Arguments
///
/// * `uri` - The S3 URI to parse.
///
/// # Returns
///
/// A `Result` containing a tuple of the parsed bucket and key. If the URI is not in the correct format, an error will be returned.
///
/// # Examples
///
/// ```
/// use s3_utils::parse_s3_uri;
///
/// let uri = "s3://my-bucket/my-key.txt";
/// let result = parse_s3_uri(uri).unwrap();
/// assert_eq!(result, ("my-bucket", "my-key.txt"));
/// ```
pub fn parse_s3_uri(uri: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let uri = uri
        .trim_start_matches("s3://")
        .trim_start_matches("https://s3.us-east-1.amazonaws.com/");
    let parts: Vec<&str> = uri.splitn(2, '/').collect();

    if parts.len() != 2 {
        return Err("Invalid S3 URI format!".into());
    }

    let bucket = parts[0].to_string();
    let key = parts[1].to_string();

    Ok((bucket, key))
}
