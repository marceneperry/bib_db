use std::string::String as String;
use uuid::Uuid;
use crate::db::{connect, MasterEntries, Book, Publisher, Relationship, Article, Author, MonthYear, Organizations};
use crate::DB_URL;
use async_trait::async_trait;

// currently only adding new items. later add ability to search and edit items.

#[async_trait]
pub trait TableInsert {
    async fn insert(&self) {} // maybe use return type Result<> here?
}

impl MasterEntries {
    pub fn new_book() -> MasterEntries {
        MasterEntries {
            cite_key: Uuid::new_v4(),
            entry_type: "BOOK".parse().unwrap()
        }
    }

    pub fn new_article() -> MasterEntries {
        MasterEntries {
            cite_key: Uuid::new_v4(),
            entry_type: "ARTICLE".parse().unwrap()
        }
    }
}

#[async_trait]
impl TableInsert for MasterEntries {
    async fn insert(&self) {
        let result = sqlx::query("INSERT INTO master_entries (cite_key, entry_type) VALUES (?,?,)")
                .bind(self.cite_key)
                .bind(&self.entry_type)
                .execute(&DB_URL)
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
        let m_y = MonthYear::new(year as u8);
        let book = Book {
            book_id: Uuid::new_4(),
            cite_key: master.cite_key,
            publisher_id: publisher.publisher_id,
            month_year_id: m_y.month_year_id,
            editor: String::new(),
            title: String::new(),
            pages: String::new(),
            volume: String::new(),
            edition: String::new(),
            series: String::new(),
            note: String::new(),
        };

        let _connection = connect();

        master.insert().await;
        book.insert().await;
        publisher.insert().await;
        m_y.insert().await;

    }
}

#[async_trait]
impl TableInsert for Book {
    async fn insert(&self) {
        let result = sqlx::query("INSERT INTO book (book_id, cite_key, publisher_id, month_year_id, editor, title, pages, volume, edition, series, notes) VALUES (?,?,?,?,?,?,?,?,?,?,?,)")
            .bind(self.book_id)
            .bind(self.cite_key)
            .bind(self.publisher_id)
            .bind(self.month_year_id)
            .bind(&self.editor)
            .bind(&self.title)
            .bind(&self.pages)
            .bind(&self.volume)
            .bind(&self.edition)
            .bind(&self.series)
            .bind(&self.note)
            .execute(&DB_URL)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }


}

impl Relationship {
    pub fn new(master_key: Uuid) -> Relationship {
        Relationship {
            parent_id: Uuid::new_v4(),
            child_id: Uuid::new_v4(),
            cite_key: master_key,
        }
    }
}

#[async_trait]
impl TableInsert for Relationship {
    async fn insert(&self) {
        let result = sqlx::query("INSERT INTO relationship (parent_id, child_id, cite_key) VALUES (?,?,?,)")
            .bind(self.parent_id)
            .bind(self.child_id)
            .bind(self.cite_key)
            .execute(&DB_URL)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}

impl Author {
    pub fn new(master_key: Uuid) -> Author {
        Author {
            cite_key: master_key,
            author_id: Uuid::new_v4(),
            authors: String::new(),
        }
    }
}

#[async_trait]
impl TableInsert for Author {
    async fn insert(&self) {
        let result = sqlx::query("INSERT INTO author (cite_key, author_id, authors) VALUES (?,?,?,)")
            .bind(self.cite_key)
            .bind(self.author_id)
            .bind(&self.authors)
            .execute(&DB_URL)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}

impl Publisher {
    pub fn new() -> Publisher {
        Publisher {
            publisher_id: Uuid::new_v4(),
            publisher: String::new(),
            address: String::new(),
        }
    }
}

#[async_trait]
impl TableInsert for Publisher {
    async fn insert(&self) {
        let result = sqlx::query("INSERT INTO publisher (publisher_id, publisher, address) VALUES (?,?,?,)")
            .bind(self.publisher_id)
            .bind(&self.publisher)
            .bind(&self.address)
            .execute(&DB_URL)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}

impl Organizations {
    pub fn new() -> Organizations {
        Organizations {
            organization_id: Uuid::new_v4(),
            organization: String::new(),
            address: String::new(),
        }
    }
}

#[async_trait]
impl TableInsert for Organizations {
    async fn insert(&self) {
        let result = sqlx::query("INSERT INTO organizations (organization_id, organization, address) VALUES (?,?,?,)")
            .bind(self.organization_id)
            .bind(&self.organization)
            .bind(&self.address)
            .execute(&DB_URL)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}

impl MonthYear {
    pub fn new(year: u8) -> MonthYear {
        MonthYear {
            month_year_id: Uuid::new_v4(),
            month: String::new(),
            year,
        }
    }
}

#[async_trait]
impl TableInsert for MonthYear {
    async fn insert(&self) {
        let result = sqlx::query("INSERT INTO month_year (month_year_id, month, year) VALUES (?,?,?,)")
            .bind(&self.month_year_id)
            .bind(&self.month)
            .bind(&self.year)
            .execute(&DB_URL)
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
        let m_y = MonthYear::new(year as u8);
        let article = Article {
            cite_key: master.cite_key,
            article_id: Uuid::new_v4(),
            publisher_id: publisher.publisher_id,
            month_year_id: m_y.month_year_id,
            title: String::new(),
            journal: String::new(),
            volume: String::new(),
            pages: String::new(),
            note: String::new(),
            edition: String::new(),
        };

        let _connection = connect();

        master.insert().await;
        article.insert().await;
        publisher.insert().await;
        m_y.insert().await;
    }
}

#[async_trait]
impl TableInsert for Article {
    async fn insert(&self) {
        let result = sqlx::query("INSERT INTO article (cite_key, article_id, publisher_id, month_year_id, title, journal, volume, pages, note, edition) VALUES (?,?,?,?,?,?,?,?,?,?)")
            .bind(self.cite_key)
            .bind(self.article_id)
            .bind(self.publisher_id)
            .bind(self.month_year_id)
            .bind(&self.title)
            .bind(&self.journal)
            .bind(&self.volume)
            .bind(&self.pages)
            .bind(&self.note)
            .bind(&self.edition)
            .execute(&DB_URL)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}

pub struct App<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            titles: vec!["Home", "NewBook", "NewArticle"],
            index: 0,
            // select_input: None,
            // current_screen: CurrentScreen::Main,
            // currently_editing: None,
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