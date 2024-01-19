use anyhow::Context;
use derivative::Derivative;
use reqwest::Client;
use scraper::{Html as HtmlParser, Selector};
use serde::Serialize;

use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct TotalPlan {
    pub infos: Vec<PlanInfo>,
    pub data: Vec<PlanData>,
}

#[derive(Debug, Serialize)]
pub struct PlanInfo {
    pub day: String,
    pub info: String,
}

#[derive(Debug, Serialize, Derivative, Clone)]
#[derivative(PartialEq)]
pub struct PlanData {
    pub day: String,
    pub class: String,
    #[derivative(PartialEq = "ignore")]
    pub lesson: String,
    pub subject: String,
    pub room: String,
    pub teachers: String,
    pub info: String,
    pub notes: String,
}

impl TotalPlan {
    pub async fn get_plan_data(
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
            let mut previous_lesson: String = "".to_owned();
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
                if columns[1] == "&nbsp;" {
                    columns[1] = previous_lesson.clone();
                } else {
                    previous_lesson = columns[1].clone();
                }

                if filter == columns[0].clone() || filter == "NOFILTER" {
                    data.push(PlanData {
                        day: day.clone(),
                        class: columns
                            .first()
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
                        room: columns.get(3).context("room element not found")?.to_owned(),
                        teachers: columns
                            .get(4)
                            .context("teachers element not found")?
                            .to_owned(),
                        info: columns.get(5).context("type element not found")?.to_owned(),
                        notes: columns
                            .get(6)
                            .context("notes element not found")?
                            .to_owned(),
                    });
                }
            }
        }

        let mut plan = TotalPlan { infos, data };
        plan.dedup();

        Ok(plan)
    }
}
