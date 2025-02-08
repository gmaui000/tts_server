use actix_web::{web, HttpResponse};
use tracing::{self, info};
use std::io::Cursor;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::super::super::AppState;
use chrono::Local;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct TTSQuery {
    /// 要合成语音的文本
    #[schema(example = "今天天气怎么样？明天大概有50%的概率下雨，请记得带伞。")]
    text: String,
}

#[utoipa::path(
    get,
    path = "/api/tts",
    responses(
        (status = 200, description = "Successfully got tts response", content_type = "audio/wav"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "TTS API"
)]
#[actix_web::get("/api/tts")]
pub async fn api_tts(data: web::Data<Arc<RwLock<AppState>>>, query: web::Query<TTSQuery>) -> HttpResponse {
    let start_time = Local::now();
    let text = query.text.clone();

    // Synthesize speech while holding the read lock only temporarily
    let wav = {
        let app_state = data.read().await;  // Acquire read lock
        app_state.engine.synthesis(&text, 0.2) // Call `synthesis` synchronously
    };

    let mut cursor = Cursor::new(Vec::new());
    let mut writer = hound::WavWriter::new(&mut cursor, hound::WavSpec{
        channels:   1,
        sample_rate: 24000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    }).expect("Failed to write sample to WAV.");
    for &sample in &wav {
        writer.write_sample(sample).expect("Failed to write sample to WAV.");
    }
    writer.finalize().unwrap();

    let duration = Local::now().signed_duration_since(start_time);
    // Write to track log
    {
        let mut app_state = data.write().await;  // Acquire write lock
        app_state.track.record_query(
            text.clone(),
            start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            std::time::Duration::from_millis(duration.num_milliseconds() as u64),
        );
    }
    info!("req: {:?} cost: {:.2}s", text, duration.num_milliseconds() as f64 / 1000.0);

    HttpResponse::Ok().content_type("audio/wav").body(cursor.into_inner())
}
