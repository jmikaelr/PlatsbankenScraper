#![allow(unused_variables)]
#![allow(dead_code)]

use crate::constants::{
    MAX_RECORDS, PAGE_SIZE, SOURCE_EXTERNAL, SOURCE_PB, URL_JOB_ADS, URL_SEARCH,
};
use crate::database::DbJobAd;
use chrono::{Duration, NaiveDateTime, Utc};
use log::{error, info};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use serde_json::Value;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};

use crate::database;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct JobAd {
    pub id: Option<String>,
    pub title: Option<String>,
    pub occupation: Option<String>,
    pub workplace: Option<String>,
    #[serde(rename = "workplaceName")]
    pub workplace_name: Option<String>,
    #[serde(rename = "publishedDate")]
    pub published_date: Option<String>,
    #[serde(rename = "lastApplicationDate")]
    pub last_application_date: Option<String>,
    #[serde(skip_deserializing)]
    pub source: Option<String>,
    #[serde(skip_deserializing)]
    pub url: Option<String>,
    #[serde(rename = "sourceLinks")]
    pub source_links: Option<Vec<SourceLinks>>,
    #[serde(skip_deserializing)]
    pub created_at: Option<String>,
}

impl From<JobAd> for DbJobAd {
    fn from(job_ad: JobAd) -> Self {
        let url = job_ad
            .source_links
            .as_ref()
            .and_then(|links| links.first().map(|link| link.url.clone()));

        DbJobAd {
            id: job_ad.id,
            title: job_ad.title,
            occupation: job_ad.occupation,
            workplace: job_ad.workplace,
            workplace_name: job_ad.workplace_name,
            published_date: job_ad.published_date,
            last_application_date: job_ad.last_application_date,
            source: job_ad.source,
            url,
            created_at: job_ad.created_at,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SourceLinks {
    pub label: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobResponse {
    pub ads: Vec<JobAd>,
    #[serde(rename = "numberOfAds")]
    pub number_of_ads: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DbJobResponse {
    pub ads: Vec<DbJobAd>,
}

impl From<JobResponse> for DbJobResponse {
    fn from(job_response: JobResponse) -> Self {
        let mut db_job_response = DbJobResponse { ads: Vec::new() };
        for job_ad in job_response.ads.iter() {
            let url = job_ad
                .source_links
                .as_ref()
                .and_then(|links| links.first().map(|link| link.url.clone()));

            let db_job_ad = DbJobAd {
                id: job_ad.id.clone(),
                title: job_ad.title.clone(),
                occupation: job_ad.occupation.clone(),
                workplace: job_ad.workplace.clone(),
                workplace_name: job_ad.workplace_name.clone(),
                published_date: job_ad.published_date.clone(),
                last_application_date: job_ad.last_application_date.clone(),
                source: job_ad.source.clone(),
                url,
                created_at: job_ad.created_at.clone(),
            };
            db_job_response.ads.push(db_job_ad);
        }
        db_job_response
    }
}

#[derive(PartialEq, Clone)]
pub enum SearchDuration {
    Daily,
    TwoDays,
    Weekly,
    Monthly,
    Max,
}

impl SearchDuration {
    pub fn to_days(&self) -> i64 {
        match self {
            SearchDuration::Daily => 1,
            SearchDuration::TwoDays => 2,
            SearchDuration::Weekly => 7,
            SearchDuration::Monthly => 30,
            SearchDuration::Max => 0,
        }
    }
}

#[derive(Clone, Debug, EnumString, EnumIter, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Region {
    Blekinge,
    Dalarna,
    Gotland,
    Gävleborg,
    Halland,
    Jämtland,
    Jönköping,
    Kalmar,
    Kronoberg,
    Norrbotten,
    Skåne,
    Stockholm,
    Södermanland,
    Uppsala,
    Värmland,
    Västerbotten,
    Västernorrland,
    Västmanland,
    VästraGötaland,
    Örebro,
    Östergötland,
}

impl Region {
    fn get_value(&self) -> &'static str {
        match self {
            Region::Blekinge => "DQZd_uYs_oKb",
            Region::Dalarna => "oDpK_oZ2_WYt",
            Region::Gotland => "K8iD_VQv_2BA",
            Region::Gävleborg => "zupA_8Nt_xcD",
            Region::Halland => "wjee_qH2_yb6",
            Region::Jämtland => "65Ms_7r1_RTG",
            Region::Jönköping => "MtbE_xWT_eMi",
            Region::Kalmar => "9QUH_2bb_6Np",
            Region::Kronoberg => "tF3y_MF9_h5G",
            Region::Norrbotten => "9hXe_F4g_eTG",
            Region::Skåne => "CaRE_1nn_cSU",
            Region::Stockholm => "CifL_Rzy_Mku",
            Region::Södermanland => "s93u_BEb_sx2",
            Region::Uppsala => "zBon_eET_fFU",
            Region::Värmland => "EVVp_h6U_GSZ",
            Region::Västerbotten => "g5Tt_CAV_zBd",
            Region::Västernorrland => "NvUF_SP1_1zo",
            Region::Västmanland => "G6DV_fKE_Viz",
            Region::VästraGötaland => "zdoY_6u5_Krt",
            Region::Örebro => "xTCk_nT5_Zjm",
            Region::Östergötland => "oLT3_Q9p_3nn",
        }
    }
}

#[derive(Clone, Debug, EnumString, EnumIter, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OccupationType {
    AdministrationEkonomiJuridik,
    ByggAnläggning,
    CheferVerksamhetsledare,
    DataIT,
    FörsäljningInköpMarknadsföring,
    Hantverksyrken,
    HotellRestaurangStorhushåll,
    HälsoSjukvård,
    IndustriellTillverkning,
    InstallationDriftUnderhåll,
    KroppsSkönhetsvård,
    KulturMediaDesign,
    Militär,
    Naturbruk,
    Naturvetenskapligt,
    Pedagogiskt,
    SaneringRenhållning,
    Socialt,
    Säkerhet,
    Teknisk,
    Transport,
}

impl OccupationType {
    pub fn as_readable_string(&self) -> String {
        let name = format!("{:?}", self);
        if name.starts_with("Data") {
            return "Data/IT".to_string();
        }
        name.chars().fold(String::new(), |mut acc, c| {
            if c.is_uppercase() && !acc.is_empty() {
                acc.push(' ');
            }
            acc.push(c);
            acc
        })
    }
}

fn setup_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("accept"),
        HeaderValue::from_static("application/json, text/plain, */*"),
    );

    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/json"),
    );

    headers.insert(
        HeaderName::from_static("user-agent"),
        HeaderValue::from_static("reqwest/0.11"),
    );
    headers
}

