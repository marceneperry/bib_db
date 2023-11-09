/// Functions for initializing SQLite database
use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};


// todo! add  DB_State?

/// First time using the application you can run this init_db main.
/// It will setup the database and initialize the base tables outlined in the migrations folder.
#[tokio::main]
async fn main() {
    let db = init_db().await;
    let _ = init_table(&db).await;
}


const DB_URL: &str = "sqlite://bibliographic_db/bib_data.db";

/// Initialize the database
async fn init_db() -> SqlitePool {
// initialize db if needed
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    SqlitePool::connect(DB_URL).await.unwrap()
}


/// Initialize the request table in the SQLite database.
async fn init_table(db: &Pool<Sqlite>) {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations = std::path::Path::new(&crate_dir).join("./migrations");

    let migration_results = sqlx::migrate::Migrator::new(migrations)
        .await
        .unwrap()
        .run(db)
        .await;

    match migration_results {
        Ok(_) => println!("Migration success"),
        Err(error) => {
            panic!("{}", error);
        }
    }
}

/// Database connection
async fn connect() -> Pool<Sqlite> {
    SqlitePool::connect(DB_URL).await.unwrap()
}
