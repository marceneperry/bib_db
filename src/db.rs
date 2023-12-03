use crate::DB_URL;
use sqlite::{State, Value};
use std::string::String;
use uuid::Uuid;

// todo! Implement remaining relational databases

/// Database Structs and implementations for sql data tables
#[derive(Clone, Debug)]
pub struct MasterEntries {
    pub(crate) cite_key: String,
    pub(crate) entry_type: String,
}

#[derive(Clone, Debug)]
pub struct Book {
    pub(crate) book_id: String,
    pub(crate) cite_key: String,
    pub(crate) publisher_id: String,
    pub(crate) month_year_id: String,
    pub(crate) author: String,
    pub(crate) title: String,
    pub(crate) pages: String,
    pub(crate) volume: String,
    pub(crate) edition: String,
    pub(crate) year: String,
    pub(crate) series: String,
    pub(crate) publisher: String,
    pub(crate) note: String,
}
// Implement Later
// #[derive(Clone, Debug)]
// pub struct Relationship {
//     pub(crate) parent_id: String,
//     pub(crate) child_id: String,
//     pub(crate) cite_key: String,
// }

// Implement Later
// #[derive(Clone, Debug)]
// pub struct Author {
//     pub(crate) cite_key: String,
//     pub(crate) author_id: String,
//     pub(crate) authors: String,
// }

#[derive(Clone, Debug)]
pub struct Publisher {
    pub(crate) publisher_id: String,
    pub(crate) publisher: String,
    pub(crate) address: String,
}

// Implement Later
// #[derive(Clone, Debug)]
// pub struct Organizations {
//     pub(crate) organization_id: String,
//     pub(crate) organization: String,
//     pub(crate) address: String,
// }

#[derive(Clone, Debug)]
pub struct MonthYear {
    pub(crate) month_year_id: String,
    pub(crate) month: String,
    pub(crate) year: String,
}

#[derive(Clone, Debug)]
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
    pub(crate) year: String,
    pub(crate) edition: String,
    pub(crate) publisher: String,
}

/// Struct Traits and Implementations
pub trait TableInsert {
    fn insert(&self) -> sqlite::Result<State>;
}

pub trait RowDelete {
    fn delete(item_id: String) -> sqlite::Result<State>;
}

pub trait RowUpdate {
    fn update(&self, item_id: String) -> sqlite::Result<State>;
}

pub trait RowSelect {
    fn select(item_id: &str) -> Vec<String>;
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
            entry_type: "ARTICLE".parse().unwrap(),
        }
    }
}

impl TableInsert for MasterEntries {
    fn insert(&self) -> sqlite::Result<State> {
        let connection = sqlite::open(DB_URL).unwrap();
        let query = "INSERT INTO master_entries VALUES (:cite_key, :entry_type)";
        let mut statement = connection.prepare(query).unwrap();
        statement
            .bind_iter::<_, (_, Value)>([
                (":cite_key", self.cite_key.clone().into()),
                (":entry_type", self.entry_type.clone().into()),
            ])
            .expect("can bind_iter");
        statement.next()
    }
}

impl RowDelete for MasterEntries {
    fn delete(item_id: String) -> sqlite::Result<State> {
        let connection = sqlite::open(DB_URL).unwrap();
        let query = "DELETE FROM master_entries WHERE cite_key = ?";
        let mut statement = connection.prepare(query).unwrap();
        statement
            .bind_iter::<_, (_, Value)>([(1, item_id.into())])
            .unwrap();
        statement.next()
    }
}

impl Book {
    /// Create and Add book to SQLite database
    pub fn book_transaction(textarea: Vec<String>) {
        let master = MasterEntries::new_book();
        let publisher = Publisher::new(textarea.clone());
        let year = textarea[5].clone();
        let m_y = MonthYear::new(year);
        let book_id = Uuid::new_v4().to_string();
        let book = Book {
            book_id,
            cite_key: master.cite_key.clone(),
            publisher_id: publisher.publisher_id.clone(),
            month_year_id: m_y.month_year_id.clone(),
            author: textarea[0].clone(),
            title: textarea[1].clone(),
            pages: textarea[2].clone(),
            volume: textarea[3].clone(),
            edition: textarea[4].clone(),
            year: textarea[5].clone(),
            series: textarea[6].clone(),
            publisher: textarea[7].clone(),
            note: textarea[8].clone(),
        };

        // todo! make these a transaction so that if one of the insert()'s fail it will rollback; probably change to use rusqlite crate instead of sqlite crate and refactor
        let _ = master.insert();
        let _ = book.insert();
        let _ = publisher.insert();
        let _ = m_y.insert();
    }

