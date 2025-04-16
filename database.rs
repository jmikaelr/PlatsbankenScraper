#![allow(dead_code)]
use csv::Writer;
use futures::future::select;
use sqlx::{Error, FromRow, Row, SqlitePool};
use std::path::Path;
use tokio::sync::OnceCell;

use crate::find_jobs::JobAd;
use crate::logging::*;
use crate::{bot::UserSelections, find_jobs::OccupationType};
use serde::{Deserialize, Serialize};

static POOL: OnceCell<SqlitePool> = OnceCell::const_new();

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DbJobAd {
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
    #[serde(skip_deserializing)]
    pub created_at: Option<String>,
}
pub trait SaveToCsv {
    fn save_to_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<(), Error>;
}

#[derive(Debug, Default)]
pub struct User {
    pub id: String,
    pub selections: UserSelections,
}

impl User {
    pub fn new(id: String) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }
}

impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for DbJobAd {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(DbJobAd {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            occupation: row.try_get("occupation")?,
            workplace: row.try_get("workplace")?,
            workplace_name: row.try_get("workplace_name")?,
            published_date: row.try_get("published_date")?,
            last_application_date: row.try_get("last_application_date")?,
            source: row.try_get("source")?,
            url: row.try_get("url")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

#[derive(Debug)]
pub enum SearchQuery {
    Location(Vec<String>),
    Title(Vec<String>),
    Occupation(Vec<String>),
    Company(Vec<String>),
    MostRecent(i32),
    All,
}

pub async fn connect_database() -> Result<(), sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://database.db".to_string());
    let pool = SqlitePool::connect(&database_url).await?;
    POOL.set(pool)
        .map_err(|_| sqlx::Error::Protocol("Database pool initialization failed".into()))?;
    info!("Database initialized!");
    Ok(())
}

pub async fn insert_jobs(job_ads: Vec<JobAd>) -> Result<(), sqlx::Error> {
    let pool = POOL.get().expect("Database pool is not initialised");

    let mut transaction = pool.begin().await?;

    for job in &job_ads {
        let query = r#"
            INSERT OR IGNORE INTO jobs (id, title, occupation, workplace, workplace_name, published_date, last_application_date, source, url, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(&job.id)
            .bind(&job.title)
            .bind(&job.occupation)
            .bind(&job.workplace)
            .bind(&job.workplace_name)
            .bind(&job.published_date.as_deref())
            .bind(&job.last_application_date.as_deref())
            .bind(&job.source)
            .bind(&job.url)
            .bind(&job.created_at)
            .execute(&mut *transaction)
            .await?;
    }
    transaction.commit().await?;

    info!("Inserted {} entries to database!", job_ads.len());
    Ok(())
}

/////////////////////// USER FUNCTIONS //////////////////////
/////////////////////// USER FUNCTIONS //////////////////////
/////////////////////// USER FUNCTIONS //////////////////////

pub async fn store_user(user: User) -> Result<(), sqlx::Error> {
    let pool = POOL.get().expect("Database pool is not initialised");
    let occupations_json = user
        .selections
        .selected_occupations
        .map(|occupations| serde_json::to_string(&occupations).unwrap_or_default());

    let regions_json = user
        .selections
        .selected_regions
        .map(|regions| serde_json::to_string(&regions).unwrap_or_default());
    let query = sqlx::query!(
        r#"
        INSERT INTO users (id, jobcategories, regions)
        VALUES ($1, $2, $3)
        ON CONFLICT (id)
        DO UPDATE SET
            jobcategories = CASE WHEN EXCLUDED.jobcategories IS NOT NULL THEN EXCLUDED.jobcategories ELSE users.jobcategories END,
            regions = CASE WHEN EXCLUDED.regions IS NOT NULL THEN EXCLUDED.regions ELSE users.regions END
        "#,
        user.id,
        occupations_json,
        regions_json
    );

    info!("Inserting or updating user {} in database!", user.id);
    query.execute(pool).await?;

    Ok(())
}

pub async fn get_database_entries_count() -> Result<i64, sqlx::Error> {
    let pool = POOL.get().expect("Database pool is not initialized");
    let result = sqlx::query!(
        r#"
SELECT COUNT(*) as count
FROM jobs
"#
    )
    .fetch_one(pool)
    .await?;
    Ok(result.count)
}

pub async fn print_database_entries_count() {
    let pool = POOL.get().expect("Database pool is not initialized");
    let result = sqlx::query!(
        r#"
SELECT COUNT(*) as count
FROM jobs
"#
    )
    .fetch_one(pool)
    .await;
    match result {
        Ok(record) => {
            let count = record.count;
            info!("There is {} entries in database!", count)
        }
        Err(e) => {
            error!("Error printing database count! {}", e)
        }
    }
}

pub async fn get_jobs_by_query(query: SearchQuery) -> Result<Vec<DbJobAd>, sqlx::Error> {
    let pool = POOL.get().expect("Database pool is not initialized");

    if let SearchQuery::All = query {
        return sqlx::query_as!(
            DbJobAd,
            r#"
        SELECT *
        FROM jobs
        "#
        )
        .fetch_all(pool)
        .await;
    }

    if let SearchQuery::MostRecent(timestamp) = query {
        return sqlx::query_as!(
            DbJobAd,
            r#"
            SELECT *
            FROM jobs
            WHERE created_at >= $1
            "#,
            timestamp
        )
        .fetch_all(pool)
        .await;
    }

    let (field, values) = match query {
        SearchQuery::Location(locations) => ("workplace", locations),
        SearchQuery::Title(titles) => ("title", titles),
        SearchQuery::Occupation(occupations) => ("occupation", occupations),
        SearchQuery::Company(companies) => ("workplace_name", companies),
        _ => unreachable!(),
    };

    let placeholders = values
        .iter()
        .map(|_| format!("{} LIKE ?", field))
        .collect::<Vec<String>>()
        .join(" OR ");
    let query = format!(
        r#"
        SELECT *
        FROM jobs
        WHERE {}
        "#,
        placeholders
    );
    let mut query_builder = sqlx::query_as::<_, DbJobAd>(&query);
    for value in values {
        let like_value = format!("%{}%", value);
        query_builder = query_builder.bind(like_value);
    }

    let jobs = query_builder.fetch_all(pool).await;
    match &jobs {
        Ok(jobs_list) => {
            info!("Fetched {} amount of jobs", jobs_list.len());
        }
        Err(e) => {
            error!("Error fetching jobs {:?}", e);
        }
    }

    jobs.map_err(|e| {
        error!("Error fetching jobs: {:?}", e);
        e
    })
}

//////////////////////// SAVING TO CSV /////////////////
//////////////////////// SAVING TO CSV /////////////////
//////////////////////// SAVING TO CSV /////////////////
//////////////////////// SAVING TO CSV /////////////////

impl SaveToCsv for Vec<DbJobAd> {
    fn save_to_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<(), Error> {
        let file_path = file_path.as_ref();
        let mut writer = Writer::from_path(file_path)
            .map_err(|e| Error::Protocol(format!("Failed to create CSV file: {:?}", e).into()))?;

        for job in self {
            writer
                .serialize(job)
                .map_err(|e| Error::Protocol(format!("Failed to write CSV row: {:?}", e).into()))?;
        }

        writer
            .flush()
            .map_err(|e| Error::Protocol(format!("Failed to flush CSV writer: {:?}", e).into()))?;
        info!("File successfuly saved to {}", file_path.display());

        Ok(())
    }
}
