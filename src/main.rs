extern crate rocket;

mod comprehend;
mod s3;
mod transcribe;
mod utils;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_comprehend::Client as ComprehendClient;
use aws_sdk_comprehendmedical::Client as ComprehendClientMedical;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_transcribe::Client as TranscribeClient;
use rocket::form::Form;
use rocket::form::FromForm;
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket_cors::AllowedMethods;
use rocket_cors::CorsOptions;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

struct AppState {
    s3_client: Arc<S3Client>,
    transcribe_client: Arc<TranscribeClient>,
    comprehend_client: Arc<ComprehendClient>,
    comprehend_client_medical: Arc<ComprehendClientMedical>,
}

#[derive(FromForm)]
struct FileUpload<'r> {
    #[field(name = "files")]
    files: Vec<TempFile<'r>>,
}

/// Handles the upload and processing of audio files.
///
/// This function receives audio files through a form submission, uploads them to S3,
/// transcribes the audio using AWS Transcribe, performs sentiment analysis on the
/// transcription, and returns the job names for the processed files.
///
/// # Parameters
///
/// * `form` - A `Form<FileUpload<'r>>` containing the uploaded audio files.
/// * `state` - A reference to the `State<Arc<Mutex<AppState>>>` containing shared
///             application state, including AWS client instances.
///
/// # Returns
///
/// A `Result` containing either:
/// - `Ok(status::Custom<Json<Vec<String>>>)` with a status code of 200 (OK) and a JSON
///   array of job names for the processed files.
/// - `Err(status::Custom<String>)` with a status code of 500 (Internal Server Error)
///   and an error message if any step in the process fails.
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

        let guard = state.lock().await;
        let bucket = "audio-wav-rust";
        let key = format!("audio_{}.wav", index);

        // Upload to S3
        s3::upload_to_s3(&guard.s3_client, bucket, &key, &file_path)
            .await
            .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

        let random_job_name = utils::generate_random_job_name();
        let media_file_uri = format!("s3://{}/{}", bucket, key);

        // Transcribe
        transcribe::transcribe_audio(
            &guard.transcribe_client,
            &media_file_uri,
            bucket,
            "my-output-files/",
            &random_job_name,
        )
        .await
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

        transcribe::check_transcription_job_status(
            &guard.transcribe_client,
            &guard.s3_client,
            &random_job_name,
        )
        .await
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

        // Get transcription result
        let transcription_text = s3::get_transcription_result(
            &guard.s3_client,
            bucket,
            &format!("my-output-files/{}.json", random_job_name),
        )
        .await
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

        comprehend::perform_sentiment_analysis(guard, &transcription_text)
            .await
            .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

        file_urls.push(random_job_name);
    }

    Ok(status::Custom(Status::Ok, Json(file_urls)))
}

/// Analyzes medical text using AWS Comprehend Medical to detect entities.
///
/// This function takes a medical text as input, sends it to AWS Comprehend Medical
/// for entity detection, and returns the detected entities or a message indicating
/// that no entities were found.
///
/// # Parameters
///
/// * `text` - A `String` containing the medical text to be analyzed.
/// * `state` - A reference to the `State` containing the shared `AppState`,
///             which includes the AWS Comprehend Medical client.
///
/// # Returns
///
/// A `Result` containing either:
/// - `Ok(status::Custom<String>)` with a status code of 200 (OK) and either:
///   - A string of formatted entities, each on a new line, if entities were found.
///   - The message "No entities found." if no entities were detected.
/// - `Err(status::Custom<String>)` with a status code of 500 (Internal Server Error)
///   and an error message if the analysis failed.
#[rocket::post("/analyze_medical_text", data = "<text>")]
async fn analyze_medical_text(
    text: String,
    state: &State<Arc<Mutex<AppState>>>,
) -> Result<status::Custom<String>, status::Custom<String>> {
    let guard = state.lock().await;
    let client = &guard.comprehend_client_medical;

    let result = client
        .detect_entities_v2()
        .text(text)
        .send()
        .await
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

        let entities = result.entities();
        let formatted_entities: Vec<String> = entities.iter().map(|entity| format!("{:?}", entity)).collect();

        if formatted_entities.is_empty() {
            Ok(status::Custom(Status::Ok, "No entities found.".to_string()))
        } else {
            Ok(status::Custom(Status::Ok, formatted_entities.join("\n")))
        }
}

/// Launches a Rocket web server with the specified routes and middleware.
///
/// # Purpose
/// This function initializes and configures a Rocket web server to handle audio file uploads,
/// transcription, sentiment analysis, and medical text analysis. It sets up AWS SDK clients for
/// S3, Transcribe, Comprehend, and Comprehend Medical services, and manages the application state
/// using an Arc and Mutex. It also applies CORS (Cross-Origin Resource Sharing) configuration.
///
/// # Parameters
/// None
///
/// # Return
/// A Rocket instance, which can be used to start the server.
#[rocket::launch]
async fn rocket() -> _ {
    // Set up AWS SDK clients with region provider and shared configuration
    let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let s3_client = S3Client::new(&shared_config);
    let transcribe_client = TranscribeClient::new(&shared_config);
    let comprehend_client = ComprehendClient::new(&shared_config);
    let comprehend_client_medical = ComprehendClientMedical::new(&shared_config);

    // Create a shared application state using Arc and Mutex
    let state = Arc::new(Mutex::new(AppState {
        s3_client: Arc::new(s3_client),
        transcribe_client: Arc::new(transcribe_client),
        comprehend_client: Arc::new(comprehend_client),
        comprehend_client_medical: Arc::new(comprehend_client_medical),
    }));

    // Configure CORS allowed methods
    let allowed_methods_list: AllowedMethods = ["Get", "Post", "Put", "Delete"]
    .iter()
    .map(|s| FromStr::from_str(s).unwrap())
    .collect();

    // Create CORS options and apply them
    let cors = CorsOptions {
        allowed_origins: rocket_cors::AllowedOrigins::All,
        allowed_methods: allowed_methods_list,
        allowed_headers: rocket_cors::AllowedHeaders::All,
        ..Default::default()
    }.to_cors().expect("Failed to create CORS");
    
    // Build and configure the Rocket instance
    rocket::build()
        .attach(cors)
        .manage(state)
        .mount("/", rocket::routes![upload_audio, analyze_medical_text])
}
