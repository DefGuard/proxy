use crate::{grpc::enrollment::proto, handlers::ApiResult, server::AppState};
use axum::{routing::post, Json, Router, extract::State};
use serde::Deserialize;
use tracing::debug;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_enrollment_process))
        .route("/activate_user", post(activate_user))
        .route("/create_device", post(create_device))
}

#[derive(Deserialize)]
pub struct EnrollmentStartRequest {
    token: String,
}

pub async fn start_enrollment_process(
    State(state): State<AppState>,
    Json(req): Json<EnrollmentStartRequest>,
) -> ApiResult<()> {
    debug!("Starting enrollment process");

    let mut client = state.client.lock().await;
    let request = proto::EnrollmentStartRequest { token: req.token };
    client.start_enrollment(request).await?;

    Ok(())
}

pub async fn activate_user() {
    unimplemented!()
}

pub async fn create_device() {
    unimplemented!()
}
