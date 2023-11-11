use std::string::String as String;
use std::sync::Arc;
use uuid::Uuid;
use crate::db::{MasterEntries, Book, Publisher, Relationship, Article, Author, MonthYear, Organizations};
use async_trait::async_trait;
use sqlx::SqlitePool;
use crate::DB_URL;


// currently only adding new items. later add ability to search and edit items.

pub struct App<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            titles: vec!["Home", "NewBook", "NewArticle"],
            index: 0,
        }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }

}

pub(crate) enum AppEvent<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum MenuItem {
    Home,
    Book,
    Article,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Book => 1,
            MenuItem::Article => 2,
        }
    }
}



/// Database structures
#[async_trait]
pub trait TableInsert {
    async fn insert(&self) {} // maybe use return type Result<> here?
}

impl MasterEntries {
    pub fn new_book() -> MasterEntries {
        let key = Uuid::new_v4().to_string();
        MasterEntries {
            cite_key: key,
            entry_type: "BOOK".parse().unwrap(),
        }
    }

    pub fn new_article() -> MasterEntries {
        let key = Uuid::new_v4().to_string();
        MasterEntries {
            cite_key: key,
            entry_type: "ARTICLE".parse().unwrap()
        }
    }
}

#[async_trait]
impl TableInsert for MasterEntries {
    async fn insert(&self) {
        let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
        let result = sqlx::query("INSERT INTO master_entries (cite_key, entry_type) VALUES (?,?,)")
                .bind(&self.cite_key)
                .bind(&self.entry_type)
                .execute(&*db)
                .await;

            match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
            };
    }
}

impl Book {
    async fn book_transaction() {
        let master = MasterEntries::new_book();
        let publisher = Publisher::new();
        let year = String::new();
        let m_y = MonthYear::new(year);
        let book_id = Uuid::new_v4().to_string();
        let book = Book {
            book_id,
            cite_key: master.cite_key.clone(),
            publisher_id: publisher.publisher_id.clone(),
            month_year_id: m_y.month_year_id.clone(),
            author: String::new(),
            title: String::new(),
            pages: String::new(),
            volume: String::new(),
            edition: String::new(),
            series: String::new(),
            note: String::new(),
        };



        master.insert().await;
        book.insert().await;
        publisher.insert().await;
        m_y.insert().await;

    }
}

#[async_trait]
impl TableInsert for Book {
    async fn insert(&self) {
        let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
        let result = sqlx::query("INSERT INTO book (book_id, cite_key, publisher_id, month_year_id, editor, title, pages, volume, edition, series, notes) VALUES (?,?,?,?,?,?,?,?,?,?,?,)")
            .bind(&self.book_id)
            .bind(&self.cite_key)
            .bind(&self.publisher_id)
            .bind(&self.month_year_id)
            .bind(&self.author)
            .bind(&self.title)
            .bind(&self.pages)
            .bind(&self.volume)
            .bind(&self.edition)
            .bind(&self.series)
            .bind(&self.note)
            .execute(&*db)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}

impl Relationship {
    pub fn new(master_key: String) -> Relationship {
        let parent_id = Uuid::new_v4().to_string();
        let child_id = Uuid::new_v4().to_string();
        Relationship {
            parent_id,
            child_id,
            cite_key: master_key,
        }
    }
}

#[async_trait]
impl TableInsert for Relationship {
    async fn insert(&self) {
        let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
        let result = sqlx::query("INSERT INTO relationship (parent_id, child_id, cite_key) VALUES (?,?,?,)")
            .bind(&self.parent_id)
            .bind(&self.child_id)
            .bind(&self.cite_key)
            .execute(&*db)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}

impl Author {
    pub fn new(master_key: String) -> Author {
        let author_id = Uuid::new_v4().to_string();
        Author {
            cite_key: master_key,
            author_id,
            authors: String::new(),
        }
    }
}

#[async_trait]
impl TableInsert for Author {
    async fn insert(&self) {
        let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
        let result = sqlx::query("INSERT INTO author (cite_key, author_id, authors) VALUES (?,?,?,)")
            .bind(&self.cite_key)
            .bind(&self.author_id)
            .bind(&self.authors)
            .execute(&*db)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}

impl Publisher {
    pub fn new() -> Publisher {
        let publisher_id = Uuid::new_v4().to_string();
        Publisher {
            publisher_id,
            publisher: String::new(),
            address: String::new(),
        }
    }
}

#[async_trait]
impl TableInsert for Publisher {
    async fn insert(&self) {
        let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
        let result = sqlx::query("INSERT INTO publisher (publisher_id, publisher, address) VALUES (?,?,?,)")
            .bind(&self.publisher_id)
            .bind(&self.publisher)
            .bind(&self.address)
            .execute(&*db)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}

impl Organizations {
    pub fn new() -> Organizations {
        let organization_id = Uuid::new_v4().to_string();
        Organizations {
            organization_id,
            organization: String::new(),
            address: String::new(),
        }
    }
}

#[async_trait]
impl TableInsert for Organizations {
    async fn insert(&self) {
        let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
        let result = sqlx::query("INSERT INTO organizations (organization_id, organization, address) VALUES (?,?,?,)")
            .bind(&self.organization_id)
            .bind(&self.organization)
            .bind(&self.address)
            .execute(&*db)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}

impl MonthYear {
    pub fn new(year: String) -> MonthYear {
        let month_year_id = Uuid::new_v4().to_string();
        MonthYear {
            month_year_id,
            month: String::new(),
            year,
        }
    }
}

#[async_trait]
impl TableInsert for MonthYear {
    async fn insert(&self) {
        let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
        let result = sqlx::query("INSERT INTO month_year (month_year_id, month, year) VALUES (?,?,?,)")
            .bind(&self.month_year_id)
            .bind(&self.month)
            .bind(&self.year)
            .execute(&*db)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}

impl Article {
    async fn article_transaction() {
        let master = MasterEntries::new_article();
        let publisher = Publisher::new();
        let year = String::new();
        let m_y = MonthYear::new(year);
        let article_id = Uuid::new_v4().to_string();
        let article = Article {
            cite_key: master.cite_key.clone(),
            article_id,
            publisher_id: publisher.publisher_id.clone(),
            month_year_id: m_y.month_year_id.clone(),
            title: String::new(),
            journal: String::new(),
            volume: String::new(),
            pages: String::new(),
            note: String::new(),
            edition: String::new(),
        };

        master.insert().await;
        article.insert().await;
        publisher.insert().await;
        m_y.insert().await;
    }
}

#[async_trait]
impl TableInsert for Article {
    async fn insert(&self) {
        let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
        let result = sqlx::query("INSERT INTO article (cite_key, article_id, publisher_id, month_year_id, title, journal, volume, pages, note, edition) VALUES (?,?,?,?,?,?,?,?,?,?)")
            .bind(&self.cite_key)
            .bind(&self.article_id)
            .bind(&self.publisher_id)
            .bind(&self.month_year_id)
            .bind(&self.title)
            .bind(&self.journal)
            .bind(&self.volume)
            .bind(&self.pages)
            .bind(&self.note)
            .bind(&self.edition)
            .execute(&*db)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}