pub async fn get_jobs_pb(duration: &SearchDuration) -> Result<(), Box<dyn std::error::Error>> {
    // Searches all the variants of the Enum Region to make it possible to fetch all jobs
    // Main function to retrieve all jobs
    // Platsbanken
    let client = Client::new();

    let headers = setup_headers();
    let from_date: Value;
    if duration.to_days() != 0 {
        let date = Utc::now() - Duration::days(duration.to_days());
        from_date = Value::String(date.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string());
    } else {
        from_date = Value::Null;
    }
    let to_date = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S.%3fZ")
        .to_string();

    let mut vec_res: Vec<JobAd> = Vec::new();
    for region in Region::iter() {
        let mut start_records = 0;
        while start_records < MAX_RECORDS {
            let payload = serde_json::json!({
                "filters": [
                    {
                        "type": "region",
                        "value": region.get_value()
                    }],
                "fromDate": from_date,
                "order": "date",
                "maxRecords": PAGE_SIZE,
                "startIndex": start_records,
                "toDate": to_date,
                "source": SOURCE_PB.to_string(),
            });
            let mut res: JobResponse = client
                .post(URL_SEARCH)
                .headers(headers.clone())
                .json(&payload)
                .send()
                .await?
                .json()
                .await?;

            if res.ads.is_empty() {
                break;
            }
            for job in res.ads.drain(..) {
                let mut job = job;
                job.published_date = job.published_date.as_ref().map(|date| {
                    NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3fZ")
                        .map(|dt| dt.date().to_string())
                        .unwrap_or_else(|_| "Invalid date".to_string())
                });
                job.last_application_date = job.last_application_date.as_ref().map(|date| {
                    NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3fZ")
                        .map(|dt| dt.date().to_string())
                        .unwrap_or_else(|_| "Invalid date".to_string())
                });

                job.source = Some("Platsbanken".to_string());
                job.url = match &job.id {
                    Some(id) => Some(format!("{}{}", URL_JOB_ADS, id)),
                    None => None,
                };

                job.created_at = Some(Utc::now().format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string());
                vec_res.push(job);
            }
            start_records += PAGE_SIZE;
        }
    }

    database::insert_jobs(vec_res).await?;
    Ok(())
}

