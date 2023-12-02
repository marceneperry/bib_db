use crate::db::{Article, Book, RowSelect};
use crate::DB_URL;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::disable_raw_mode;
use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::prelude::{Alignment, Color, Constraint, Direction, Layout, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Tabs};
use ratatui::Terminal;
use sqlite::State;
use std::io::Error;
use std::iter::Cloned;
use std::rc::Rc;
use std::slice::Iter;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tui_textarea::TextArea;

// currently only adding new items. later add ability to search and edit items.
// todo! implement show ALL items sorted by cite_key?

#[derive(Copy, Clone, Debug)]
pub(crate) enum InputMode {
    Command,
    Input,
}

#[derive(Debug)]
pub(crate) enum AppEvent<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum MenuItem {
    // todo! future version do sub menus:
    // `Home` `Display All Items` `Books` `Articles` `Quit`
    // Books: `Display All` `Add New Book` `Delete Book` `Find Book`
    // Articles: `Display All` `Add New Article` `Delete Article` `Find Article`
    Home,
    ShowBooks,
    NewBook(InputMode),
    ListArticles,
    InsertArticle(InputMode),
}

impl MenuItem {
    fn ordinal(&self) -> usize {
        match self {
            MenuItem::Home => 0,
            MenuItem::ShowBooks => 1,
            MenuItem::NewBook(_) => 2,
            MenuItem::ListArticles => 3,
            MenuItem::InsertArticle(_) => 4,
        }
    }
}

#[derive(Clone)]
pub struct App {
    pub menu_titles: Vec<&'static str>,
    pub index: usize,
    active_menu_item: MenuItem,
    pub book_list_state: Arc<Mutex<ListState>>,
    pub article_list_state: Arc<Mutex<ListState>>,
    update_item_id: String,
    update_flag: bool,
}

impl App {
    pub fn new() -> App {
        App {
            menu_titles: vec![
                "Home",
                "Show Books",
                "Book Add",
                "List Articles",
                "Article Add",
                "Quit",
            ],
            index: 0,
            active_menu_item: MenuItem::Home,
            book_list_state: Arc::new(Mutex::new(ListState::default())),
            article_list_state: Arc::new(Mutex::new(ListState::default())),
            update_item_id: "64406381-99a9-44bf-9bfc-a594ddd13356".to_string(),
            update_flag: false,
        }
    }

