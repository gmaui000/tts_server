use actix_web::*;
use actix_files as fs;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::api::{tts_handler, index};
use super::super::base::configuration::AppConfigItem;
use super::engine::tts_engine::TTSEngine;
use super::super::{AppState, QueryTracker};
use tracing::{self, info};
use chrono::{Local, Datelike, Timelike};

#[actix_web::main]
pub async fn start(config: &AppConfigItem) -> anyhow::Result<()> {

    let now = Local::now();
    let nowtime = format!("{:02}/{:02}/{:04} {:02}:{:02}:{:02}", now.month(), now.day(), now.year(), now.hour(), now.minute(), now.second());
    info!("tts_server start at {}.", nowtime);

    let app_state = web::Data::new(Arc::new(RwLock::new(AppState {
        engine: TTSEngine::new(),
        track: QueryTracker::new(nowtime),
    })));

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(tts_handler::api_tts)
            .service(index::index)
            .service(fs::Files::new("/demo", "demo"))
            .configure(init)
    })
    .bind((config.ip.clone(), config.port))?
    .run()
    .await?;
    Ok(())
}

fn init(_cfg: &mut web::ServiceConfig) {

}
