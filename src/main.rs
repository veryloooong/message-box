use axum::{
  extract::State,
  http::StatusCode,
  routing::{get, post},
  Json, Router,
};
use sqlx::{Executor, PgPool};

async fn hello_world() -> &'static str {
  "OK"
}

async fn receive_message(State(state): State<AppState>, Json(message): Json<Message>) -> StatusCode {
  match sqlx::query("INSERT INTO messages (msg) VALUES ($1)")
    .bind(&message.msg)
    .execute(&state.db)
    .await
  {
    Ok(_) => {
      println!("Message received: {}", message.msg);
      StatusCode::OK
    },
    Err(e) => {
      eprintln!("Error sending message: {}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    },
  }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Message {
  msg: String,
}

#[derive(Clone)]
pub struct AppState {
  db: PgPool,
}

#[shuttle_runtime::main]
async fn main(
  #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
  #[shuttle_shared_db::Postgres(local_uri = "{secrets.POSTGRES_LOCAL_URI}")] pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
  sqlx::migrate!("./migrations")
    .run(&pool)
    .await
    .expect("Failed to run migrations");

  let state = AppState { db: pool };

  let router = Router::new()
    .route("/", get(hello_world))
    .route("/send", post(receive_message))
    .with_state(state);

  Ok(router.into())
}
