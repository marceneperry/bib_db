# Bibliographic Record Database Definition

- This will be binary that will define the database table schemas for a bibliographic database, used in library management systems.
- My background is in Bibliographic cataloging. I know how these databases are used in the real world. Implementing the database definitions will help me to understand the inner workings of databases, as well as understand and utilize the tools in Rust that make it a good fit for database applications.

## Crates

- `diesel` (Object-Relations Mapper) for database interaction with sqlite3; does not support async.
- `sea-orm` similar to diesel, but supports async.
- `sqlx` 
- `ratatui` for terminal user interface

## Resources

- Blog on setting up diesel and basic database setup: [Working with SQL Databases in Rust](https://www.makeuseof.com/working-with-sql-databases-in-rust/)
- ISBD rules: If I want to make sure data is standardized, [ISBD PDF](https://www.ifla.org/wp-content/uploads/2019/05/assets/cataloguing/isbd/isbd-cons_20110321.pdf)
- Database design thesis: lays out what to include for each table, for each type of medium, [Database Design Thesis](https://digitalscholarship.unlv.edu/cgi/viewcontent.cgi?article=3240&context=rtds)

## Basic Implementation

### Tables (First pass)

- MASTER ENTRIES (entire collection of items)
- BOOK
- ARTICLE
- MAP
- RELATIONSHIP
- AUTHOR
- PUBLISHER
- ORGANIZATION
- MONTH_YEAR

### Tables (Second pass: more media to add to the database)

- SERIALS (journals: physical, online)
- RECORDING (audio)
- RECORDING (video)