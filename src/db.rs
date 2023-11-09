use sqlx::{FromRow, Pool, Sqlite, SqlitePool};
use uuid::Uuid;
use crate::DB_URL;

/// Database connection
pub(crate) async fn connect() -> Pool<Sqlite> {
    SqlitePool::connect(DB_URL).await.unwrap()
}


/// Structs for sql data tables
#[derive(Clone, FromRow, Debug)]
pub struct MasterEntries {
    pub(crate) cite_key: Uuid,
    pub(crate) entry_type: String,
}

#[derive(Clone, FromRow, Debug)]
pub struct Book {
    pub(crate) book_id: Uuid,
    pub(crate) cite_key: Uuid,
    pub(crate) publisher_id: Uuid,
    pub(crate) month_year_id: Uuid,
    pub(crate) editor: String,
    pub(crate) title: String,
    pub(crate) pages: String,
    pub(crate) volume: String,
    pub(crate) edition: String,
    pub(crate) series: String,
    pub(crate) note: String,
}

#[derive(Clone, FromRow, Debug)]
pub struct Relationship {
    pub(crate) parent_id: Uuid,
    pub(crate) child_id: Uuid,
    pub(crate) cite_key: Uuid,
}

#[derive(Clone, FromRow, Debug)]
pub struct Author {
    pub(crate) cite_key: Uuid,
    pub(crate) author_id: Uuid,
    pub(crate) authors: String,
}

#[derive(Clone, FromRow, Debug)]
pub struct Publisher {
    pub(crate) publisher_id: Uuid,
    pub(crate) publisher: String,
    pub(crate) address: String,
}

#[derive(Clone, FromRow, Debug)]
pub struct Organizations {
    pub(crate) organization_id: Uuid,
    pub(crate) organization: String,
    pub(crate) address: String,
}

#[derive(Clone, FromRow, Debug)]
pub struct MonthYear {
    pub(crate) month_year_id: Uuid,
    pub(crate) month: String,
    pub(crate) year: u8,
}

#[derive(Clone, FromRow, Debug)]
pub struct Article {
    pub(crate) cite_key: Uuid,
    pub(crate) article_id: Uuid,
    pub(crate) publisher_id: Uuid,
    pub(crate) month_year_id: Uuid,
    pub(crate) title: String,
    pub(crate) journal: String,
    pub(crate) volume: String,
    pub(crate) pages: String,
    pub(crate) note: String,
    pub(crate) edition: String,
}


