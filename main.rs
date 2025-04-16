#![allow(unused_variables)]
#![allow(unused_must_use)]
#![allow(dead_code)]
#![allow(unused_imports)]

use crate::database::{connect_database, SearchQuery};
use crate::find_jobs::SearchDuration;
use crate::logging::{error, setup_log};
use bot::run_bot;
use database::SaveToCsv;
use log::info;
use tokio;

mod bot;
mod constants;
mod database;
mod find_jobs;
mod logging;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log().await?;
    connect_database().await?;

    // find_jobs::get_all_jobs(&SearchDuration::Weekly).await?;

    let res = database::get_jobs_by_query(SearchQuery::Title(vec!["Data".to_string()])).await?;
    res.save_to_csv("jobs.csv")?;
    Ok(())
}
