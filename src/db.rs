use sqlx::FromRow;
use anyhow;


/// Structs for sql data tables
#[derive(Clone, FromRow, Debug)]
pub struct MasterEntries {
    pub(crate) cite_key: String,
    pub(crate) entry_type: String,
    // pub(crate) connection: SqlitePool,
}

#[derive(Clone, FromRow, Debug)]
pub struct Book {
    pub(crate) book_id: String,
    pub(crate) cite_key: String,
    pub(crate) publisher_id: String,
    pub(crate) month_year_id: String,
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
    pub(crate) parent_id: String,
    pub(crate) child_id: String,
    pub(crate) cite_key: String,
}

#[derive(Clone, FromRow, Debug)]
pub struct Author {
    pub(crate) cite_key: String,
    pub(crate) author_id: String,
    pub(crate) authors: String,
}

#[derive(Clone, FromRow, Debug)]
pub struct Publisher {
    pub(crate) publisher_id: String,
    pub(crate) publisher: String,
    pub(crate) address: String,
}

#[derive(Clone, FromRow, Debug)]
pub struct Organizations {
    pub(crate) organization_id: String,
    pub(crate) organization: String,
    pub(crate) address: String,
}

#[derive(Clone, FromRow, Debug)]
pub struct MonthYear {
    pub(crate) month_year_id: String,
    pub(crate) month: String,
    pub(crate) year: String,
}

#[derive(Clone, FromRow, Debug)]
pub struct Article {
    pub(crate) cite_key: String,
    pub(crate) article_id: String,
    pub(crate) publisher_id: String,
    pub(crate) month_year_id: String,
    pub(crate) title: String,
    pub(crate) journal: String,
    pub(crate) volume: String,
    pub(crate) pages: String,
    pub(crate) note: String,
    pub(crate) edition: String,
}


