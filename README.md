# bib_db

A Bibliographic Relational Database built in Rust. A basic implementation to store metadata about Books and Articles in
the way a University library would keep track of what items they have available to their students. Other uses would be
for public libraries, medical libraries. These are called Library Catalogs... think of it as a digital version of a '
card catalog'.

This is a TUI (Terminal User Interface) program that allows a user to store and retrieve data from an SQLite database.
Still in development phase.

[demo.mp4](demo.mp4)

# First usage

For the first time using the application, initialize an SQLite database using `cargo run --bin init_db`
This will create the database and all the relational tables needed to store data about different types of bibliographic
data

# General usage

Initialize the TUI by using `cargo run --bin bib_db`
Inside the TUI the menu shows the following options:

- `Home` Introduction page
    - Displays hot keys to navigate the menu
- `Show Books` Display a list of books
    - Use up and down arrow keys to move through the list of books in the database
    - Use `Ctrl-D` to delete the selected book
    - Use `Ctrl-U` to update the selected book
- `Book Add` Add a new book
    - To begin editing press `F2`
    - After you have entered all data and exited editing mode press `F9` to save the book to the database
    - To exit editing press `F12` changes wil lnto be saved
    - Required fields for the database are labeled red
    - Other fields are optional
- `List Articles` Display a list of articles
    - Use up and down arrow keys to move through the list of books in the database
    - Use `Ctrl-D` to delete the selected article
    - Use `Ctrl-U` to delete the selected article
- `Article Add` Add a new article
    - To begin editing press `F2`
    - After you have entered all data and exited editing mode press `F9` to save the article to the database
    - To exit editing press `F12` changes will not be saved
    - Required fields for the database are labeled red
    - Other fields are optional
- `Quit`
    - Exit the program
    - Must not be in editing mode to quit. If you are in editing mode press `F12` to exit then pres `Q` to quit

# Tables created

- Master Entries
-
    - Automatically generates a unique cite_key. Also creates an entry_type based on the item type: Book or Article

- Books
-
    - Automatically generates a unique book_id, cite_key (reference), publisher_id (reference), month_year_id (
      reference)
-
    - Store a Book item with the following data: title, editor, pages, volume, edition, series, note

- Article
-
    - Automatically generates a unique cite_key (reference), article_id, publisher_id (reference), month_year_id (
      reference)
-
    - Stores the following data: title, journal, volume, pages, note, edition

- Publisher
-
    - Automatically generates a unique publisher_id
-
    - Stores the following data: publisher, address

- Month Year
-
    - Automatically generates a unique month_year_id
-
    - Stores the following data: month, year

# To implement later

These tables should be implemented later on to complete the Relational Database structure

- Author
-
    - Automatically generates a unique cite_key (reference), author_id
-
    - Stores an author name

- Relationship
-
    - Automatically generates a unique parent_id, cite_key (reference), child_id

- Organizations
-
    - Automatically generates a unique organization_id
-
    - Stores the following data: organization, address

Add other item types: such as 'Audiobook', 'OnlineResource', 'Photograph', 'Painting', etc.

Add catalog searching, and indexing of items

# Resources used to build the application

- ratatui: https://docs.rs/ratatui/latest/ratatui/ Crate for terminal user interface
- crossterm: https://docs.rs/crossterm/latest/crossterm/ Terminal manipulation library
- sqlite: https://docs.rs/sqlite/latest/sqlite/crate Crate to interface with SQLite (Examples were not very explicit,
  had some help from Shane Perry understanding how to implement the different structs)
- TUI example: https://blog.logrocket.com/rust-and-tui-building-a-command-line-interface-in-rust/#handlinginputintui
    - This blog post helped me get a good start on setting up the terminal and render loop for the TUI.
    - I had to change lots of things to fit my needs. I also refactored it so that it was not all in a main function.
