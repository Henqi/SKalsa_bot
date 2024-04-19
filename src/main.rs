use std::fmt;
use std::str::FromStr;

use anyhow::{anyhow, Context};
use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, TimeZone, Timelike, Utc};
use clap::Parser;
use colored::{ColoredString, Colorize};
use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;
use strum::EnumString;

const API_URL: &str = "https://avoinna24.fi/api/slot";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_4_1) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4.1 Safari/605.1.15";

#[derive(Parser)]
#[command(author, about, version)]
struct Args {
    /// Optional court name to check
    court: Option<CourtName>,

    /// Weekday to check when specifying court name [1-7]
    #[arg(value_enum, short, long)]
    day: Option<Weekday>,

    /// Hour to check when specifying court name [00-23]
    #[arg(short, long, value_name = "HOUR")]
    time: Option<u32>,

    /// Print verbose information
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

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

#[derive(Debug, Clone, Copy, clap::ValueEnum, EnumString)]
enum CourtName {
    Hakis,
    Delsu,
}

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
    let args = Args::parse();
    let client = Client::builder().user_agent(USER_AGENT).build()?;

    let hakis = CourtId::new(
        "Hakis",
        "2b325906-5b7a-11e9-8370-fa163e3c66dd",
        "a17ccc08-838a-11e9-8fd9-fa163e3c66dd",
        "59305e30-8b49-11e9-800b-fa163e3c66dd",
        "d7c92d04-807b-11e9-b480-fa163e3c66dd",
    );

    let delsu = CourtId::new(
        "Delsu",
        "2b325906-5b7a-11e9-8370-fa163e3c66dd",
        "a17ccc08-838a-11e9-8fd9-fa163e3c66dd",
        "59305e30-8b49-11e9-800b-fa163e3c66dd",
        "ea8b1cf4-807b-11e9-93b7-fa163e3c66dd",
    );

    match args.court {
        Some(CourtName::Delsu) => {
            println!(
                "{}",
                check_court(
                    &client,
                    &delsu,
                    args.day.unwrap_or(Weekday::Tuesday),
                    args.time.unwrap_or(19),
                    args.verbose
                )
                .await?
            );
        }
        Some(CourtName::Hakis) => {
            println!(
                "{}",
                check_court(
                    &client,
                    &hakis,
                    args.day.unwrap_or(Weekday::Wednesday),
                    args.time.unwrap_or(18),
                    args.verbose
                )
                .await?
            );
        }
        None => {
            println!(
                "{}",
                check_court(&client, &delsu, Weekday::Tuesday, 19, args.verbose).await?
            );
            println!(
                "{}",
                check_court(&client, &hakis, Weekday::Wednesday, 18, args.verbose).await?
            );
        }
    }

    Ok(())
}

async fn check_court(
    client: &Client,
    court: &CourtId,
    day: Weekday,
    hour: u32,
    verbose: bool,
) -> anyhow::Result<ColoredString> {
    let data = get_slot_availability_data(client, court, &day).await?;
    let slots = extract_free_slots_from_response(data);

    if verbose {
        println!("{}", format!("Free slots for {}:", court.name).bold());
        for (index, slot) in slots.iter().enumerate() {
            println!("{index:>2}: {}", slot);
        }
    }

    let date_str = day.date_str();
    let utc_hour = local_hour_to_utc(hour)?;
    match check_slot_availability(&slots, utc_hour) {
        None => Ok(format!("Päivälle {date_str} ei löytynyt yhtään vapaata vuoroa").red()),
        Some(free) => {
            if free {
                Ok(
                    format!("Päivälle {date_str} on vapaana vuoro joka loppuu tunnilla {hour}")
                        .green(),
                )
            } else {
                Ok(
                    format!(
                        "Päivälle {date_str} EI OLE vapaata vuoroa joka loppuu tunnilla {hour}"
                    )
                    .yellow(),
                )
            }
        }
    }
}

async fn get_slot_availability_data(
    client: &Client,
    court: &CourtId,
    weekday: &Weekday,
) -> anyhow::Result<ApiResponse> {
    let mut headers = HeaderMap::new();
    headers.insert("X-Subdomain", "arenacenter".parse()?);

    let response = client
        .get(API_URL)
        .query(&court.query_parameters(weekday))
        .headers(headers)
        .send()
        .await
        .context("Request failed")?;

    if response.status().is_success() {
        let api_response: ApiResponse = response.json().await?;
        Ok(api_response)
    } else {
        Err(anyhow!("Failed to fetch data: {}", response.status()))
    }
}

fn check_slot_availability(court_data: &[Slot], utc_hour: u32) -> Option<bool> {
    if !court_data.is_empty() {
        for slot in court_data.iter() {
            // TODO: better availability check
            if slot.end_time.hour() == utc_hour {
                return Some(true);
            }
        }
        Some(false)
    } else {
        None
    }
}

fn extract_free_slots_from_response(api_response: ApiResponse) -> Vec<Slot> {
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

fn local_hour_to_utc(hour: u32) -> anyhow::Result<u32> {
    let naive_time = NaiveTime::from_hms_opt(hour, 0, 0)
        .ok_or_else(|| anyhow!("Invalid hour provided: {hour}"))
        .context("Failed to create time from given hour")?;

    let local_date = Local::now().date_naive();
    let naive_datetime = local_date.and_time(naive_time);

    let local_datetime = Local
        .from_local_datetime(&naive_datetime)
        .single()
        .ok_or_else(|| anyhow!("Failed to convert naive datetime to local datetime"))?;

    let utc_datetime = local_datetime.with_timezone(&Utc);
    Ok(utc_datetime.hour())
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
            self.start_time.with_timezone(&Local).hour(),
            self.start_time.minute(),
            self.end_time.with_timezone(&Local).hour(),
            self.end_time.minute()
        )
    }
}

impl FromStr for Weekday {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Weekday::Monday),
            "2" => Ok(Weekday::Tuesday),
            "3" => Ok(Weekday::Wednesday),
            "4" => Ok(Weekday::Thursday),
            "5" => Ok(Weekday::Friday),
            "6" => Ok(Weekday::Saturday),
            "7" => Ok(Weekday::Sunday),
            _ => Err("Invalid day number"),
        }
    }
}

impl TryFrom<i32> for Weekday {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Weekday::Monday),
            2 => Ok(Weekday::Tuesday),
            3 => Ok(Weekday::Wednesday),
            4 => Ok(Weekday::Thursday),
            5 => Ok(Weekday::Friday),
            6 => Ok(Weekday::Saturday),
            7 => Ok(Weekday::Sunday),
            _ => Err("Invalid day of the week"),
        }
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
