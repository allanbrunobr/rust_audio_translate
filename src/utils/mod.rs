use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::result::Result;

pub fn generate_random_job_name() -> String {
    let mut rng = thread_rng();
    let job_name: String = (0..10).map(|_| rng.sample(Alphanumeric) as char).collect();
    job_name
}

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
