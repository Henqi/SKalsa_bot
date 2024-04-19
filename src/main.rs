use std::fmt;

use anyhow::{anyhow, Context};
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

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

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct CourtId {
    name: String,
    branch_id: String,
    group_id: String,
    product_id: String,
    user_id: String,
}

#[derive(Debug, Eq, PartialEq, Deserialize)]
struct ApiResponse {
    data: Vec<DataItem>,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Deserialize)]
struct DataItem {
    id: Option<String>,
    #[serde(rename = "type")]
    data_type: String,
    attributes: Option<Attributes>,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Deserialize)]
struct Attributes {
    #[serde(skip_serializing_if = "Option::is_none")]
    product_id: Option<String>,
    #[serde(rename = "starttime")]
    start_time: DateTime<Utc>,
    #[serde(rename = "endtime")]
    end_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct Slot {
    id: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::builder().user_agent(USER_AGENT).build()?;
    //println!("{:#?}", client);

    println!("Hakis:\n{}", check_hakis(&client).await?);
    println!("Delsu:\n{}", check_delsu(&client).await?);

    Ok(())
}

async fn check_hakis(client: &Client) -> anyhow::Result<String> {
    let day = Weekday::Wednesday;
    let hour: u32 = 18;
    let hakis = CourtId::new(
        "Hakis",
        "2b325906-5b7a-11e9-8370-fa163e3c66dd",
        "a17ccc08-838a-11e9-8fd9-fa163e3c66dd",
        "59305e30-8b49-11e9-800b-fa163e3c66dd",
        "d7c92d04-807b-11e9-b480-fa163e3c66dd",
    );
    // println!("Hakis:\n{:#?}", hakis);
    let data = get_slot_availability_data(client, &hakis, &day).await?;
    let slots = extract_slots_from_response(data);
    let result = check_slot_availability(&slots, &day.date_str(), hour);
    Ok(result)
}

async fn check_delsu(client: &Client) -> anyhow::Result<String> {
    let day = Weekday::Tuesday;
    let hour: u32 = 19;
    let delsu = CourtId::new(
        "Delsu",
        "2b325906-5b7a-11e9-8370-fa163e3c66dd",
        "a17ccc08-838a-11e9-8fd9-fa163e3c66dd",
        "59305e30-8b49-11e9-800b-fa163e3c66dd",
        "ea8b1cf4-807b-11e9-93b7-fa163e3c66dd",
    );
    // println!("Delsu:\n{:#?}", delsu);
    let data = get_slot_availability_data(client, &delsu, &day).await?;
    let slots = extract_slots_from_response(data);
    let result = check_slot_availability(&slots, &day.date_str(), hour);
    Ok(result)
}

async fn get_slot_availability_data(
    client: &Client,
    court: &CourtId,
    weekday: &Weekday,
) -> anyhow::Result<ApiResponse> {
    let mut headers = HeaderMap::new();
    headers.insert("X-Subdomain", "arenacenter".parse()?);

    let request = client
        .get(API_URL)
        .query(&court.query_parameters(weekday))
        .headers(headers);

    //println!("Request:\n{:#?}", request);

    let response = request.send().await.context("Request failed")?;

    //println!("{:#?}", response);

    if response.status().is_success() {
        //let body = response.text().await?;
        //println!("Response:\n{}", body);
        //let json: serde_json::Value = response
        //    .json()
        //    .await
        //    .context("Failed to parse response json")?;
        //println!("Response:\n{json:#?}");
        let api_response: ApiResponse = response.json().await?;
        Ok(api_response)
    } else {
        Err(anyhow!("Failed to fetch data: {}", response.status()))
    }
}

fn check_slot_availability(court_data: &[Slot], day_as_string: &str, hour: u32) -> String {
    if !court_data.is_empty() {
        for (index, slot) in court_data.iter().enumerate() {
            println!("{index:>2}: {:}", slot);
            // TODO: better availability check
            if slot.end_time.hour() == hour {
                return format!(
                    "Päivälle {day_as_string} on vapaana vuoro joka loppuu tunnilla {hour}"
                );
            }
        }
        format!("Päivälle {day_as_string} EI OLE vapaata vuoroa joka loppuu tunnilla {hour}")
    } else {
        format!("Päivälle {day_as_string} ei löytynyt yhtään vapaata vuoroa / dataa ei löytynyt",)
    }
}

fn extract_slots_from_response(api_response: ApiResponse) -> Vec<Slot> {
    api_response
        .data
        .into_iter()
        .filter(|item| item.data_type == "slot" && item.attributes.is_some())
        .map(|item| {
            let attributes = item.attributes.unwrap();
            Slot {
                id: attributes.product_id.unwrap_or_default(),
                start_time: attributes.start_time,
                end_time: attributes.end_time,
            }
        })
        .collect()
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
    pub fn new(
        name: &str,
        branch_id: &str,
        group_id: &str,
        product_id: &str,
        user_id: &str,
    ) -> Self {
        CourtId {
            name: name.to_string(),
            branch_id: branch_id.to_string(),
            group_id: group_id.to_string(),
            product_id: product_id.to_string(),
            user_id: user_id.to_string(),
        }
    }

    pub fn query_parameters(&self, day: &Weekday) -> Vec<(&'static str, String)> {
        let date = day.date_str();
        vec![
            ("filter[ismultibooking]", "false".to_string()),
            ("filter[branch_id]", self.branch_id.clone()),
            ("filter[group_id]", self.group_id.clone()),
            ("filter[product_id]", self.product_id.clone()),
            ("filter[user_id]", self.user_id.clone()),
            ("filter[date]", date.clone()),
            ("filter[start]", date.clone()),
            ("filter[end]", date),
        ]
    }
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Slot {} {} {:02}:{:02} - {:02}:{:02}",
            self.start_time.format("%Y-%m-%d"),
            self.start_time.weekday(),
            self.start_time.hour(),
            self.start_time.minute(),
            self.end_time.hour(),
            self.end_time.minute()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, NaiveDateTime};