    pub fn delete_book(item_id: String) {
        let _ = MasterEntries::delete(item_id.clone());
        let _ = Book::delete(item_id.clone());
    }

    pub fn book_update(textarea: Vec<String>, item_id: String) {
        let master = MasterEntries::new_book();
        let publisher = Publisher::new(textarea.clone());
        let year = textarea[5].clone();
        let m_y = MonthYear::new(year);
        let book = Book {
            book_id: item_id.clone(),
            cite_key: master.cite_key.clone(),
            publisher_id: publisher.publisher_id.clone(),
            month_year_id: m_y.month_year_id.clone(),
            author: textarea[0].clone(),
            title: textarea[1].clone(),
            pages: textarea[2].clone(),
            volume: textarea[3].clone(),
            edition: textarea[4].clone(),
            year: textarea[5].clone(),
            series: textarea[6].clone(),
            publisher: textarea[7].clone(),
            note: textarea[8].clone(),
        };
        let _ = Book::update(&book, item_id.clone());
    }
}

impl TableInsert for Book {
    fn insert(&self) -> sqlite::Result<State> {
        let connection = sqlite::open(DB_URL).unwrap();
        let query = "INSERT INTO book VALUES (:book_id, :cite_key, :publisher_id, :month_year_id, :author, :title, :pages, :volume, :edition, :year, :series, :publisher, :note)";
        let mut statement = connection.prepare(query).unwrap();
        statement
            .bind_iter::<_, (_, Value)>([
                (":book_id", self.book_id.clone().into()),
                (":cite_key", self.cite_key.clone().into()),
                (":publisher_id", self.publisher_id.clone().into()),
                (":month_year_id", self.month_year_id.clone().into()),
                (":author", self.author.clone().into()),
                (":title", self.title.clone().into()),
                (":pages", self.pages.clone().into()),
                (":volume", self.volume.clone().into()),
                (":edition", self.edition.clone().into()),
                (":year", self.year.clone().into()),
                (":series", self.series.clone().into()),
                (":publisher", self.publisher.clone().into()),
                (":note", self.note.clone().into()),
            ])
            .unwrap();
        statement.next()
    }
}

impl RowUpdate for Book {
    fn update(&self, item_id: String) -> sqlite::Result<State> {
        let connection = sqlite::open(DB_URL).unwrap();
        let query = "UPDATE book SET author = :author, title = :title, pages = :pages, volume = :volume, edition = :edition, year = :year, series = :series, publisher = :publisher, note = :note WHERE cite_key = :cite_key";
        let mut statement = connection.prepare(query).unwrap();
        statement
            .bind_iter::<_, (_, Value)>([
                (":author", self.author.clone().into()),
                (":title", self.title.clone().into()),
                (":pages", self.pages.clone().into()),
                (":volume", self.volume.clone().into()),
                (":edition", self.edition.clone().into()),
                (":year", self.year.clone().into()),
                (":series", self.series.clone().into()),
                (":publisher", self.publisher.clone().into()),
                (":note", self.note.clone().into()),
                (":cite_key", item_id.into()),
            ])
            .unwrap();
        statement.next()
    }
}

impl RowDelete for Book {
    fn delete(item_id: String) -> sqlite::Result<State> {
        let connection = sqlite::open(DB_URL).unwrap();
        let query = "DELETE FROM book WHERE cite_key = ?";
        let mut statement = connection.prepare(query).unwrap();
        statement
            .bind_iter::<_, (_, Value)>([(1, item_id.into())])
            .unwrap();
        statement.next()
    }
}

impl RowSelect for Book {
    fn select(item_id: &str) -> Vec<String> {
        let connection = sqlite::open(DB_URL).unwrap();
        let query = "SELECT * FROM book WHERE cite_key = :cite_key";
        let mut statement = connection.prepare(query).unwrap();
        statement.bind((":cite_key", item_id)).expect("can bind id");
        let mut text_vec = Vec::new();

        while let Ok(State::Row) = statement.next() {
            text_vec = vec![
                "author",
                "title",
                "pages",
                "volume",
                "edition",
                "year",
                "series",
                "publisher",
                "note",
            ]
            .into_iter()
            .map(|index| statement.read::<String, _>(index).unwrap())
            .collect();
        }
        text_vec
    }
}

impl MonthYear {
    pub fn new(year: String) -> MonthYear {
        let month_year_id = Uuid::new_v4().to_string();
        MonthYear {
            month_year_id,
            month: String::from("01"),
            year,
        }
    }
}