pub async fn get_jobs_external(
    duration: &SearchDuration,
) -> Result<(), Box<dyn std::error::Error>> {
    // Searches all the variants of the Enum Region to make it possible to fetch all jobs
    // Main function to retrieve all jobs
    // External
    let client = Client::new();

    let headers = setup_headers();
    let from_date: Value;
    if duration.to_days() != 0 {
        let date = Utc::now() - Duration::days(duration.to_days());
        from_date = Value::String(date.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string());
    } else {
        from_date = Value::Null;
    }
    let to_date = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S.%3fZ")
        .to_string();

    let mut vec_res: Vec<JobAd> = Vec::new();
    for region in Region::iter() {
        let mut start_records = 0;
        while start_records < MAX_RECORDS {
            let payload = serde_json::json!({
                "filters": [
                    {
                        "type": "region",
                        "value": region.get_value()
                    }],
                "fromDate": from_date,
                "order": "date",
                "maxRecords": PAGE_SIZE,
                "startIndex": start_records,
                "toDate": to_date,
                "source": SOURCE_EXTERNAL.to_string(),
            });
            let mut res: JobResponse = client
                .post(URL_SEARCH)
                .headers(headers.clone())
                .json(&payload)
                .send()
                .await?
                .json()
                .await?;

            if res.ads.is_empty() {
                break;
            }
            for job in res.ads.drain(..) {
                let mut job = job;
                job.published_date = job.published_date.as_ref().map(|date| {
                    NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3fZ")
                        .map(|dt| dt.date().to_string())
                        .unwrap_or_else(|_| "Invalid date".to_string())
                });
                job.last_application_date = job.last_application_date.as_ref().map(|date| {
                    NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3fZ")
                        .map(|dt| dt.date().to_string())
                        .unwrap_or_else(|_| "Invalid date".to_string())
                });

                job.source_links
                    .as_ref()
                    .and_then(|links| links.first())
                    .map(|link| link.url.clone());

                job.created_at = Some(Utc::now().format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string());
                vec_res.push(job);
            }
            start_records += PAGE_SIZE;
        }
    }

    database::insert_jobs(vec_res).await?;
    Ok(())
}