    #[test]
    fn test_to_chrono() {
        assert_eq!(Weekday::Monday.to_chrono(), chrono::Weekday::Mon);
        assert_eq!(Weekday::Tuesday.to_chrono(), chrono::Weekday::Tue);
        assert_eq!(Weekday::Wednesday.to_chrono(), chrono::Weekday::Wed);
        assert_eq!(Weekday::Thursday.to_chrono(), chrono::Weekday::Thu);
        assert_eq!(Weekday::Friday.to_chrono(), chrono::Weekday::Fri);
        assert_eq!(Weekday::Saturday.to_chrono(), chrono::Weekday::Sat);
        assert_eq!(Weekday::Sunday.to_chrono(), chrono::Weekday::Sun);
    }

    #[test]
    fn test_next_date() {
        let test_day = Weekday::Monday;
        let next_monday = test_day.next_date();
        assert_eq!(next_monday.weekday(), chrono::Weekday::Mon);
        assert!(next_monday >= Utc::now());
    }

    #[test]
    fn test_date_str_format() {
        let test_day = Weekday::Friday;
        let date_str = test_day.date_str();
        // Example test to ensure format is "YYYY-MM-DD"
        assert!(date_str.chars().nth(4) == Some('-') && date_str.chars().nth(7) == Some('-'));
        assert_eq!(date_str.len(), 10);
    }

    #[test]
    fn test_deserialization() {
        let json_data = r#"
        {
            "data": [
                {
                    "id": null,
                    "type": "slot",
                    "attributes": {
                        "product_id": "59305e30-8b49-11e9-800b-fa163e3c66dd",
                        "starttime": "2024-04-24T06:00:00Z",
                        "endtime": "2024-04-24T07:00:00Z"
                    },
                    "relationships": null,
                    "links": {
                        "self_link": "/slot/"
                    },
                    "meta": null
                }
            ],
            "meta": null,
            "included": null
        }
        "#;

        let parsed_data: ApiResponse = serde_json::from_str(json_data).unwrap();

        let expected_data = ApiResponse {
            data: vec![DataItem {
                id: None,
                data_type: String::from("slot"),
                attributes: Option::from(Attributes {
                    product_id: Option::from(String::from("59305e30-8b49-11e9-800b-fa163e3c66dd")),
                    start_time: NaiveDateTime::parse_from_str(
                        "2024-04-24T06:00:00Z",
                        "%Y-%m-%dT%H:%M:%SZ",
                    )
                    .unwrap()
                    .and_utc(),
                    end_time: NaiveDateTime::parse_from_str(
                        "2024-04-24T07:00:00Z",
                        "%Y-%m-%dT%H:%M:%SZ",
                    )
                    .unwrap()
                    .and_utc(),
                }),
            }],
        };

        assert_eq!(parsed_data, expected_data);
    }
}
