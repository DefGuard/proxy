use axum::{routing::post, Router};

pub fn router() -> Router {
    Router::new()
        .route("/start", post(start_enrollment_process))
        .route("/activate_user", post(activate_user))
        .route("/create_device", post(create_device))
}

pub async fn start_enrollment_process() {
    unimplemented!()
}

pub async fn activate_user() {
    unimplemented!()
}

pub async fn create_device() {
    unimplemented!()
}