impl TableInsert for MonthYear {
    fn insert(&self) -> sqlite::Result<State> {
        let connection = sqlite::open(DB_URL).unwrap();
        let query = "INSERT INTO month_year VALUES (:month_year_id, :month, :year)";
        let mut statement = connection.prepare(query).unwrap();
        statement
            .bind_iter::<_, (_, Value)>([
                (":month_year_id", self.month_year_id.clone().into()),
                (":month", self.month.clone().into()),
                (":year", self.year.clone().into()),
            ])
            .unwrap();
        statement.next()
    }
}

impl Article {
    /// Create and Add article to SQLite database
    pub(crate) fn article_transaction(textarea: Vec<String>) {
        let master = MasterEntries::new_article();
        let publisher = Publisher::new(textarea.clone());
        let year = textarea[5].clone();
        let m_y = MonthYear::new(year);
        let article_id = Uuid::new_v4().to_string();
        let article = Article {
            cite_key: master.cite_key.clone(),
            article_id,
            publisher_id: publisher.publisher_id.clone(),
            month_year_id: m_y.month_year_id.clone(),
            title: textarea[0].clone(),
            journal: textarea[1].clone(),
            volume: textarea[2].clone(),
            pages: textarea[3].clone(),
            note: textarea[4].clone(),
            year: textarea[5].clone(),
            edition: textarea[6].clone(),
            publisher: textarea[7].clone(),
        };

        let _ = master.insert();
        let _ = article.insert();
        let _ = publisher.insert();
        let _ = m_y.insert();
    }

    pub fn delete_article(item_id: String) {
        let _ = MasterEntries::delete(item_id.clone());
        let _ = Article::delete(item_id.clone());
    }

    pub fn article_update(textarea: Vec<String>, item_id: String) {
        let master = MasterEntries::new_article();
        let publisher = Publisher::new(textarea.clone());
        let year = textarea[5].clone();
        let m_y = MonthYear::new(year);
        // let article_id = Uuid::new_v4().to_string();
        let article = Article {
            cite_key: master.cite_key.clone(),
            article_id: item_id.clone(),
            publisher_id: publisher.publisher_id.clone(),
            month_year_id: m_y.month_year_id.clone(),
            title: textarea[0].clone(),
            journal: textarea[1].clone(),
            volume: textarea[2].clone(),
            pages: textarea[3].clone(),
            note: textarea[4].clone(),
            year: textarea[5].clone(),
            edition: textarea[6].clone(),
            publisher: textarea[7].clone(),
        };
        let _ = Article::update(&article, item_id.clone());
    }
}

impl TableInsert for Article {
    fn insert(&self) -> sqlite::Result<State> {
        let connection = sqlite::open(DB_URL).unwrap();
        let query = "INSERT INTO article VALUES (:cite_key, :article_id, :publisher_id, :month_year_id, :title, :journal, :volume, :pages, :note, :year, :edition, :publisher)";
        let mut statement = connection.prepare(query).unwrap();
        statement
            .bind_iter::<_, (_, Value)>([
                (":cite_key", self.cite_key.clone().into()),
                (":article_id", self.article_id.clone().into()),
                (":publisher_id", self.publisher_id.clone().into()),
                (":month_year_id", self.month_year_id.clone().into()),
                (":title", self.title.clone().into()),
                (":journal", self.journal.clone().into()),
                (":volume", self.volume.clone().into()),
                (":pages", self.pages.clone().into()),
                (":note", self.note.clone().into()),
                (":year", self.year.clone().into()),
                (":edition", self.edition.clone().into()),
                (":publisher", self.publisher.clone().into()),
            ])
            .unwrap();
        statement.next()
    }
}

impl RowDelete for Article {
    fn delete(item_id: String) -> sqlite::Result<State> {
        let connection = sqlite::open(DB_URL).unwrap();
        let query = "DELETE FROM article WHERE cite_key = ?";
        let mut statement = connection.prepare(query).unwrap();
        statement
            .bind_iter::<_, (_, Value)>([(1, item_id.into())])
            .unwrap();
        statement.next()
    }
}

impl RowUpdate for Article {
    fn update(&self, item_id: String) -> sqlite::Result<State> {
        let connection = sqlite::open(DB_URL).unwrap();
        let query = "UPDATE article SET title = :title, journal = :journal, volume = :volume, pages = :pages, note = :note, year = :year, edition = :edition, publisher = :publisher WHERE cite_key = :cite_key";
        let mut statement = connection.prepare(query).unwrap();
        statement
            .bind_iter::<_, (_, Value)>([
                (":title", self.title.clone().into()),
                (":journal", self.journal.clone().into()),
                (":volume", self.volume.clone().into()),
                (":pages", self.pages.clone().into()),
                (":note", self.note.clone().into()),
                (":year", self.year.clone().into()),
                (":edition", self.edition.clone().into()),
                (":publisher", self.publisher.clone().into()),
                (":cite_key", item_id.into()),
            ])
            .unwrap();
        statement.next()
    }
}

