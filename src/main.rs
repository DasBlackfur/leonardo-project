use axum::{extract::Path, response::Html, routing::get, Json, Router};
use config::{PASSWORD, USERNAME};
use error::AppError;
use reqwest::Client;
use scraper::{Html as HtmlParser, Selector};
use serde::Serialize;
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
    filter: Option<String>,
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
            .unwrap()
            .value()
            .attr("onclick")
            .unwrap()
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
            .unwrap()
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
            if filter.clone().unwrap_or("NOFILTER".to_string()) == columns[0].clone() || filter.is_none(){
                data.push(PlanData {
                    day: day.clone(),
                    class: columns
                        .get(0)
                        .unwrap_or(&"CLASS ELEMENT NOT FOUND".to_owned())
                        .to_owned(),
                    lesson: columns
                        .get(1)
                        .unwrap_or(&"LESSON ELEMENT NOT FOUND".to_owned())
                        .to_owned(),
                    subject: columns
                        .get(2)
                        .unwrap_or(&"SUBJECT ELEMENT NOT FOUND".to_owned())
                        .to_owned(),
                    room: columns
                        .get(3)
                        .unwrap_or(&"ROOM ELEMENT NOT FOUND".to_owned())
                        .to_owned(),
                    teachers: columns
                        .get(4)
                        .unwrap_or(&"TEACHER ELEMENT NOT FOUND".to_owned())
                        .to_owned(),
                    info: columns
                        .get(5)
                        .unwrap_or(&"TYPE ELEMENT NOT FOUND".to_owned())
                        .to_owned(),
                    notes: columns
                        .get(6)
                        .unwrap_or(&"NOTES ELEMENT NOT FOUND".to_owned())
                        .to_owned(),
                });
            }
        }
    }

    Ok(TotalPlan { infos, data })
}

async fn get_hint() -> Html<String> {
    Html("Use /total or /get/<classname>".to_owned())
}
async fn get_total() -> Result<Json<TotalPlan>, AppError> {
    let plan = get_plan_data(USERNAME.to_owned(), PASSWORD.to_owned(), None).await?;
    Ok(Json(plan))
}
async fn get_class(Path(class): Path<String>) -> Result<Json<TotalPlan>, AppError> {
    let plan = get_plan_data(USERNAME.to_owned(), PASSWORD.to_owned(), Some(class)).await?;
    Ok(Json(plan))
}
