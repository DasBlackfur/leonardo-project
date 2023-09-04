use anyhow::Context;
use axum::{extract::Path, response::Html, routing::get, Json, Router};
use config::{PASSWORD, USERNAME};
use error::AppError;
use reqwest::Client;
use scraper::{Html as HtmlParser, Selector};
use serde::Serialize;
use tokio::time::Instant;
use std::net::SocketAddr;
use tracing::info;

mod config;
mod error;

#[derive(Debug, Serialize)]
struct TotalPlan {
    infos: Vec<PlanInfo>,
    data: Vec<PlanData>,
}

#[derive(Debug, Serialize)]
struct PlanInfo {
    day: String,
    info: String,
}

#[derive(Debug, Serialize)]
struct PlanData {
    day: String,
    class: String,
    lesson: String,
    subject: String,
    room: String,
    teachers: String,
    info: String,
    notes: String,
}

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

async fn get_plan_data(
    username: String,
    password: String,
    filter: String,
) -> Result<TotalPlan, AppError> {
    let client = Client::new();
    let mut data: Vec<PlanData> = Vec::new();
    let mut infos: Vec<PlanInfo> = Vec::new();
    let mut previous_uri = "".to_owned();
    let mut counter = 1;
    let mut done = false;

    while !done {
        let response_text = client
            .get(format!(include_str!("config/base_uri.txt"), counter))
            .basic_auth(&username, Some(&password))
            .send()
            .await?
            .text()
            .await?;
        let dom = HtmlParser::parse_document(&response_text);

        let current_uri = dom
            .select(&Selector::parse(".nav-right-button").unwrap())
            .next()
            .context("Nav Bar not found")?
            .value()
            .attr("onclick")
            .context("onclick attribute not found")?
            .to_owned();
        if current_uri == previous_uri {
            done = true;
        } else {
            previous_uri = current_uri;
            counter += 1;
        }

        let day = dom
            .select(&Selector::parse("h1").unwrap())
            .next()
            .context("day element not found")?
            .inner_html()
            .trim_matches('\n')
            .to_owned();

        let info = dom.select(&Selector::parse(".callout").unwrap()).next();
        if let Some(i) = info {
            infos.push(PlanInfo {
                day: day.clone(),
                info: i.inner_html().trim_matches('\n').to_owned(),
            });
        }

        let rows = dom
            .select(&Selector::parse("tbody tr").unwrap())
            .collect::<Vec<_>>();
        let mut previous_class: String = "".to_owned();
        for row in rows {
            let mut columns = row
                .select(&Selector::parse("td").unwrap())
                .map(|x| x.inner_html().trim_matches('\n').to_owned())
                .collect::<Vec<_>>();
            if columns[0] == "&nbsp;" {
                columns[0] = previous_class.clone();
            } else {
                previous_class = columns[0].clone();
            }
            if filter == columns[0].clone() || filter == "NOFILTER" {
                data.push(PlanData {
                    day: day.clone(),
                    class: columns
                        .get(0)
                        .context("class element not found")?
                        .to_owned(),
                    lesson: columns
                        .get(1)
                        .context("lesson element not found")?
                        .to_owned(),
                    subject: columns
                        .get(2)
                        .context("subject element not found")?
                        .to_owned(),
                    room: columns
                        .get(3)
                        .context("room element not found")?
                        .to_owned(),
                    teachers: columns
                        .get(4)
                        .context("teachers element not found")?
                        .to_owned(),
                    info: columns
                        .get(5)
                        .context("type element not found")?
                        .to_owned(),
                    notes: columns
                        .get(6)
                        .context("notes element not found")?
                        .to_owned(),
                });
            }
        }
    }

    Ok(TotalPlan { infos, data })
}

async fn get_hint() -> Html<String> {
    let start = Instant::now();
    let duration = start.elapsed().as_millis();
    info!("Request / took {}ms", duration);
    Html("Use /total or /get/classname".to_owned())
}
async fn get_total() -> Result<Json<TotalPlan>, AppError> {
    let start = Instant::now();
    let plan = get_plan_data(USERNAME.to_owned(), PASSWORD.to_owned(), "NOFILTER".to_owned()).await?;
    let duration = start.elapsed().as_millis();
    info!("Request /total took {}ms", duration);
    Ok(Json(plan))
}
async fn get_class(Path(class): Path<String>) -> Result<Json<TotalPlan>, AppError> {
    let start = Instant::now();
    let plan = get_plan_data(USERNAME.to_owned(), PASSWORD.to_owned(), class).await?;
    let duration = start.elapsed().as_millis();
    info!("Request /total took {}ms", duration);
    Ok(Json(plan))
}