impl RowSelect for Article {
    fn select(item_id: &str) -> Vec<String> {
        let connection = sqlite::open(DB_URL).unwrap();
        let query = "SELECT * FROM article WHERE cite_key = ?";
        let mut statement = connection.prepare(query).unwrap();
        statement
            .bind_iter::<_, (_, Value)>([(1, item_id.into())])
            .expect("can bind id");
        let mut text_vec = Vec::new();

        while let Ok(State::Row) = statement.next() {
            text_vec = vec![
                "title",
                "journal",
                "volume",
                "pages",
                "note",
                "year",
                "edition",
                "publisher",
            ]
            .into_iter()
            .map(|index| statement.read::<String, _>(index).unwrap())
            .collect();
        }
        text_vec
    }
}

impl Publisher {
    pub fn new(vec: Vec<String>) -> Publisher {
        let publisher_id = Uuid::new_v4().to_string();
        Publisher {
            publisher_id,
            publisher: vec[7].clone(),
            address: String::from("n/a"), // This would be a lookup based on the publisher name
        }
    }
}

impl TableInsert for Publisher {
    fn insert(&self) -> sqlite::Result<State> {
        let connection = sqlite::open(DB_URL).unwrap();
        let query = "INSERT INTO publisher VALUES (:publisher_id, :publisher, :address)";
        let mut statement = connection.prepare(query).unwrap();
        statement
            .bind_iter::<_, (_, Value)>([
                (":publisher_id", self.publisher_id.clone().into()),
                (":publisher", self.publisher.clone().into()),
                (":address", self.address.clone().into()),
            ])
            .as_ref()
            .unwrap();
        statement.next()
    }
}

// impl Relationship {
//     pub fn new(master_key: String) -> Relationship {
//         let parent_id = Uuid::new_v4().to_string();
//         let child_id = Uuid::new_v4().to_string();
//         Relationship {
//             parent_id,
//             child_id,
//             cite_key: master_key,
//         }
//     }
// }
//
// impl TableInsert for Relationship {
//     fn insert(&self) {
//         let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
//         let result = sqlx::query("INSERT INTO relationship (parent_id, child_id, cite_key) VALUES (?,?,?,)")
//             .bind(&self.parent_id)
//             .bind(&self.child_id)
//             .bind(&self.cite_key)
//             .execute(&*db)
//             .await;
//
//         match result {
//             Ok(rs) => eprintln!("Row inserted: {:?}", rs),
//             Err(e) => eprintln!("Error inserting row: {}", e),
//         };
//     }
// }

// // Implement later
// impl Author {
//     pub fn new(master_key: String) -> Author {
//         let author_id = Uuid::new_v4().to_string();
//         Author {
//             cite_key: master_key,
//             author_id,
//             authors: String::new(),
//         }
//     }
// }
//
// impl TableInsert for Author {
//     fn insert(&self) {
//         let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
//         let result = sqlx::query("INSERT INTO author (cite_key, author_id, authors) VALUES (?,?,?,)")
//             .bind(&self.cite_key)
//             .bind(&self.author_id)
//             .bind(&self.authors)
//             .execute(&*db)
//             .await;
//
//         match result {
//             Ok(rs) => eprintln!("Row inserted: {:?}", rs),
//             Err(e) => eprintln!("Error inserting row: {}", e),
//         };
//     }
// }
// // Implement later
// impl Organizations {
//     pub fn new() -> Organizations {
//         let organization_id = Uuid::new_v4().to_string();
//         Organizations {
//             organization_id,
//             organization: String::new(),
//             address: String::new(),
//         }
//     }
// }
//
// impl TableInsert for Organizations {
//     fn insert(&self) {
//         let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
//         let result = sqlx::query("INSERT INTO organizations (organization_id, organization, address) VALUES (?,?,?,)")
//             .bind(&self.organization_id)
//             .bind(&self.organization)
//             .bind(&self.address)
//             .execute(&*db)
//             .await;
//
//         match result {
//             Ok(rs) => eprintln!("Row inserted: {:?}", rs),
//             Err(e) => eprintln!("Error inserting row: {}", e),
//         };
//     }
// }

// #[cfg(test)]
// mod test {
//     /// Initialize a memory test by using MEM_DB as the connection path in init_db and running `cargo --bin init_db`
//     use super::*;
//     const MEM_DB: &str = ":memory:";
//
//     #[test]
//     fn test_sqlite_connection() {
//         let connection = sqlite::open(DB_URL).unwrap();
//     }
//
//     // assert!()
// }
