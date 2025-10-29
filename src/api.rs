use crate::models::ContributionCalendar;
use anyhow::{Context, Result};
use reqwest::Client;

pub async fn fetch_contributions(token: &str, username: &str) -> Result<ContributionCalendar> {
    let query = r#"
        query($username:String!) {
            user(login: $username) {
                contributionsCollection {
                    contributionCalendar {
                        totalContributions
                        weeks {
                            contributionDays {
                                date
                                contributionCount
                            }
                        }
                    }
                }
            }
        }
    "#;

    let body = serde_json::json!({
        "query": query,
        "variables": { "username": username }
    });

    let client = Client::new();
    let response: serde_json::Value = client
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .header("User-Agent", "trexanh")
        .json(&body)
        .send()
        .await
        .context("Failed to send request to GitHub API")?
        .json()
        .await
        .context("Failed to parse GitHub response as JSON")?;

    let calendar_value =
        response["data"]["user"]["contributionsCollection"]["contributionCalendar"].clone();
    if calendar_value.is_null() {
        anyhow::bail!("Missing contributionCalendar field in response");
    }
    let calendar: ContributionCalendar = serde_json::from_value(calendar_value)
        .context("Failed to deserialize contribution calendar")?;

    Ok(calendar)
}
