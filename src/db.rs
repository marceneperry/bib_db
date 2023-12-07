use crate::DB_URL;
use sqlite::{State, Value};
use std::io::Error;
use std::string::String;
use uuid::Uuid;

/// Database Structs and implementations for `SQLite` data tables
// todo! Implement remaining relational databases
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

#[derive(Clone, Debug)]
pub struct Publisher {
    pub(crate) publisher_id: String,
    pub(crate) publisher: String,
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
            .expect("should bind_iter");
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
    /// Create and add `book` to `SQLite` database
    pub fn book_transaction(textarea: Vec<String>) {
        let master = MasterEntries::new_book();
        let publisher = Publisher::new(textarea[7].clone());
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

    /// Remove item from `book` and `master_entries` tables
    pub fn delete_book(item_id: String) {
        let _ = MasterEntries::delete(item_id.clone());
        let _ = Book::delete(item_id.clone());
    }

    /// Update the data in the `book` table
    pub fn book_update(textarea: Vec<String>, item_id: String) {
        let book = Book {
            book_id: item_id.clone(),
            cite_key: "n/a".to_string(),
            publisher_id: "n/a".to_string(),
            month_year_id: "n/a".to_string(),
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
        statement
            .bind((":cite_key", item_id))
            .expect("should bind id");
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

// todo! test? Other tests test this logic
/// Read the `SQLite` database `book` table and returns a vector of `book` objects
pub fn read_sqlite_book_table() -> Result<Vec<Book>, Error> {
    let connection = sqlite::open(DB_URL).unwrap();
    let query = "SELECT book_id, cite_key, publisher_id, month_year_id, author, title, pages, volume, edition, year, series, publisher, note FROM book";
    let mut statement = connection.prepare(query).unwrap();
    let mut parsed = Vec::new();

    while let Ok(State::Row) = statement.next() {
        parsed.push(Book {
            book_id: statement.read::<String, _>("book_id").unwrap(),
            cite_key: statement.read::<String, _>("cite_key").unwrap(),
            publisher_id: statement.read::<String, _>("publisher_id").unwrap(),
            month_year_id: statement.read::<String, _>("month_year_id").unwrap(),
            author: statement.read::<String, _>("author").unwrap(),
            title: statement.read::<String, _>("title").unwrap(),
            pages: statement.read::<String, _>("pages").unwrap(),
            volume: statement.read::<String, _>("volume").unwrap(),
            edition: statement.read::<String, _>("edition").unwrap(),
            year: statement.read::<String, _>("year").unwrap(),
            series: statement.read::<String, _>("series").unwrap(),
            publisher: statement.read::<String, _>("publisher").unwrap(),
            note: statement.read::<String, _>("note").unwrap(),
        });
    }
    Ok(parsed)
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
    /// Create and add `article` to `SQLite` database
    pub(crate) fn article_transaction(textarea: Vec<String>) {
        let master = MasterEntries::new_article();
        let publisher = Publisher::new(textarea[7].clone());
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
        let article = Article {
            cite_key: "n/a".to_string(),
            article_id: item_id.clone(),
            publisher_id: "n/a".to_string(),
            month_year_id: "n/a".to_string(),
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
            .expect("should bind id");
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

/// Read the `SQLite` database `article` table and returns a vector of `article` objects
// todo! test? Other tests test this logic
pub fn read_sqlite_article_table() -> Result<Vec<Article>, Error> {
    let connection = sqlite::open(DB_URL).unwrap();
    let query = "SELECT cite_key, article_id, publisher_id, month_year_id, title, journal, volume, pages, note, year, edition, publisher FROM article";
    let mut statement = connection.prepare(query).unwrap();
    let mut parsed = Vec::new();

    while let Ok(State::Row) = statement.next() {
        parsed.push(Article {
            cite_key: statement.read::<String, _>("cite_key").unwrap(),
            article_id: statement.read::<String, _>("article_id").unwrap(),
            publisher_id: statement.read::<String, _>("publisher_id").unwrap(),
            month_year_id: statement.read::<String, _>("month_year_id").unwrap(),
            title: statement.read::<String, _>("title").unwrap(),
            journal: statement.read::<String, _>("journal").unwrap(),
            pages: statement.read::<String, _>("pages").unwrap(),
            volume: statement.read::<String, _>("volume").unwrap(),
            note: statement.read::<String, _>("note").unwrap(),
            year: statement.read::<String, _>("year").unwrap(),
            edition: statement.read::<String, _>("edition").unwrap(),
            publisher: statement.read::<String, _>("publisher").unwrap(),
        });
    }
    Ok(parsed)
}

impl Publisher {
    pub fn new(vec: String) -> Publisher {
        let publisher_id = Uuid::new_v4().to_string();
        Publisher {
            publisher_id,
            publisher: vec,
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

#[cfg(test)]
mod test {
    /// To test on a blank database Initialize a different database by changing the DB_URL path
    /// and running `cargo run --bin init_db`
    ///
    /// A better way to do this may have been to have the init_db split into other files. Having
    /// the query str available from db.rs, and then passing in the desired database path passed
    /// into the init_db() function; then using a once_cell Lazy<Arc<sqlite::Connection>> to pass
    /// around the database connection.
    use super::*;
    use serial_test::serial;
    use sqlite::State::{Done, Row};

    fn exists(table: String, cite_key: String) -> bool {
        let connection = sqlite::open(DB_URL).unwrap();
        let query = format!("SELECT * FROM {} WHERE cite_key = ?", table);
        let mut statement = connection.prepare(query).unwrap();
        statement
            .bind_iter::<_, (_, Value)>([(1, cite_key.into())])
            .unwrap();

        match statement.next() {
            Ok(Row) => true,
            Ok(Done) => false,
            _ => false,
        }
    }
    #[test]
    #[serial]
    fn new_book_master_entries() {
        let x = MasterEntries::new_book();
        assert_eq!(x.entry_type, "BOOK".to_string());
        let y = MasterEntries::new_book();
        assert_ne!(x.cite_key, y.cite_key);
    }

    #[test]
    #[serial]
    fn new_article_master_entries() {
        let x = MasterEntries::new_article();
        assert_eq!(x.entry_type, "ARTICLE".to_string());
        let y = MasterEntries::new_article();
        assert_ne!(x.cite_key, y.cite_key);
    }

    #[test]
    #[serial]
    fn insert_and_delete_master_entries() {
        let x = MasterEntries::new_book();
        let actual = x.insert();
        // assert_eq!(false, actual.is_ok());
        assert_eq!(Done, actual.unwrap());
        let result = exists("master_entries".to_string(), x.cite_key.clone());
        assert_eq!(result, true, "you are here");
        MasterEntries::delete(x.cite_key.clone()).expect("TODO: panic message");
        let result = exists("master_entries".to_string(), x.cite_key.clone());
        assert_eq!(result, false, "you are not here");
    }

    // Note: Book::book_transaction() and Article::article_transaction() are not tested, but the
    // individual elements of the functions are

    #[test]
    #[serial]
    // Tests two trait functions BookInsert insert() and RowDelete delete() for book
    // Tests Book transaction delete_book()
    fn insert_and_delete_new_book() {
        let book_textarea: Vec<String> = vec![
            "New Author".to_string(),
            "New Title".to_string(),
            "300 pages".to_string(),
            "Volume 1".to_string(),
            "1st edition".to_string(),
            "2023".to_string(),
            "New Series".to_string(),
            "New Publisher".to_string(),
            "New Note".to_string(),
        ];
        // Note: Book::book_transaction() is not tested, but the individual elements of the function are
        let b = Book {
            book_id: Uuid::new_v4().to_string(),
            cite_key: Uuid::new_v4().to_string(),
            publisher_id: Uuid::new_v4().to_string(),
            month_year_id: Uuid::new_v4().to_string(),
            author: book_textarea[0].clone(),
            title: book_textarea[1].clone(),
            pages: book_textarea[2].clone(),
            volume: book_textarea[3].clone(),
            edition: book_textarea[4].clone(),
            year: book_textarea[5].clone(),
            series: book_textarea[6].clone(),
            publisher: book_textarea[7].clone(),
            note: book_textarea[8].clone(),
        };

        // Test that the the result == State<Done>
        let actual = b.insert();
        assert_eq!(true, actual.is_ok());
        assert_eq!(Done, actual.unwrap());

        // Test that the item is inserted into the `book` table
        let result = exists("book".to_string(), b.cite_key.clone());
        assert_eq!(result, true, "you are here");

        // Test that the item is removed from the `book` table
        let _ = Book::delete(b.cite_key.clone());
        let result = exists("book".to_string(), b.cite_key.clone());
        assert_eq!(result, false, "you are not here");
    }

    #[test]
    #[serial]
    // Tests two trait functions RowSelect select() and RowUpdate update() for book
    fn select_and_update_book() {
        // Instantiate book object and insert to database
        let book_textarea: Vec<String> = vec![
            "New Author".to_string(),
            "New Title".to_string(),
            "300 pages".to_string(),
            "Volume 1".to_string(),
            "1st edition".to_string(),
            "2023".to_string(),
            "New Series".to_string(),
            "New Publisher".to_string(),
            "New Note".to_string(),
        ];

        let b = Book {
            book_id: Uuid::new_v4().to_string(),
            cite_key: Uuid::new_v4().to_string(),
            publisher_id: Uuid::new_v4().to_string(),
            month_year_id: Uuid::new_v4().to_string(),
            author: book_textarea[0].clone(),
            title: book_textarea[1].clone(),
            pages: book_textarea[2].clone(),
            volume: book_textarea[3].clone(),
            edition: book_textarea[4].clone(),
            year: book_textarea[5].clone(),
            series: book_textarea[6].clone(),
            publisher: book_textarea[7].clone(),
            note: book_textarea[8].clone(),
        };
        let _ = b.insert();

        // New vec of strings to update
        let new_book_textarea: Vec<String> = vec![
            "NewNew Author".to_string(),
            "NewNew Title".to_string(),
            "300 pages".to_string(),
            "Volume 1".to_string(),
            "1st edition".to_string(),
            "2024".to_string(),
            "NewNew Series".to_string(),
            "NewNew Publisher".to_string(),
            "NewNew Note".to_string(),
        ];
        // Update original book with new vec of strings
        let _ = Book::book_update(new_book_textarea, b.cite_key.clone());

        // Find book with original cite_key and verify that the data is updated
        let found = Book::select(b.cite_key.as_str());
        assert_eq!(found[0], "NewNew Author".to_string());
        assert_eq!(found[1], "NewNew Title".to_string());
        assert_eq!(found[2], "300 pages".to_string());
        assert_eq!(found[3], "Volume 1".to_string());
        assert_eq!(found[4], "1st edition".to_string());
        assert_eq!(found[5], "2024".to_string());
        assert_eq!(found[6], "NewNew Series".to_string());
        assert_eq!(found[7], "NewNew Publisher".to_string());
        assert_eq!(found[8], "NewNew Note".to_string());
    }

    #[test]
    #[serial]
    // Tests two trait functions ArticleInsert insert() and RowDelete delete() for article
    // Tests Article transaction delete_book()
    fn insert_and_delete_new_article() {
        let article_textarea: Vec<String> = vec![
            "New Title".to_string(),
            "New Journal".to_string(),
            "Volume 1".to_string(),
            "300-305".to_string(),
            "New Note".to_string(),
            "2023".to_string(),
            "New Edition".to_string(),
            "New Publisher".to_string(),
        ];
        let a = Article {
            article_id: Uuid::new_v4().to_string(),
            cite_key: Uuid::new_v4().to_string(),
            publisher_id: Uuid::new_v4().to_string(),
            month_year_id: Uuid::new_v4().to_string(),
            title: article_textarea[0].clone(),
            journal: article_textarea[1].clone(),
            volume: article_textarea[2].clone(),
            pages: article_textarea[3].clone(),
            note: article_textarea[4].clone(),
            year: article_textarea[5].clone(),
            edition: article_textarea[6].clone(),
            publisher: article_textarea[7].clone(),
        };

        // Test that the the result == State<Done>
        let actual = a.insert();
        assert_eq!(true, actual.is_ok());
        assert_eq!(Done, actual.unwrap());

        // Test that the item is inserted into the `article` table
        let result = exists("article".to_string(), a.cite_key.clone());
        assert_eq!(result, true, "you are here");

        // Test that the item is removed from the `article` table
        let _ = Article::delete(a.cite_key.clone());
        let result = exists("article".to_string(), a.cite_key.clone());
        assert_eq!(result, false, "you are not here");
    }

    #[test]
    #[serial]
    // Tests two trait functions RowSelect select() and RowUpdate update() for article
    fn select_and_update_article() {
        // Instantiate article object and insert to database
        let article_textarea: Vec<String> = vec![
            "New Title".to_string(),
            "New Journal".to_string(),
            "Volume 1".to_string(),
            "300-305".to_string(),
            "New Note".to_string(),
            "2023".to_string(),
            "New Edition".to_string(),
            "New Publisher".to_string(),
        ];

        let a = Article {
            article_id: Uuid::new_v4().to_string(),
            cite_key: Uuid::new_v4().to_string(),
            publisher_id: Uuid::new_v4().to_string(),
            month_year_id: Uuid::new_v4().to_string(),
            title: article_textarea[0].clone(),
            journal: article_textarea[1].clone(),
            volume: article_textarea[2].clone(),
            pages: article_textarea[3].clone(),
            note: article_textarea[4].clone(),
            year: article_textarea[5].clone(),
            edition: article_textarea[6].clone(),
            publisher: article_textarea[7].clone(),
        };
        let _ = a.insert();

        // New vec of strings to update
        let new_article_textarea: Vec<String> = vec![
            "NewNew Title".to_string(),
            "NewNew Journal".to_string(),
            "Volume 1".to_string(),
            "300-305".to_string(),
            "NewNew Note".to_string(),
            "2024".to_string(),
            "NewNew Edition".to_string(),
            "NewNew Publisher".to_string(),
        ];
        // Update original article with new vec of strings
        let _ = Article::article_update(new_article_textarea, a.cite_key.clone());

        // Find article with original cite_key and verify that the data is updated
        let found = Article::select(a.cite_key.as_str());
        assert_eq!(found[0], "NewNew Title".to_string());
        assert_eq!(found[1], "NewNew Journal".to_string());
        assert_eq!(found[2], "Volume 1".to_string());
        assert_eq!(found[3], "300-305".to_string());
        assert_eq!(found[4], "NewNew Note".to_string());
        assert_eq!(found[5], "2024".to_string());
        assert_eq!(found[6], "NewNew Edition".to_string());
        assert_eq!(found[7], "NewNew Publisher".to_string());
    }

    #[test]
    #[serial]
    fn new_month_year() {
        let x = MonthYear::new("1923".to_string());
        let y = MonthYear::new("2023".to_string());
        assert_eq!(x.year, "1923".to_string());
        assert_eq!(y.year, "2023".to_string());
        assert_ne!(x.month, y.month_year_id)
    }

    #[test]
    #[serial]
    fn month_year_insert() {
        let x = MonthYear::new("2023".to_string());
        let _ = x.insert();
        let connection = sqlite::open(DB_URL).unwrap();
        let query = "SELECT * FROM month_year WHERE month_year_id = :month_year_id";
        let mut statement = connection.prepare(query).unwrap();
        statement
            .bind_iter::<_, (_, Value)>([(1, x.month_year_id.into())])
            .unwrap();

        let result = statement.next();
        assert_eq!(true, result.is_ok());
        assert_eq!(Row, result.unwrap());
    }

    #[test]
    #[serial]
    fn new_publisher() {
        let x = Publisher::new("New Publisher".to_string());
        let y = Publisher::new("Other New Publisher".to_string());
        assert_eq!(x.publisher, "New Publisher".to_string());
        assert_eq!(y.publisher, "Other New Publisher".to_string());
        assert_ne!(x.publisher_id, y.publisher_id);
    }

    #[test]
    #[serial]
    fn publisher_insert() {
        let x = Publisher::new("New Publisher".to_string());
        let _ = x.insert();
        let connection = sqlite::open(DB_URL).unwrap();
        let query = "SELECT * FROM publisher WHERE publisher_id = :publisher_id";
        let mut statement = connection.prepare(query).unwrap();
        statement
            .bind_iter::<_, (_, Value)>([(1, x.publisher_id.into())])
            .unwrap();

        let result = statement.next();
        assert_eq!(true, result.is_ok());
        assert_eq!(Row, result.unwrap());
    }
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

// Implement Later
// #[derive(Clone, Debug)]
// pub struct Organizations {
//     pub(crate) organization_id: String,
//     pub(crate) organization: String,
//     pub(crate) address: String,
// }

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
