use axum::{extract::Path, response::Html, routing::get, Json, Router};
use std::net::SocketAddr;
use tokio::time::Instant;
use tracing::info;

use config::{PASSWORD, USERNAME};
use data::TotalPlan;
use error::AppError;

mod config;
mod data;
mod dedup;
mod error;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(get_hint))
        .route("/total", get(get_total))
        .route("/get/:class", get(get_class));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    info!("Starting server on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_hint() -> Html<String> {
    let start = Instant::now();
    let duration = start.elapsed().as_millis();
    info!("Request / took {}ms", duration);
    Html("Use /total or /get/classname".to_owned())
}
async fn get_total() -> Result<Json<TotalPlan>, AppError> {
    let start = Instant::now();
    let plan = TotalPlan::get_plan_data(
        USERNAME.to_owned(),
        PASSWORD.to_owned(),
        "NOFILTER".to_owned(),
    )
    .await?;
    let duration = start.elapsed().as_millis();
    info!("Request /total took {}ms", duration);
    Ok(Json(plan))
}

async fn get_class(Path(class): Path<String>) -> Result<Json<TotalPlan>, AppError> {
    let start = Instant::now();
    let plan =
        TotalPlan::get_plan_data(USERNAME.to_owned(), PASSWORD.to_owned(), class.clone()).await?;
    let duration = start.elapsed().as_millis();
    info!("Request /get/{} took {}ms", class, duration);
    Ok(Json(plan))
}
