use crate::{
    grpc::enrollment::proto::{
        ActivateUserRequest, CreateDeviceResponse, EnrollmentStartRequest, EnrollmentStartResponse,
        NewDevice,
    },
    handlers::ApiResult,
    server::AppState,
};
use axum::{extract::State, routing::post, Json, Router};
use tracing::debug;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_enrollment_process))
        .route("/activate_user", post(activate_user))
        .route("/create_device", post(create_device))
}

pub async fn start_enrollment_process(
    State(state): State<AppState>,
    Json(req): Json<EnrollmentStartRequest>,
) -> ApiResult<Json<EnrollmentStartResponse>> {
    debug!("Starting enrollment process");

    let mut client = state.client.lock().await;
    let response = client.start_enrollment(req).await?;

    Ok(Json(response.into_inner()))
}

pub async fn activate_user(
    State(state): State<AppState>,
    Json(req): Json<ActivateUserRequest>,
) -> ApiResult<()> {
    debug!("Activating user");

    let mut client = state.client.lock().await;
    client.activate_user(req).await?;

    Ok(())
}

pub async fn create_device(
    State(state): State<AppState>,
    Json(req): Json<NewDevice>,
) -> ApiResult<Json<CreateDeviceResponse>> {
    debug!("Adding new device");

    let mut client = state.client.lock().await;
    let response = client.create_device(req).await?;

    Ok(Json(response.into_inner()))
}
