use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionCalendar {
    pub weeks: Vec<Week>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Week {
    pub contribution_days: Vec<ContributionDay>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionDay {
    pub date: String,
    pub contribution_count: u32,
}