pub async fn get_jobs_with_title(
    duration: &SearchDuration,
    title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let headers = setup_headers();
    let from_date = serde_json::Value::Null;
    if duration.to_days() != 0 {
        let from_date = Utc::now() - Duration::days(duration.to_days());
        let from_date = from_date.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
    }
    let to_date = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S.%3fZ")
        .to_string();

    let mut vec_res: Vec<JobAd> = Vec::new();
    let job_sources = vec![SOURCE_PB.to_string(), SOURCE_EXTERNAL.to_string()];
    for source in job_sources.iter() {
        let mut start_records = 0;
        while start_records < MAX_RECORDS {
            let payload = serde_json::json!({
                "filters": [{
                    "type": "freetext",
                    "value": title.to_string()}],
                "fromDate": from_date,
                "order": "date",
                "maxRecords": PAGE_SIZE,
                "startIndex": start_records,
                "toDate": to_date,
                "source": source,
            });
            let mut res: JobResponse = client
                .post(URL_SEARCH)
                .headers(headers.clone())
                .json(&payload)
                .send()
                .await?
                .json()
                .await?;

            for job in res.ads.drain(..) {
                let mut job = job;
                job.published_date = job.published_date.as_ref().map(|date| {
                    NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3fZ")
                        .map(|dt| dt.date().to_string())
                        .unwrap_or_else(|_| "Invalid date".to_string())
                });

                job.last_application_date = job.last_application_date.as_ref().map(|date| {
                    NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3fZ")
                        .map(|dt| dt.date().to_string())
                        .unwrap_or_else(|_| "Invalid date".to_string())
                });

                job.source = if source == &SOURCE_PB.to_string() {
                    Some("Platsbanken".to_string())
                } else {
                    Some("External".to_string())
                };
                job.url = if source == &SOURCE_PB.to_string() {
                    match &job.id {
                        Some(id) => Some(format!("{}{}", URL_JOB_ADS, id)),
                        None => None,
                    }
                } else {
                    job.source_links
                        .as_ref()
                        .and_then(|links| links.first())
                        .map(|link| link.url.clone())
                };

                job.created_at = Some(Utc::now().format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string());
                vec_res.push(job);
            }
            start_records += PAGE_SIZE;
        }
    }
    database::insert_jobs(vec_res).await?;
    Ok(())
}

pub async fn get_jobs_with_search_region(
    duration: &SearchDuration,
    region: Region,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let headers = setup_headers();
    let from_date = serde_json::Value::Null;
    if duration.to_days() != 0 {
        let from_date = Utc::now() - Duration::days(duration.to_days());
        let from_date = from_date.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
    }
    let to_date = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S.%3fZ")
        .to_string();

    let mut vec_res: Vec<JobAd> = Vec::new();
    let job_sources = vec![SOURCE_PB.to_string(), SOURCE_EXTERNAL.to_string()];
    for source in job_sources.iter() {
        let mut start_records = 0;
        while start_records < MAX_RECORDS {
            let payload = serde_json::json!({
                "filters": [{
                    "type": "region",
                    "value": region.get_value()}],
                "fromDate": from_date,
                "order": "date",
                "maxRecords": PAGE_SIZE,
                "startIndex": start_records,
                "toDate": to_date,
                "source": source,
            });
            let mut res: JobResponse = client
                .post(URL_SEARCH)
                .headers(headers.clone())
                .json(&payload)
                .send()
                .await?
                .json()
                .await?;

            for job in res.ads.drain(..) {
                let mut job = job;
                job.published_date = job.published_date.as_ref().map(|date| {
                    NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3fZ")
                        .map(|dt| dt.date().to_string())
                        .unwrap_or_else(|_| "Invalid date".to_string())
                });

                job.last_application_date = job.last_application_date.as_ref().map(|date| {
                    NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3fZ")
                        .map(|dt| dt.date().to_string())
                        .unwrap_or_else(|_| "Invalid date".to_string())
                });

                job.source = if source == &SOURCE_PB.to_string() {
                    Some("Platsbanken".to_string())
                } else {
                    Some("External".to_string())
                };
                job.url = if source == &SOURCE_PB.to_string() {
                    match &job.id {
                        Some(id) => Some(format!("{}{}", URL_JOB_ADS, id)),
                        None => None,
                    }
                } else {
                    job.source_links
                        .as_ref()
                        .and_then(|links| links.first())
                        .map(|link| link.url.clone())
                };

                job.created_at = Some(Utc::now().format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string());

                vec_res.push(job);
            }
            start_records += PAGE_SIZE;
        }
    }
    database::insert_jobs(vec_res).await?;
    Ok(())
}

