extern crate rocket;

mod s3;
mod transcribe;
mod utils;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_transcribe::Client as TranscribeClient;
use rocket::form::Form;
use rocket::form::FromForm;
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use std::sync::Arc;
use tokio::sync::Mutex;
use transcribe::{check_transcription_job_status, transcribe_audio};
use utils::generate_random_job_name;

struct AppState {
    s3_client: Arc<S3Client>,
    transcribe_client: Arc<TranscribeClient>,
}

#[derive(FromForm)]
struct FileUpload<'r> {
    #[field(name = "files")]
    files: Vec<TempFile<'r>>,
}

#[rocket::post("/upload", data = "<form>")]
async fn upload_audio<'r>(
    form: Form<FileUpload<'r>>,
    state: &State<Arc<Mutex<AppState>>>,
) -> Result<status::Custom<Json<Vec<String>>>, status::Custom<String>> {
    let mut file_urls = Vec::new();
    let files = &mut form.into_inner().files;

    for (index, temp_file) in files.iter_mut().enumerate() {
        let file_path = format!("/tmp/audio_{}.wav", index);
        temp_file
            .persist_to(&file_path)
            .await
            .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

        let state = state.lock().await;
        let bucket = "audio-wav-rust";
        let key = format!("audio_{}.wav", index);

        // Upload to S3
        s3::upload_to_s3(&state.s3_client, bucket, &key, &file_path)
            .await
            .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

        let random_job_name = generate_random_job_name();
        let media_file_uri = format!("s3://{}/{}", bucket, key);

        // Transcribe
        transcribe_audio(
            &state.transcribe_client,
            &media_file_uri,
            bucket,
            "my-output-files/",
            &random_job_name,
        )
        .await
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

        check_transcription_job_status(
            &state.transcribe_client,
            &state.s3_client,
            &random_job_name,
        )
        .await
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

        file_urls.push(random_job_name);
    }

    Ok(status::Custom(Status::Ok, Json(file_urls)))
}

#[rocket::launch]
async fn rocket() -> _ {
    let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let s3_client = S3Client::new(&shared_config);
    let transcribe_client = TranscribeClient::new(&shared_config);

    let state = Arc::new(Mutex::new(AppState {
        s3_client: Arc::new(s3_client),
        transcribe_client: Arc::new(transcribe_client),
    }));

    rocket::build()
        .manage(state)
        .mount("/", rocket::routes![upload_audio])
}