    // Run the terminal loop with event handlers
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Error> {
        // setup mpsc to handle the channels in the rendering loop
        let (tx, rx) = mpsc::channel();
        let tick_rate = Duration::from_millis(200);
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                if let Event::Key(key) = event::read().expect("can read events") {
                    if key.kind == KeyEventKind::Press {
                        tx.send(AppEvent::Input(Event::Key(key)))
                            .expect("can send events");
                    }
                }

                if last_tick.elapsed() >= tick_rate && tx.send(AppEvent::Tick).is_ok() {
                    last_tick = Instant::now();
                }
            }
        });

        let mut book_text_area = TextArea::default();
        let mut article_text_area = TextArea::default();

        loop {
            let terminal_size = terminal.size().expect("can size terminal");
            let menu_titles = self.menu_titles.iter().cloned();
            let active_menu_item = self.active_menu_item;
            let book_list_state = self.book_list_state.clone();
            let article_list_state = self.article_list_state.clone();
            book_text_area.set_block(App::new_book_block(self.update_flag.clone()));
            let book_text_widget = book_text_area.widget();
            article_text_area.set_block(App::new_article_block(self.update_flag));
            let article_text_widget = article_text_area.widget();

            terminal.draw(move |frame| {
                let chunks = App::panes(terminal_size);

                // Main menu section
                frame.render_widget(
                    App::menu(menu_titles, active_menu_item.ordinal()),
                    chunks[0],
                );

                // Change to a different menu item
                match active_menu_item {
                    MenuItem::Home => {
                        let (left, right) = App::render_home();
                        frame.render_widget(left, App::home_panes(chunks.clone())[0]);
                        frame.render_widget(right, App::home_panes(chunks.clone())[1]);
                    }
                    MenuItem::ShowBooks => {
                        let mut lock = book_list_state.lock().expect("can lock state");
                        if lock.selected().is_none() {
                            lock.select(Some(0));
                        }
                        drop(lock);

                        let (left, middle, right) = App::render_books(book_list_state.clone());
                        let mut lock = book_list_state.lock().expect("can lock state");
                        frame.render_stateful_widget(
                            left,
                            App::show_panes(chunks.clone())[0],
                            &mut *lock,
                        );
                        frame.render_widget(middle, App::show_panes(chunks.clone())[1]);
                        frame.render_widget(right, App::show_panes(chunks.clone())[2]);
                        drop(lock);
                    }
                    MenuItem::NewBook(..) => {
                        frame.render_widget(
                            App::render_add_book(),
                            App::add_panes(chunks.clone())[0],
                        );
                        frame.render_widget(book_text_widget, App::add_panes(chunks.clone())[1]);
                    }
                    MenuItem::ListArticles => {
                        let mut lock = article_list_state.lock().expect("can lock state");
                        if lock.selected().is_none() {
                            lock.select(Some(0));
                        }
                        drop(lock);

                        let (left, middle, right) =
                            App::render_articles(article_list_state.clone());
                        let mut lock = article_list_state.lock().expect("can lock state");
                        frame.render_stateful_widget(
                            left,
                            App::show_panes(chunks.clone())[0],
                            &mut *lock,
                        );
                        frame.render_widget(middle, App::show_panes(chunks.clone())[1]);
                        frame.render_widget(right, App::show_panes(chunks.clone())[2]);
                        drop(lock);
                    }
                    MenuItem::InsertArticle(..) => {
                        frame.render_widget(
                            App::render_add_article(),
                            App::add_panes(chunks.clone())[0],
                        );
                        frame.render_widget(article_text_widget, App::add_panes(chunks.clone())[1]);
                    }
                }

                // Copyright section
                frame.render_widget(App::copyright(), chunks[2]);
            })?;

            // Match key events to move around the app menu and edit in text areas
            match rx.recv().unwrap() {
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Char('q'), // Quit
                    ..
                })) if self.is_command_mode() => {
                    disable_raw_mode().expect("can disable raw mode");
                    terminal.show_cursor().expect("can show cursor");
                    break;
                }
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Char('h'), // Home menu
                    ..
                })) if self.is_command_mode() => self.active_menu_item = MenuItem::Home,
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Char('s'), // Show list of books
                    ..
                })) if self.is_command_mode() => self.active_menu_item = MenuItem::ShowBooks,
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Char('b'), // Add a new book
                    ..
                })) if self.is_command_mode() => {
                    self.active_menu_item = MenuItem::NewBook(InputMode::Command)
                }
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Char('l'), // Show a list of articles
                    ..
                })) if self.is_command_mode() => self.active_menu_item = MenuItem::ListArticles,
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Char('a'), // Add a new article
                    ..
                })) if self.is_command_mode() => {
                    self.active_menu_item = MenuItem::InsertArticle(InputMode::Command)
                }
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::F(2), // Enter edit mode
                    ..
                })) => {
                    self.enter_input_mode()
                }
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::F(12), // Exit edit mode
                    ..
                })) => {
                    self.update_flag = false;
                    self.exit_input_mode()
                }
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::F(9), // Save to database
                    ..
                })) => {
                    if let MenuItem::NewBook(_) = self.active_menu_item {
                        self.save_as_item_type(&book_text_area);
                        book_text_area = TextArea::default();
                        book_text_area.set_block(App::new_book_block(false));
                    } else if let MenuItem::InsertArticle(_) = self.active_menu_item {
                        self.save_as_item_type(&article_text_area);
                        article_text_area = TextArea::default();
                        article_text_area.set_block(App::new_article_block(false));
                    }
                    self.update_flag = false;
                    self.exit_input_mode()
                }
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Char('u'), // Update selected item
                    modifiers,
                    ..
                })) if KeyModifiers::CONTROL == modifiers => {
                    if let MenuItem::ShowBooks = self.active_menu_item {
                        self.update_flag = true;
                        self.get_item_id();
                        let text_vec = Book::select(&self.update_item_id);
                        book_text_area = TextArea::new(text_vec);
                        book_text_area.set_block(App::new_book_block(self.update_flag));
                        self.active_menu_item = MenuItem::NewBook(InputMode::Input);
                    } else if let MenuItem::ListArticles = self.active_menu_item {
                        self.update_flag = true;
                        self.get_item_id();
                        let text_vec = Article::select(&self.update_item_id);
                        article_text_area = TextArea::new(text_vec);
                        article_text_area.set_block(App::new_book_block(self.update_flag));
                        self.active_menu_item = MenuItem::InsertArticle(InputMode::Input);
                    }
                }
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Char('d'), // Delete selected item
                    modifiers,
                    ..
                })) if KeyModifiers::CONTROL == modifiers => {
                    if let MenuItem::ShowBooks = self.active_menu_item {
                        let book_list = App::read_sqlite_book_table().expect("can fetch book list");
                        if book_list.is_empty() {
                        } else {
                            self.get_item_id();
                            Book::delete_book(self.update_item_id.clone());

                            // if last item in list move selected item back to top of list
                            let mut lock = self.book_list_state.lock().expect("can lock state");
                            if let Some(selected) = lock.selected() {
                                let amount_books = App::read_sqlite_book_table()
                                    .expect("can fetch book list")
                                    .len();
                                if amount_books == 0 {}
                                else if selected >= amount_books - 1 {
                                    lock.select(Some(0));
                                } else {
                                    lock.select(Some(selected + 1));
                                }
                            }
                        }
                    } else if let MenuItem::ListArticles = self.active_menu_item {
                        let article_list = App::read_sqlite_article_table().expect("can fetch book list");
                        if article_list.is_empty() {
                        } else {
                            self.get_item_id();
                            Article::delete_article(self.update_item_id.clone());

                            // if last item in list move selected item back to top of list
                            let mut lock = self.article_list_state.lock().expect("can lock state");
                            if let Some(selected) = lock.selected() {
                                let amount_books = App::read_sqlite_article_table()
                                    .expect("can fetch book list")
                                    .len();
                                if amount_books == 0 {}
                                else if selected >= amount_books - 1 {
                                    lock.select(Some(0));
                                } else {
                                    lock.select(Some(selected + 1));
                                }
                            }
                        }
                    }
                }
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Down, .. // Move down in the list of books or articles; wraps around
                })) if self.is_command_mode() => {
                    if let MenuItem::ShowBooks = self.active_menu_item {
                        let mut lock = self.book_list_state.lock().expect("can lock state");
                        if let Some(selected) = lock.selected() {
                            let amount_books = App::read_sqlite_book_table()
                                .expect("can fetch book list")
                                .len();
                            if selected >= amount_books - 1 {
                                lock.select(Some(0));
                            } else {
                                lock.select(Some(selected + 1));
                            }
                        }
                        drop(lock);
                    } else if let MenuItem::ListArticles = self.active_menu_item {
                        let mut lock = self.article_list_state.lock().expect("can lock state");
                        if let Some(selected) = lock.selected() {
                            let amount_articles = App::read_sqlite_article_table()
                                .expect("can fetch book list")
                                .len();
                            if selected >= amount_articles - 1 {
                                lock.select(Some(0));
                            } else {
                                lock.select(Some(selected + 1));
                            }
                        }
                        drop(lock)
                    }
                }
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Up, .. // Move up in the list of books or articles; wraps around
                })) if self.is_command_mode() => {
                    if let MenuItem::ShowBooks = self.active_menu_item {
                        let mut lock = self.book_list_state.lock().expect("can lock state");
                        if let Some(selected) = lock.selected() {
                            let amount_books = App::read_sqlite_book_table()
                                .expect("can fetch book list")
                                .len();
                            if selected > 0 {
                                lock.select(Some(selected - 1));
                            } else {
                                lock.select(Some(amount_books - 1));
                            }
                        }
                        drop(lock)
                    } else if let MenuItem::ListArticles = self.active_menu_item {
                        let mut lock = self.article_list_state.lock().expect("can lock state");
                        if let Some(selected) = lock.selected() {
                            let amount_articles = App::read_sqlite_article_table()
                                .expect("can fetch book list")
                                .len();
                            if selected > 0 {
                                lock.select(Some(selected - 1));
                            } else {
                                lock.select(Some(amount_articles - 1));
                            }
                        }
                        drop(lock)
                    }
                }
                AppEvent::Tick => {}
                AppEvent::Input(input) if !self.is_command_mode() => {
                    // Text area input mode
                    if let MenuItem::NewBook(InputMode::Input) = self.active_menu_item {
                        book_text_area.input(input);
                    } else if let MenuItem::InsertArticle(InputMode::Input) = self.active_menu_item
                    {
                        article_text_area.input(input);
                    }
                }
                _ => {}
            };
        }
        Ok(())
    }

    /// Retrieves cite_key of current item (book or article); Used to update or delete an item.
    fn get_item_id(&mut self) {
        self.update_item_id = "".to_string();
        if let MenuItem::ShowBooks = self.active_menu_item {
            let book_list = App::read_sqlite_book_table().expect("can fetch book list");
            let selected = self
                .book_list_state
                .lock()
                .expect("can lock list state")
                .selected();
            let selected_item = book_list
                .get(selected.unwrap_or(0))
                .expect("exists")
                .clone();
            self.update_item_id = selected_item.cite_key.clone();
        } else if let MenuItem::ListArticles = self.active_menu_item {
            let article_list = App::read_sqlite_article_table().expect("can fetch book list");
            let selected = self
                .article_list_state
                .lock()
                .expect("can lock list state")
                .selected();
            let selected_item = article_list
                .get(selected.unwrap_or(0))
                .expect("exists")
                .clone();
            self.update_item_id = selected_item.cite_key;
        }
    }

    /// Saves the data entered in the textarea to Book or Article table
    fn save_as_item_type(&mut self, text_area: &TextArea) {
        let mut text_vec = Vec::new();
        for line in text_area.lines() {
            text_vec.push(line.to_string());
        }
        if let MenuItem::NewBook(_) = self.active_menu_item {
            if self.update_flag == false {
                Book::book_transaction(text_vec);
            } else {
                Book::book_update(text_vec, self.update_item_id.clone());
            }
        } else if let MenuItem::InsertArticle(_) = self.active_menu_item {
            if self.update_flag == false {
                Article::article_transaction(text_vec);
            } else {
                Article::article_update(text_vec, self.update_item_id.clone());
            }
        }
    }

    /// Change the state of the app from Command mode to Input mode
    fn enter_input_mode(&mut self) {
        if let MenuItem::NewBook(InputMode::Command) = self.active_menu_item {
            self.active_menu_item = MenuItem::NewBook(InputMode::Input)
        }
        if let MenuItem::InsertArticle(InputMode::Command) = self.active_menu_item {
            self.active_menu_item = MenuItem::InsertArticle(InputMode::Input)
        }
    }

    /// Change the state of the app from Input mode to Command mode
    fn exit_input_mode(&mut self) {
        if let MenuItem::NewBook(InputMode::Input) = self.active_menu_item {
            self.active_menu_item = MenuItem::NewBook(InputMode::Command)
        } else if let MenuItem::InsertArticle(InputMode::Input) = self.active_menu_item {
            self.active_menu_item = MenuItem::InsertArticle(InputMode::Command)
        }
    }

    /// Check if the app is in command mode or input mode
    fn is_command_mode(&self) -> bool {
        !matches!(
            self.active_menu_item,
            MenuItem::NewBook(InputMode::Input) | MenuItem::InsertArticle(InputMode::Input)
        ) // cool clippy suggestion!
    }

    /// Read the sqlite database book table and returns a vector of book objects
    fn read_sqlite_book_table() -> Result<Vec<Book>, Error> {
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
            })
        }
        Ok(parsed)
    }

    /// Read the sqlite database article table and returns a vector of article objects
    fn read_sqlite_article_table() -> Result<Vec<Article>, Error> {
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
            })
        }
        Ok(parsed)
    }

    /// UI for adding a new article
    fn render_add_article() -> Paragraph<'static> {
        return Paragraph::new(vec![
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                "Title: ",
                Style::default().fg(Color::LightRed),
            )]),
            Line::from(vec![Span::styled(
                "Journal: ",
                Style::default().fg(Color::LightRed),
            )]),
            Line::from(vec![Span::styled(
                "Volume: ",
                Style::default().fg(Color::LightRed),
            )]),
            Line::from(vec![Span::styled(
                "Pages: ",
                Style::default().fg(Color::LightBlue),
            )]),
            Line::from(vec![Span::styled(
                "Note: ",
                Style::default().fg(Color::LightBlue),
            )]),
            Line::from(vec![Span::styled(
                "Year: ",
                Style::default().fg(Color::LightRed),
            )]),
            Line::from(vec![Span::styled(
                "Edition: ",
                Style::default().fg(Color::LightBlue),
            )]),
            Line::from(vec![Span::styled(
                "Publisher: ",
                Style::default().fg(Color::LightRed),
            )]),
            Line::from(vec![Span::styled("", Style::default())]),
            Line::from(vec![Span::styled(
                "Required input is red ",
                Style::default().fg(Color::LightRed),
            )]),
            Line::from(vec![Span::styled(
                "Optional input is blue ",
                Style::default().fg(Color::LightBlue),
            )]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                "Press 'F2' to start editing ",
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(vec![Span::styled(
                "Press 'F12' to stop editing ",
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(vec![Span::styled(
                "Press 'F9' to save to database ",
                Style::default().fg(Color::Cyan),
            )]),
        ])
        .alignment(Alignment::Right);
    }

    /// UI for adding a new book
    fn render_add_book() -> Paragraph<'static> {
        return Paragraph::new(vec![
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                "Author: ",
                Style::default().fg(Color::LightRed),
            )]),
            Line::from(vec![Span::styled(
                "Title: ",
                Style::default().fg(Color::LightRed),
            )]),
            Line::from(vec![Span::styled(
                "Pages: ",
                Style::default().fg(Color::LightRed),
            )]),
            Line::from(vec![Span::styled(
                "Volume: ",
                Style::default().fg(Color::LightBlue),
            )]),
            Line::from(vec![Span::styled(
                "Edition: ",
                Style::default().fg(Color::LightBlue),
            )]),
            Line::from(vec![Span::styled(
                "Year: ",
                Style::default().fg(Color::LightRed),
            )]),
            Line::from(vec![Span::styled(
                "Series: ",
                Style::default().fg(Color::LightBlue),
            )]),
            Line::from(vec![Span::styled(
                "Publisher: ",
                Style::default().fg(Color::LightRed),
            )]),
            Line::from(vec![Span::styled(
                "Note: ",
                Style::default().fg(Color::LightBlue),
            )]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                "Required input is red ",
                Style::default().fg(Color::LightRed),
            )]),
            Line::from(vec![Span::styled(
                "Optional input is blue ",
                Style::default().fg(Color::LightBlue),
            )]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                "Press 'F2' to start editing ",
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(vec![Span::styled(
                "Press 'F12' to stop editing ",
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(vec![Span::styled(
                "Press 'F9' to save to database ",
                Style::default().fg(Color::Cyan),
            )]),
        ])
        .alignment(Alignment::Right);
    }

    /// UI for rendering all articles in the database
    fn render_articles(
        article_list_state: Arc<Mutex<ListState>>,
    ) -> (List<'static>, Paragraph<'static>, Paragraph<'static>) {
        let header = Paragraph::new(vec![
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                "ID ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Title ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Journal ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Volume ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Pages ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Note ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Year ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Edition ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Publisher ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
        ])
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        );

        let articles = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Articles   Delete selected book with `Ctrl-D`")
            .border_type(BorderType::Plain);

        let items: Vec<_> = App::read_sqlite_article_table()
            .expect("can fetch article list")
            .iter()
            .map(|article| {
                ListItem::new(Line::from(vec![Span::styled(
                    article.title.clone(),
                    Style::default(),
                )]))
            })
            .collect();

        let list = List::new(items).block(articles).highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

        let article_list = App::read_sqlite_article_table().expect("can fetch article list");
        if article_list.is_empty() {
            let article_detail = Paragraph::default();
            return (list, header, article_detail);
        }

        let selected = article_list_state
            .lock()
            .expect("can lock article state")
            .selected();
        let selected_article = article_list
            .get(selected.unwrap_or(0))
            .expect("exists")
            .clone();

        let article_detail = Paragraph::new(vec![
            Line::from(Span::raw(selected_article.article_id)),
            Line::from(Span::raw(selected_article.title)),
            Line::from(Span::raw(selected_article.journal)),
            Line::from(Span::raw(selected_article.volume)),
            Line::from(Span::raw(selected_article.pages)),
            Line::from(Span::raw(selected_article.note)),
            Line::from(Span::raw(selected_article.year)),
            Line::from(Span::raw(selected_article.edition)),
            Line::from(Span::raw(selected_article.publisher)),
        ])
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Article Detail")
                .border_type(BorderType::Plain),
        );

        (list, header, article_detail)
    }

    /// UI for rendering all books in the database
    fn render_books(
        book_list_state: Arc<Mutex<ListState>>,
    ) -> (List<'static>, Paragraph<'static>, Paragraph<'static>) {
        let header = Paragraph::new(vec![
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                "ID ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Author ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Title ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Pages ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Volume ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Edition ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Year ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Series ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Publisher ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Note ",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )]),
        ])
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        );

        let books = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Books   Delete selected book with `Ctrl-D`")
            .border_type(BorderType::Plain);

        let items: Vec<_> = App::read_sqlite_book_table()
            .expect("can fetch book list")
            .iter()
            .map(|book| {
                ListItem::new(Line::from(vec![Span::styled(
                    book.title.clone(),
                    Style::default(),
                )]))
            })
            .collect();

        let list = List::new(items).block(books).highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

        let book_list = App::read_sqlite_book_table().expect("can fetch book list");
        if book_list.is_empty() {
            let book_detail = Paragraph::default();
            return (list, header, book_detail);
        }

        let selected = book_list_state.lock().expect("can lock state").selected();
        let selected_book = book_list
            .get(selected.unwrap_or(0))
            .expect("exists")
            .clone();

        let book_detail = Paragraph::new(vec![
            Line::from(Span::raw(selected_book.book_id)),
            Line::from(Span::raw(selected_book.title)),
            Line::from(Span::raw(selected_book.author)),
            Line::from(Span::raw(selected_book.pages)),
            Line::from(Span::raw(selected_book.volume)),
            Line::from(Span::raw(selected_book.edition)),
            Line::from(Span::raw(selected_book.year)),
            Line::from(Span::raw(selected_book.series)),
            Line::from(Span::raw(selected_book.publisher)),
            Line::from(Span::raw(selected_book.note)),
        ])
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Book Detail")
                .border_type(BorderType::Plain),
        );

        (list, header, book_detail)
    }

    /// UI for rendering the home section
    fn render_home() -> (Paragraph<'static>, Paragraph<'static>) {
        let left = Paragraph::new(vec![
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("Welcome to")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                "Library DB",
                Style::default().fg(Color::LightBlue),
            )]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                "To return to this Home page press 'H'",
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                "Press 'S' to Show a list of books",
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(vec![Span::styled(
                "Press 'B' to add a new Book",
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(vec![Span::styled(
                "Press 'L' to show a List of articles",
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(vec![Span::styled(
                "Press 'A' to add a new Article",
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(vec![Span::styled(
                "Press 'Q' to Quit",
                Style::default().fg(Color::Cyan),
            )]),
        ])
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Home")
                .border_type(BorderType::Plain),
        );

        let right = Paragraph::new(vec![
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("Editing Tips")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("'F2' to begin editing")]),
            Line::from(vec![Span::raw("'F9' to save to database")]),
            Line::from(vec![Span::raw("'F12' to exit editing without saving")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("'Ctrl-D' to Delete current item in list")]),
            Line::from(vec![Span::raw("'Ctrl-U' to Update current item in list")]),
        ])
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Editing Hot Keys")
                .border_type(BorderType::Plain),
        );
        (left, right)
    }

    /// UI for rendering the copyright section
    fn copyright() -> Paragraph<'static> {
        return Paragraph::new("Library DB 2023 - all rights reserved")
            .style(Style::default().fg(Color::Rgb(35, 70, 184)))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("Copyright")
                    .border_type(BorderType::Plain),
            );
    }

    /// Define terminal sections
    fn panes(rect: Rect) -> Rc<[Rect]> {
        return Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3), // main menu chunks[0]
                    Constraint::Min(2),    // content chunks[1]
                    Constraint::Length(3), // copyright chunks[2]
                ]
                .as_ref(),
            )
            .split(rect);
    }

    /// Define `home` sections
    fn home_panes(rect: Rc<[Rect]>) -> Rc<[Rect]> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(rect[1]);
        chunks
    }

    /// Define `show_` sections
    fn show_panes(rect: Rc<[Rect]>) -> Rc<[Rect]> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(20),
                    Constraint::Percentage(8),
                    Constraint::Percentage(80),
                ]
                .as_ref(),
            )
            .split(rect[1]);
        chunks
    }

    /// Define `add_` sections
    fn add_panes(rect: Rc<[Rect]>) -> Rc<[Rect]> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(rect[1]);
        chunks
    }

    /// UI for new_book
    // todo! combine block for book and article?
    fn new_book_block(x: bool) -> Block<'static> {
        if x == false {
            let new_book = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::LightCyan))
                .title("New Book:     Press 'F2' to enter edit mode and 'F9' to save     ")
                .border_type(BorderType::Plain);
            new_book
        } else {
            let edit_book = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::LightCyan))
                .title("Update Book:     Press 'F2' to enter edit mode and 'F9' to save     ")
                .border_type(BorderType::Plain);
            edit_book
        }
    }

    /// UI for new_article
    fn new_article_block(x: bool) -> Block<'static> {
        if x == false {
            let new_article = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::LightCyan))
                .title("New Article:     Press 'F2' to enter edit mode and 'F9' to save     ")
                .border_type(BorderType::Plain);
            new_article
        } else {
            let edit_article = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::LightCyan))
                .title("Update Article:     Press 'F2' to enter edit mode and 'F9' to save     ")
                .border_type(BorderType::Plain);
            edit_article
        }
    }

    /// UI for Menu bar
    // todo! change underlined letter on menu bar?
    fn menu<'a>(titles: Cloned<Iter<'a, &'static str>>, select: usize) -> Tabs<'a> {
        let menu = titles
            .map(|t| {
                let (first, rest) = t.split_at(1);
                Line::from(vec![
                    Span::styled(
                        first,
                        Style::default()
                            .fg(Color::Rgb(35, 70, 184))
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                    Span::styled(rest, Style::default().fg(Color::White)),
                ])
            })
            .collect();

        return Tabs::new(menu)
            .select(select)
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Rgb(35, 70, 184)))
            .divider(Span::raw("|"));
    }
}