pub async fn get_jobs_with_search_region_and_title(
    duration: &SearchDuration,
    region: Region,
    title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let headers = setup_headers();
    let from_date = serde_json::Value::Null;
    if duration.to_days() != 0 {
        let from_date = Utc::now() - Duration::days(duration.to_days());
        let from_date = from_date.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
    }
    let to_date = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S.%3fZ")
        .to_string();

    let mut vec_res: Vec<JobAd> = Vec::new();
    let job_sources = vec![SOURCE_PB.to_string(), SOURCE_EXTERNAL.to_string()];
    for source in job_sources.iter() {
        let mut start_records = 0;
        while start_records < MAX_RECORDS {
            let payload = serde_json::json!({
                "filters": [
                    {
                        "type": "freetext",
                        "value": title.to_string(),
                    },
                    {
                        "type": "region",
                        "value": region.get_value()
                    }],
                "fromDate": from_date,
                "order": "date",
                "maxRecords": PAGE_SIZE,
                "startIndex": start_records,
                "toDate": to_date,
                "source": source,
            });
            let mut res: JobResponse = client
                .post(URL_SEARCH)
                .headers(headers.clone())
                .json(&payload)
                .send()
                .await?
                .json()
                .await?;

            for job in res.ads.drain(..) {
                let mut job = job;
                job.published_date = job.published_date.as_ref().map(|date| {
                    NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3fZ")
                        .map(|dt| dt.date().to_string())
                        .unwrap_or_else(|_| "Invalid date".to_string())
                });

                job.last_application_date = job.last_application_date.as_ref().map(|date| {
                    NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3fZ")
                        .map(|dt| dt.date().to_string())
                        .unwrap_or_else(|_| "Invalid date".to_string())
                });

                job.source = if source == &SOURCE_PB.to_string() {
                    Some("Platsbanken".to_string())
                } else {
                    Some("External".to_string())
                };
                job.url = if source == &SOURCE_PB.to_string() {
                    match &job.id {
                        Some(id) => Some(format!("{}{}", URL_JOB_ADS, id)),
                        None => None,
                    }
                } else {
                    job.source_links
                        .as_ref()
                        .and_then(|links| links.first())
                        .map(|link| link.url.clone())
                };

                job.created_at = Some(Utc::now().format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string());
                vec_res.push(job);
            }
            start_records += PAGE_SIZE;
        }
    }
    database::insert_jobs(vec_res).await?;
    Ok(())
}

pub async fn get_jobs_abroad(duration: &SearchDuration) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let headers = setup_headers();
    let from_date: Value;
    if duration.to_days() != 0 {
        let date = Utc::now() - Duration::days(duration.to_days());
        from_date = Value::String(date.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string());
    } else {
        from_date = Value::Null;
    }
    let to_date = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S.%3fZ")
        .to_string();

    let mut vec_res: Vec<JobAd> = Vec::new();
    let mut start_records = 0;
    while start_records < MAX_RECORDS {
        let payload = serde_json::json!({
            "filters": [
                {
                    "type": "abroad",
                    "value": "true"
                }],
            "fromDate": from_date,
            "order": "date",
            "maxRecords": PAGE_SIZE,
            "startIndex": start_records,
            "toDate": to_date,
            "source": SOURCE_PB.to_string(),
        });
        let mut res: JobResponse = client
            .post(URL_SEARCH)
            .headers(headers.clone())
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;

        if res.ads.is_empty() {
            break;
        }
        for job in res.ads.drain(..) {
            let mut job = job;
            job.published_date = job.published_date.as_ref().map(|date| {
                NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3fZ")
                    .map(|dt| dt.date().to_string())
                    .unwrap_or_else(|_| "Invalid date".to_string())
            });
            job.last_application_date = job.last_application_date.as_ref().map(|date| {
                NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3fZ")
                    .map(|dt| dt.date().to_string())
                    .unwrap_or_else(|_| "Invalid date".to_string())
            });

            job.source = Some("Platsbanken".to_string());
            job.url = match &job.id {
                Some(id) => Some(format!("{}{}", URL_JOB_ADS, id)),
                None => None,
            };

            job.created_at = Some(Utc::now().format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string());
            vec_res.push(job);
        }
        start_records += PAGE_SIZE;
    }
    database::insert_jobs(vec_res).await?;
    Ok(())
}

