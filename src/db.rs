use serde::{Deserialize, Serialize};
// use sqlx::{FromRow, SqlitePool};
// use uuid::Uuid;
// use crate::DB_URL;
// use sqlite::{Row, };


/// Database Structs and implementations for sql data tables
#[derive(Clone, Debug)]
pub struct MasterEntries {
    pub(crate) cite_key: String,
    pub(crate) entry_type: String,
    // pub(crate) connection: SqlitePool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub(crate) series: String,
    pub(crate) note: String,
}

#[derive(Clone, Debug)]
pub struct Relationship {
    pub(crate) parent_id: String,
    pub(crate) child_id: String,
    pub(crate) cite_key: String,
}

#[derive(Clone, Debug)]
pub struct Author {
    pub(crate) cite_key: String,
    pub(crate) author_id: String,
    pub(crate) authors: String,
}

#[derive(Clone, Debug)]
pub struct Publisher {
    pub(crate) publisher_id: String,
    pub(crate) publisher: String,
    pub(crate) address: String,
}

#[derive(Clone, Debug)]
pub struct Organizations {
    pub(crate) organization_id: String,
    pub(crate) organization: String,
    pub(crate) address: String,
}

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
    pub(crate) edition: String,
}



// /// Struct Traits and Implementations
// // #[async_trait]
// pub trait TableInsert {
//     fn insert(&self) {} // maybe use return type Result<> here?
// }
//
// impl MasterEntries {
//     pub fn new_book() -> MasterEntries {
//         let key = Uuid::new_v4().to_string();
//         MasterEntries {
//             cite_key: key,
//             entry_type: "BOOK".parse().unwrap(),
//         }
//     }
//
//     pub fn new_article() -> MasterEntries {
//         let key = Uuid::new_v4().to_string();
//         MasterEntries {
//             cite_key: key,
//             entry_type: "ARTICLE".parse().unwrap()
//         }
//     }
// }
//
// // #[async_trait]
// impl TableInsert for MasterEntries {
//     fn insert(&self) {
//         let db = SqlitePool::connect(DB_URL).unwrap();
//         let result = sqlx::query("INSERT INTO master_entries (cite_key, entry_type) VALUES (?,?,)")
//                 .bind(&self.cite_key)
//                 .bind(&self.entry_type)
//                 .execute(&*db);
//
//             match result {
//             Ok(rs) => eprintln!("Row inserted: {:?}", rs),
//             Err(e) => eprintln!("Error inserting row: {}", e),
//             };
//     }
// }
//
// impl Book {
//     /// Create and Add book to SQLite database
//     fn book_transaction() {
//         let master = MasterEntries::new_book();
//         let publisher = Publisher::new();
//         let year = String::new();
//         let m_y = MonthYear::new(year);
//         let book_id = Uuid::new_v4().to_string();
//         let book = Book {
//             book_id,
//             cite_key: master.cite_key.clone(),
//             publisher_id: publisher.publisher_id.clone(),
//             month_year_id: m_y.month_year_id.clone(),
//             author: String::new(),
//             title: String::new(),
//             pages: String::new(),
//             volume: String::new(),
//             edition: String::new(),
//             series: String::new(),
//             note: String::new(),
//         };
//
//         // master.insert();
//         // book.insert();
//         // publisher.insert();
//         // m_y.insert();
//     }
// }
//
// // #[async_trait]
// impl TableInsert for Book {
//     fn insert(&self) {
//         let db = sqlite::open(DB_URL).unwrap();
//         let result = sqlite::  ("INSERT INTO book (book_id, cite_key, publisher_id, month_year_id, editor, title, pages, volume, edition, series, notes) VALUES (?,?,?,?,?,?,?,?,?,?,?,)")
//             .bind(&self.book_id)
//             .bind(&self.cite_key)
//             .bind(&self.publisher_id)
//             .bind(&self.month_year_id)
//             .bind(&self.author)
//             .bind(&self.title)
//             .bind(&self.pages)
//             .bind(&self.volume)
//             .bind(&self.edition)
//             .bind(&self.series)
//             .bind(&self.note)
//             .execute(&*db);
//
//         match result {
//             Ok(rs) => eprintln!("Row inserted: {:?}", rs),
//             Err(e) => eprintln!("Error inserting row: {}", e),
//         };
//     }
// }
//
//
// impl MonthYear {
//     pub fn new(year: String) -> MonthYear {
//         let month_year_id = Uuid::new_v4().to_string();
//         MonthYear {
//             month_year_id,
//             month: String::new(),
//             year,
//         }
//     }
// }
//
// // #[async_trait]
// impl TableInsert for MonthYear {
//     fn insert(&self) {
//         let db = SqlitePool::connect(DB_URL).unwrap();
//         let result = sqlx::query("INSERT INTO month_year (month_year_id, month, year) VALUES (?,?,?,)")
//             .bind(&self.month_year_id)
//             .bind(&self.month)
//             .bind(&self.year)
//             .execute(&*db);
//
//         match result {
//             Ok(rs) => eprintln!("Row inserted: {:?}", rs),
//             Err(e) => eprintln!("Error inserting row: {}", e),
//         };
//     }
// }
//
// impl Article {
//     /// Create and Add book to SQLite database
//     fn article_transaction() {
//         let master = MasterEntries::new_article();
//         let publisher = Publisher::new();
//         let year = String::new();
//         let m_y = MonthYear::new(year);
//         let article_id = Uuid::new_v4().to_string();
//         let article = Article {
//             cite_key: master.cite_key.clone(),
//             article_id,
//             publisher_id: publisher.publisher_id.clone(),
//             month_year_id: m_y.month_year_id.clone(),
//             title: String::new(),
//             journal: String::new(),
//             volume: String::new(),
//             pages: String::new(),
//             note: String::new(),
//             edition: String::new(),
//         };
//
//         master.insert();
//         article.insert();
//         publisher.insert();
//         m_y.insert();
//     }
// }
//
// // #[async_trait]
// impl TableInsert for Article {
//     fn insert(&self) {
//         let db = SqlitePool::connect(DB_URL).unwrap();
//         let result = sqlx::query("INSERT INTO article (cite_key, article_id, publisher_id, month_year_id, title, journal, volume, pages, note, edition) VALUES (?,?,?,?,?,?,?,?,?,?)")
//             .bind(&self.cite_key)
//             .bind(&self.article_id)
//             .bind(&self.publisher_id)
//             .bind(&self.month_year_id)
//             .bind(&self.title)
//             .bind(&self.journal)
//             .bind(&self.volume)
//             .bind(&self.pages)
//             .bind(&self.note)
//             .bind(&self.edition)
//             .execute(&*db);
//
//         match result {
//             Ok(rs) => eprintln!("Row inserted: {:?}", rs),
//             Err(e) => eprintln!("Error inserting row: {}", e),
//         };
//     }
// }
//
//
// impl Publisher {
//     pub fn new() -> Publisher {
//         let publisher_id = Uuid::new_v4().to_string();
//         Publisher {
//             publisher_id,
//             publisher: String::new(),
//             address: String::new(),
//         }
//     }
// }
//
// // #[async_trait]
// impl TableInsert for Publisher {
//     fn insert(&self) {
//         let db = SqlitePool::connect(DB_URL).unwrap();
//         let result = sqlx::query("INSERT INTO publisher (publisher_id, publisher, address) VALUES (?,?,?,)")
//             .bind(&self.publisher_id)
//             .bind(&self.publisher)
//             .bind(&self.address)
//             .execute(&*db);
//
//         match result {
//             Ok(rs) => eprintln!("Row inserted: {:?}", rs),
//             Err(e) => eprintln!("Error inserting row: {}", e),
//         };
//     }
// }


// // Implement later
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
// #[async_trait]
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
// #[async_trait]
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
// #[async_trait]
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


