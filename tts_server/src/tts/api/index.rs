use actix_web::{web, HttpRequest, HttpResponse};
use std::sync::Arc;
use tokio::sync::RwLock;
use super::super::super::AppState;

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Successfully got index response", body = String)
    )
)]
#[actix_web::get("/")]
pub async fn index(data: web::Data<Arc<RwLock<AppState>>>, _req: HttpRequest) -> HttpResponse {
    let track_string = {
        let app_state = data.read().await;
        app_state.track.to_table_string() // Clone track output while holding lock
    };

    HttpResponse::Ok()
    .content_type("text/plain; charset=utf-8")
    .body(track_string)
}