pub async fn get_jobs_unspecified(
    duration: &SearchDuration,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let headers = setup_headers();
    let from_date: Value;
    if duration.to_days() != 0 {
        let date = Utc::now() - Duration::days(duration.to_days());
        from_date = Value::String(date.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string());
    } else {
        from_date = Value::Null;
    }
    let to_date = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S.%3fZ")
        .to_string();

    let mut vec_res: Vec<JobAd> = Vec::new();
    let mut start_records = 0;
    while start_records < MAX_RECORDS {
        let payload = serde_json::json!({
            "filters": [
                {
                    "type": "unspecifiedSwedenWorkplace",
                    "value": "true"
                }],
            "fromDate": from_date,
            "order": "date",
            "maxRecords": PAGE_SIZE,
            "startIndex": start_records,
            "toDate": to_date,
            "source": SOURCE_PB.to_string(),
        });
        let mut res: JobResponse = client
            .post(URL_SEARCH)
            .headers(headers.clone())
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;

        if res.ads.is_empty() {
            break;
        }
        for job in res.ads.drain(..) {
            let mut job = job;
            job.published_date = job.published_date.as_ref().map(|date| {
                NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3fZ")
                    .map(|dt| dt.date().to_string())
                    .unwrap_or_else(|_| "Invalid date".to_string())
            });
            job.last_application_date = job.last_application_date.as_ref().map(|date| {
                NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3fZ")
                    .map(|dt| dt.date().to_string())
                    .unwrap_or_else(|_| "Invalid date".to_string())
            });

            job.source = Some("Platsbanken".to_string());
            job.url = match &job.id {
                Some(id) => Some(format!("{}{}", URL_JOB_ADS, id)),
                None => None,
            };

            job.created_at = Some(Utc::now().format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string());
            vec_res.push(job);
        }
        start_records += PAGE_SIZE;
    }
    database::insert_jobs(vec_res).await?;
    Ok(())
}

pub async fn get_all_jobs(search_duration: &SearchDuration) -> Result<(), Box<dyn std::error::Error>> {
    let pb_duration = search_duration.clone();
    let external_duration = search_duration.clone();
    let abroad_duration = search_duration.clone();
    let unspecified_duration = search_duration.clone();

    let pb_handle = tokio::spawn(async move {
        if let Err(e) = get_jobs_pb(&pb_duration).await {
            error!("Failed to get jobs from PB: {:?}", e);
        } else {
            info!("Platsbanken done!");
        }
    });

    let external_handle = tokio::spawn(async move {
        if let Err(e) = get_jobs_external(&external_duration).await {
            error!("Failed to get external jobs: {:?}", e);
        } else {
            info!("External done!");
        }
    });

    let abroad_handle = tokio::spawn(async move {
        if let Err(e) = get_jobs_abroad(&abroad_duration).await {
            error!("Failed to get abroad jobs: {:?}", e);
        } else {
            info!("Abroad jobs done!");
        }
    });
    let unspecified_handle = tokio::spawn(async move {
        if let Err(e) = get_jobs_unspecified(&unspecified_duration).await {
            error!("Failed to get unspecified jobs: {:?}", e);
        } else {
            info!("Unspecified done!");
        }
    });

    let (pb_res, external_res, abroad_res, unspecificed_res) = tokio::try_join!(
        pb_handle,
        external_handle,
        abroad_handle,
        unspecified_handle,
    )?;

    info!("All jobs fetched!");
    Ok(())
}
