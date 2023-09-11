use crate::data::{PlanData, TotalPlan};

impl TotalPlan {
    pub fn dedup(&mut self) {
        self.data.dedup_by(|a, b| b.eq_and_merge(a));
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
}
