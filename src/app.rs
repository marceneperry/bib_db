use crate::db::{read_sqlite_article_table, read_sqlite_book_table, Article, Book, RowSelect};
use crate::ui::*;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::disable_raw_mode;
use ratatui::backend::Backend;
use ratatui::widgets::ListState;
use ratatui::Terminal;
use std::io::Error;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tui_textarea::TextArea;

/// The app module structs and functions
// todo! Future implementation show ALL items sorted by cite_key?
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
    // todo! Future implementation: do sub menus:
    // `Home` `Display All Items` `Books` `Articles` `Quit`
    // `Books`: `Display All` `Add New Book` `Delete Book` `Find Book`
    // `Articles`: `Display All` `Add New Article` `Delete Article` `Find Article`
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
            update_item_id: "".to_string(),
            update_flag: false,
        }
    }

    /// Run the terminal loop with event handlers
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Error> {
        // setup mpsc to handle the channels in the rendering loop
        let (tx, rx) = mpsc::channel();
        let tick_rate = Duration::from_millis(200);
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                if let Event::Key(key) = event::read().expect("should read events") {
                    if key.kind == KeyEventKind::Press {
                        tx.send(AppEvent::Input(Event::Key(key)))
                            .expect("should send events");
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
            let terminal_size = terminal.size().expect("should size terminal");
            let menu_titles = self.menu_titles.iter().cloned();
            let active_menu_item = self.active_menu_item;
            let book_list_state = self.book_list_state.clone();
            let article_list_state = self.article_list_state.clone();
            book_text_area.set_block(new_book_block(self.update_flag));
            let book_text_widget = book_text_area.widget();
            article_text_area.set_block(new_article_block(self.update_flag));
            let article_text_widget = article_text_area.widget();

            // todo! Future implementation: Move the terminal to a tui.rs?
            terminal.draw(move |frame| {
                let chunks = panes(terminal_size);

                // Main menu section
                frame.render_widget(menu(menu_titles, active_menu_item.ordinal()), chunks[0]);

                // Change to a different menu item
                match active_menu_item {
                    MenuItem::Home => {
                        let (left, right) = render_home();
                        frame.render_widget(left, home_panes(chunks.clone())[0]);
                        frame.render_widget(right, home_panes(chunks.clone())[1]);
                    }
                    MenuItem::ShowBooks => {
                        let mut lock = book_list_state.lock().expect("should lock state");
                        if lock.selected().is_none() {
                            lock.select(Some(0));
                        }
                        drop(lock);

                        let (left, middle, right) = render_books(book_list_state.clone());
                        let mut lock = book_list_state.lock().expect("should lock state");
                        frame.render_stateful_widget(
                            left,
                            show_panes(chunks.clone())[0],
                            &mut *lock,
                        );
                        frame.render_widget(middle, show_panes(chunks.clone())[1]);
                        frame.render_widget(right, show_panes(chunks.clone())[2]);
                        drop(lock);
                    }
                    MenuItem::NewBook(..) => {
                        frame.render_widget(render_add_book(), add_panes(chunks.clone())[0]);
                        frame.render_widget(book_text_widget, add_panes(chunks.clone())[1]);
                    }
                    MenuItem::ListArticles => {
                        let mut lock = article_list_state.lock().expect("should lock state");
                        if lock.selected().is_none() {
                            lock.select(Some(0));
                        }
                        drop(lock);

                        let (left, middle, right) = render_articles(article_list_state.clone());
                        let mut lock = article_list_state.lock().expect("should lock state");
                        frame.render_stateful_widget(
                            left,
                            show_panes(chunks.clone())[0],
                            &mut *lock,
                        );
                        frame.render_widget(middle, show_panes(chunks.clone())[1]);
                        frame.render_widget(right, show_panes(chunks.clone())[2]);
                        drop(lock);
                    }
                    MenuItem::InsertArticle(..) => {
                        frame.render_widget(render_add_article(), add_panes(chunks.clone())[0]);
                        frame.render_widget(article_text_widget, add_panes(chunks.clone())[1]);
                    }
                }

                // Copyright section
                frame.render_widget(copyright(), chunks[2]);
            })?;

            // Match key events to move around the app menu and edit in text areas
            match rx.recv().unwrap() {
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Char('q'), // Quit
                    ..
                })) if self.is_command_mode() => {
                    disable_raw_mode().expect("should disable raw mode");
                    terminal.show_cursor().expect("should show cursor");
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
                    self.active_menu_item = MenuItem::NewBook(InputMode::Command);
                }
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Char('l'), // Show a list of articles
                    ..
                })) if self.is_command_mode() => self.active_menu_item = MenuItem::ListArticles,
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Char('a'), // Add a new article
                    ..
                })) if self.is_command_mode() => {
                    self.active_menu_item = MenuItem::InsertArticle(InputMode::Command);
                }
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::F(2), // Enter edit mode
                    ..
                })) => {
                    self.enter_input_mode();
                }
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::F(12), // Exit edit mode
                    ..
                })) => {
                    self.update_flag = false;
                    self.exit_input_mode();
                }
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::F(9), // Save to database
                    ..
                })) => {
                    if let MenuItem::NewBook(_) = self.active_menu_item {
                        self.save_as_item_type(&book_text_area);
                        book_text_area = TextArea::default();
                        book_text_area.set_block(new_book_block(false));
                    } else if let MenuItem::InsertArticle(_) = self.active_menu_item {
                        self.save_as_item_type(&article_text_area);
                        article_text_area = TextArea::default();
                        article_text_area.set_block(new_article_block(false));
                    }
                    self.update_flag = false;
                    self.exit_input_mode();
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
                        book_text_area.set_block(new_book_block(self.update_flag));
                        self.active_menu_item = MenuItem::NewBook(InputMode::Input);
                    } else if let MenuItem::ListArticles = self.active_menu_item {
                        self.update_flag = true;
                        self.get_item_id();
                        let text_vec = Article::select(&self.update_item_id);
                        article_text_area = TextArea::new(text_vec);
                        article_text_area.set_block(new_article_block(self.update_flag));
                        self.active_menu_item = MenuItem::InsertArticle(InputMode::Input);
                    }
                }
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Char('d'), // Delete selected item
                    modifiers,
                    ..
                })) if KeyModifiers::CONTROL == modifiers => {
                    if let MenuItem::ShowBooks = self.active_menu_item {
                        let book_list = read_sqlite_book_table().expect("should fetch book list");
                        if book_list.is_empty() {
                        } else {
                            self.get_item_id();
                            Book::delete_book(self.update_item_id.clone());

                            // if last item in list move selected item back to top of list
                            let mut lock = self.book_list_state.lock().expect("should lock state");
                            if let Some(selected) = lock.selected() {
                                let amount_books = read_sqlite_book_table()
                                    .expect("should fetch book list")
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
                        let article_list = read_sqlite_article_table().expect("should fetch book list");
                        if article_list.is_empty() {
                        } else {
                            self.get_item_id();
                            Article::delete_article(self.update_item_id.clone());

                            // if last item in list move selected item back to top of list
                            let mut lock = self.article_list_state.lock().expect("should lock state");
                            if let Some(selected) = lock.selected() {
                                let amount_books = read_sqlite_article_table()
                                    .expect("should fetch book list")
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
                        let mut lock = self.book_list_state.lock().expect("should lock state");
                        if let Some(selected) = lock.selected() {
                            let amount_books = read_sqlite_book_table()
                                .expect("should fetch book list")
                                .len();
                            if selected >= amount_books - 1 {
                                lock.select(Some(0));
                            } else {
                                lock.select(Some(selected + 1));
                            }
                        }
                        drop(lock);
                    } else if let MenuItem::ListArticles = self.active_menu_item {
                        let mut lock = self.article_list_state.lock().expect("should lock state");
                        if let Some(selected) = lock.selected() {
                            let amount_articles = read_sqlite_article_table()
                                .expect("should fetch book list")
                                .len();
                            if selected >= amount_articles - 1 {
                                lock.select(Some(0));
                            } else {
                                lock.select(Some(selected + 1));
                            }
                        }
                        drop(lock);
                    }
                }
                AppEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Up, .. // Move up in the list of books or articles; wraps around
                })) if self.is_command_mode() => {
                    if let MenuItem::ShowBooks = self.active_menu_item {
                        let mut lock = self.book_list_state.lock().expect("should lock state");
                        if let Some(selected) = lock.selected() {
                            let amount_books = read_sqlite_book_table()
                                .expect("should fetch book list")
                                .len();
                            if selected > 0 {
                                lock.select(Some(selected - 1));
                            } else {
                                lock.select(Some(amount_books - 1));
                            }
                        }
                        drop(lock);
                    } else if let MenuItem::ListArticles = self.active_menu_item {
                        let mut lock = self.article_list_state.lock().expect("should lock state");
                        if let Some(selected) = lock.selected() {
                            let amount_articles = read_sqlite_article_table()
                                .expect("should fetch book list")
                                .len();
                            if selected > 0 {
                                lock.select(Some(selected - 1));
                            } else {
                                lock.select(Some(amount_articles - 1));
                            }
                        }
                        drop(lock);
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
                AppEvent::Input(_) => {}
            };
        }
        Ok(())
    }

    /// Retrieves `cite_key` of current item (`book` or `article`); Used to update or delete an item.
    fn get_item_id(&mut self) {
        self.update_item_id = String::new();
        if let MenuItem::ShowBooks = self.active_menu_item {
            let book_list = read_sqlite_book_table().expect("should fetch book list");
            let selected = self
                .book_list_state
                .lock()
                .expect("should lock list state")
                .selected();
            let selected_item = book_list
                .get(selected.unwrap_or(0))
                .expect("exists")
                .clone();
            self.update_item_id = selected_item.cite_key.clone();
        } else if let MenuItem::ListArticles = self.active_menu_item {
            let article_list = read_sqlite_article_table().expect("should fetch book list");
            let selected = self
                .article_list_state
                .lock()
                .expect("should lock list state")
                .selected();
            let selected_item = article_list
                .get(selected.unwrap_or(0))
                .expect("exists")
                .clone();
            self.update_item_id = selected_item.cite_key;
        }
    }

    /// Save the data entered in the textarea to Book or Article table
    fn save_as_item_type(&mut self, text_area: &TextArea) {
        let mut text_vec = Vec::new();
        for line in text_area.lines() {
            text_vec.push(line.to_string());
        }
        if let MenuItem::NewBook(_) = self.active_menu_item {
            if !self.update_flag {
                Book::book_transaction(text_vec);
            } else {
                Book::book_update(text_vec, self.update_item_id.clone());
            }
        } else if let MenuItem::InsertArticle(_) = self.active_menu_item {
            if !self.update_flag {
                Article::article_transaction(text_vec);
            } else {
                Article::article_update(text_vec, self.update_item_id.clone());
            }
        }
    }

    /// Change the state of the app from Command mode to Input mode
    fn enter_input_mode(&mut self) {
        if let MenuItem::NewBook(InputMode::Command) = self.active_menu_item {
            self.active_menu_item = MenuItem::NewBook(InputMode::Input);
        }
        if let MenuItem::InsertArticle(InputMode::Command) = self.active_menu_item {
            self.active_menu_item = MenuItem::InsertArticle(InputMode::Input);
        }
    }

    /// Change the state of the app from Input mode to Command mode
    fn exit_input_mode(&mut self) {
        if let MenuItem::NewBook(InputMode::Input) = self.active_menu_item {
            self.active_menu_item = MenuItem::NewBook(InputMode::Command);
        } else if let MenuItem::InsertArticle(InputMode::Input) = self.active_menu_item {
            self.active_menu_item = MenuItem::InsertArticle(InputMode::Command);
        }
    }

    /// Check if the app is in command mode or input mode
    fn is_command_mode(&self) -> bool {
        !matches!(
            self.active_menu_item,
            MenuItem::NewBook(InputMode::Input) | MenuItem::InsertArticle(InputMode::Input)
        ) // cool clippy suggestion!
    }
}
