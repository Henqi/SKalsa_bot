use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use chrono::{Datelike, Duration};
use reqwest::header::HeaderMap;
use reqwest::Client;

const API_URL: &str = "https://avoinna24.fi/api/slot";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36";

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum Weekday {
    Monday = 1,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone)]
struct CourtId {
    branch_id: String,
    group_id: String,
    product_id: String,
    user_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::builder().user_agent(USER_AGENT).build()?;
    println!("{:#?}", client);

    println!("Hakis:\n{}", check_hakis(&client).await?);
    println!("Delsu:\n{}", check_delsu(&client).await?);

    Ok(())
}

async fn get_slot_availability_data(
    client: &Client,
    booking: &CourtId,
    weekday: &Weekday,
) -> anyhow::Result<serde_json::Value> {
    let mut headers = HeaderMap::new();
    headers.insert("X-Subdomain", "arenacenter".parse()?);

    let request = client
        .get(API_URL)
        .query(&booking.query_parameters(weekday))
        .headers(headers);

    println!("Request:\n{:#?}", request);

    let response = request.send().await.context("Request failed")?;

    println!("{:#?}", response);

    if response.status().is_success() {
        //let body = response.text().await?;
        //println!("Response:\n{}", body);
        let json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse response json")?;
        //println!("Response:\n{json:#?}");
        Ok(json)
    } else {
        Err(anyhow!("Failed to fetch data: {}", response.status()))
    }
}

fn check_slot_availability(
    court_data: &serde_json::Value,
    day_as_string: &str,
    hour: &str,
) -> String {
    if court_data["data"].as_array().map_or(0, |v| v.len()) > 0 {
        for value in court_data["data"].as_array().unwrap() {
            let end_time = value["attributes"]["endtime"].as_str().unwrap_or("");
            if end_time.contains(hour) {
                return format!(
                    "Päivälle {} on vapaana vuoro joka loppuu tunnilla {}",
                    day_as_string, hour
                );
            }
        }
        format!(
            "Päivälle {} EI OLE vapaata vuoroa joka loppuu tunnilla {}",
            day_as_string, hour
        )
    } else {
        format!(
            "Päivälle {} ei löytynyt yhtään vapaata vuoroa / dataa ei löytynyt",
            day_as_string
        )
    }
}

async fn check_hakis(client: &Client) -> anyhow::Result<String> {
    let day = Weekday::Wednesday;
    let hour = "18".to_string();
    let hakis = CourtId::new(
        "2b325906-5b7a-11e9-8370-fa163e3c66dd",
        "a17ccc08-838a-11e9-8fd9-fa163e3c66dd",
        "59305e30-8b49-11e9-800b-fa163e3c66dd",
        "d7c92d04-807b-11e9-b480-fa163e3c66dd",
    );
    // println!("Hakis:\n{:#?}", hakis);
    let data = get_slot_availability_data(client, &hakis, &day).await?;
    let result = check_slot_availability(&data, &day.date_str(), &hour);
    Ok(result)
}

async fn check_delsu(client: &Client) -> anyhow::Result<String> {
    let day = Weekday::Tuesday;
    let hour = String::from("19");
    let delsu = CourtId::new(
        "2b325906-5b7a-11e9-8370-fa163e3c66dd",
        "a17ccc08-838a-11e9-8fd9-fa163e3c66dd",
        "59305e30-8b49-11e9-800b-fa163e3c66dd",
        "ea8b1cf4-807b-11e9-93b7-fa163e3c66dd",
    );
    // println!("Delsu:\n{:#?}", delsu);
    let data = get_slot_availability_data(client, &delsu, &day).await?;
    let result = check_slot_availability(&data, &day.date_str(), &hour);
    Ok(result)
}

impl Weekday {
    /// Convert from `Weekday` enum to Chrono `Weekday`.
    pub fn to_chrono(self) -> chrono::Weekday {
        match self {
            Weekday::Monday => chrono::Weekday::Mon,
            Weekday::Tuesday => chrono::Weekday::Tue,
            Weekday::Wednesday => chrono::Weekday::Wed,
            Weekday::Thursday => chrono::Weekday::Thu,
            Weekday::Friday => chrono::Weekday::Fri,
            Weekday::Saturday => chrono::Weekday::Sat,
            Weekday::Sunday => chrono::Weekday::Sun,
        }
    }

    pub fn date_str(&self) -> String {
        self.next_date().format("%Y-%m-%d").to_string()
    }

    /// Returns the next date for the given weekday.
    pub fn next_date(&self) -> DateTime<Utc> {
        let today = Utc::now();
        let current_weekday = today.weekday() as u32;
        let target_weekday = self.to_chrono() as u32;
        let days_until_target = if target_weekday == 0 {
            7
        } else {
            target_weekday
        };
        let days_diff = (days_until_target + 7 - current_weekday) % 7;
        today + Duration::days(days_diff as i64)
    }
}

impl CourtId {
    pub fn new(branch_id: &str, group_id: &str, product_id: &str, user_id: &str) -> Self {
        CourtId {
            branch_id: branch_id.to_string(),
            group_id: group_id.to_string(),
            product_id: product_id.to_string(),
            user_id: user_id.to_string(),
        }
    }

    pub fn query_parameters(&self, day: &Weekday) -> Vec<(String, String)> {
        let date = day.date_str();
        let query_params: Vec<(String, String)> = vec![
            ("filter[ismultibooking]".to_string(), "false".to_string()),
            ("filter[branch_id]".to_string(), self.branch_id.to_string()),
            ("filter[group_id]".to_string(), self.group_id.to_string()),
            (
                "filter[product_id]".to_string(),
                self.product_id.to_string(),
            ),
            ("filter[user_id]".to_string(), self.user_id.to_string()),
            ("filter[date]".to_string(), date.clone()),
            ("filter[start]".to_string(), date.clone()),
            ("filter[end]".to_string(), date),
        ];
        query_params
    }
}