use std::cmp::Ordering;

use anyhow::Context;
use tracing::error;

use crate::{
    data::{PlanData, TotalPlan},
    error::AppError,
};

impl TotalPlan {
    pub fn dedup(&mut self) {
        self.data.sort_by(|a, b| a.subject.cmp(&b.subject));
        self.data.dedup_by(|a, b| b.eq_and_merge(a));
        self.data.sort_by(|a, b| a.cmp_custom(b));
    }
}

impl PlanData {
    pub fn eq_and_merge(&mut self, other: &mut PlanData) -> bool {
        if self == other {
            self.lesson = format!("{} & {}", self.lesson, other.lesson);
            return true;
        }
        false
    }

    pub fn cmp_custom(&self, other: &PlanData) -> Ordering {
        let date = compare_dates(&self.day, &other.day);
        match date {
            Ok(o) => {
                if o != Ordering::Equal {
                    return o;
                }
            }
            Err(e) => {
                e.print_error();
            }
        }

        return self.lesson.cmp(&other.lesson);
    }
}

fn compare_dates(left: &String, right: &String) -> Result<Ordering, AppError> {
    let left: Vec<_> = left
        .split(" ")
        .nth(1)
        .context("date format error")?
        .split(".")
        .collect();
    let right: Vec<_> = right
        .split(" ")
        .nth(1)
        .context("date format error")?
        .split(".")
        .collect();
    let year = left
        .get(2)
        .context("date format error")?
        .cmp(right.get(2).context("date format error")?);
    if year != Ordering::Equal {
        return Ok(year);
    }
    let month = left
        .get(1)
        .context("date format error")?
        .cmp(right.get(1).context("date format error")?);
    if month != Ordering::Equal {
        return Ok(month);
    }
    return Ok(left
        .get(0)
        .context("date format error")?
        .cmp(right.get(0).context("date format error")?));
}
