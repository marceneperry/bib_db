use std::sync::{Arc, mpsc, Mutex};
use std::{thread};
use std::io::Error;
use std::iter::Cloned;
use std::rc::Rc;
use std::slice::Iter;
use std::time::{Duration, Instant};
use crate::db::{Book};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::{event};
use crossterm::terminal::{disable_raw_mode};
use ratatui::backend::{Backend};
use ratatui::{Terminal};
use ratatui::layout::Rect;
use ratatui::prelude::{Alignment, Color, Constraint, Direction, Layout, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, BorderType, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs};
use sqlite::State;
use tui_textarea::{TextArea};
use crate::{DB_PATH, DB_URL};

// currently only adding new items. later add ability to search and edit items.


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

pub struct App<'a> {
    pub menu_titles: Vec<&'a str>,
    pub index: usize,
    active_menu_item: MenuItem,
    pub book_list_state: Arc<Mutex<ListState>>,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            menu_titles: vec!["Home", "Show Books", "Book Add", "List Articles", "Article Add", "Quit"],
            index: 0,
            active_menu_item: MenuItem::Home,
            book_list_state: Arc::new(Mutex::new(ListState::default())),
        }
    }

    // Iterating menu functions
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.menu_titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.menu_titles.len() - 1;
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
                            tx.send(AppEvent::Input(Event::Key(key))).expect("can send events");
                        }
                }

                if last_tick.elapsed() >= tick_rate {
                    if let Ok(_) = tx.send(AppEvent::Tick) {
                        last_tick = Instant::now();
                    }
                }
            }
        });

        // Create an empty `TextArea` instance which manages the editor state
        let new_book = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightCyan))
            .title("New Book:     Press 'Alt-i' to enter edit mode and 'Alt-x' to exit edit mode     ")
            .border_type(BorderType::Plain);
        let mut book_text_area = TextArea::default();
        book_text_area.set_block(new_book.clone());


        loop {
            let terminal_size = terminal.size().expect("can size terminal");
            let menu_titles = self.menu_titles.iter().cloned();
            let active_menu_item = self.active_menu_item;
            let book_list_state = self.book_list_state.clone();
            let book_text_widget = book_text_area.widget();

            terminal.draw( move |frame|{
                let chunks = App::panes(terminal_size);

                // Main menu section
                frame.render_widget(App::menu(menu_titles, active_menu_item.ordinal()), chunks[0]);

                // Change to a different menu item
                match active_menu_item {
                    MenuItem::Home => {frame.render_widget(App::render_home(), chunks[1])},
                    MenuItem::ShowBooks => {
                        let mut lock = book_list_state.lock().expect("can lock state");
                        if lock.selected().is_none() {
                            lock.select(Some(0));
                        }
                        drop(lock);

                        let book_chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints(
                                [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                            )
                            .split(chunks[1]);


                        let (left, right) = App::render_books(book_list_state.clone());
                        let mut lock = book_list_state.lock().expect("can lock state");
                        frame.render_stateful_widget(left, book_chunks[0],  &mut *lock);
                        frame.render_widget(right, book_chunks[1]);
                        drop(lock);
                    }
                    MenuItem::NewBook(..) => {
                        let book_panes = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints(
                                [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                            )
                            .split(chunks[1]);


                        frame.render_widget(App::render_add_book(), book_panes[0]);
                        frame.render_widget(book_text_widget, book_panes[1]);
                    }
                    MenuItem::ListArticles => {}
                    MenuItem::InsertArticle(..) => {}
                }

                // Copyright section
                frame.render_widget(App::copyright(), chunks[2]);
            })?;

            match rx.recv().unwrap() {
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('q'),  ..})) if self.is_command_mode() => {
                            disable_raw_mode().expect("can disable raw mode");
                            terminal.show_cursor().expect("can show cursor");
                            break;
                },
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('h'), ..})) if self.is_command_mode() => {self.active_menu_item = MenuItem::Home}
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('s'), ..})) if self.is_command_mode() => {self.active_menu_item = MenuItem::ShowBooks},
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('b'), ..})) if self.is_command_mode() => {self.active_menu_item = MenuItem::NewBook(InputMode::Command)},
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('l'), ..})) if self.is_command_mode() => {self.active_menu_item = MenuItem::ListArticles},
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('a'), ..})) if self.is_command_mode() => {self.active_menu_item = MenuItem::InsertArticle(InputMode::Command)},
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('i'), modifiers, ..})) if KeyModifiers::ALT == modifiers && self.is_command_mode() => {
                    self.enter_input_mode()
                },
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('x'), modifiers, ..})) if KeyModifiers::ALT == modifiers && !self.is_command_mode() => {
                    self.exit_input_mode()
                },

                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('p'), modifiers,  ..})) if KeyModifiers::CONTROL == modifiers => {
                    self.save_as_item_type(&book_text_area);
                    book_text_area = TextArea::default();
                    book_text_area.set_block(new_book.clone());
                },
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Down,  ..})) if self.is_command_mode() => {
                    let mut lock = self.book_list_state.lock().expect("can lock state");
                    if let Some(selected) = lock.selected() {
                        let amount_books = App::read_sqlite_db().expect("can fetch book list").len();
                        if selected >= amount_books - 1 {
                            lock.select(Some(0));
                        } else {
                            lock.select(Some(selected + 1));
                        }
                    }
                    drop(lock);
                },
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Up, ..})) if self.is_command_mode() => {
                    let mut lock = self.book_list_state.lock().expect("can lock state");
                    if let Some(selected) = lock.selected() {
                        let amount_books = App::read_sqlite_db().expect("can fetch book list").len();
                        if selected > 0 {
                            lock.select(Some(selected - 1));
                        } else {
                            lock.select(Some(amount_books - 1));
                        }
                    }
                },
                AppEvent::Tick => {},
                AppEvent::Input(input) if !self.is_command_mode() => { book_text_area.input(input); },
                _ => {}
            };
        }
        Ok(())

    }

    fn save_as_item_type(&mut self, text_area: &TextArea) {
        if let MenuItem::NewBook(_) = self.active_menu_item {
            // println!("{:?}", text_area.lines());
            let mut text_vec = Vec::new();
            for line in text_area.lines() {
                text_vec.push(line.to_string());
            }
            Book::book_transaction(text_vec);

        } else if let MenuItem::InsertArticle(_) = self.active_menu_item {
            println!("{:?}", text_area.lines());
        }
    }
    fn enter_input_mode(&mut self) {
        if let MenuItem::NewBook(InputMode::Command) = self.active_menu_item {
            self.active_menu_item = MenuItem::NewBook(InputMode::Input)
        } else if let MenuItem::InsertArticle(InputMode::Command) = self.active_menu_item {
            self.active_menu_item = MenuItem::InsertArticle(InputMode::Input)
        }
    }

    fn exit_input_mode(&mut self) {
        if let MenuItem::NewBook(InputMode::Input) = self.active_menu_item {
            self.active_menu_item = MenuItem::NewBook(InputMode::Command)
        } else if let MenuItem::InsertArticle(InputMode::Input) = self.active_menu_item {
            self.active_menu_item = MenuItem::InsertArticle(InputMode::Command)
        }
    }

    fn is_command_mode(&self) -> bool {
        match self.active_menu_item {
            MenuItem::NewBook(InputMode::Input) | MenuItem::InsertArticle(InputMode::Input) => false,
            _ => true
        }
    }

    fn read_sqlite_db() -> Result<Vec<Book>, Error> {
        let connection = sqlite::open(DB_URL).unwrap();
        let query = "SELECT book_id, cite_key, publisher_id, month_year_id, author, title, pages, volume, edition, series, note FROM book";
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
                    series: statement.read::<String, _>("series").unwrap(),
                    note: statement.read::<String, _>("note").unwrap(),
                })
        }
        Ok(parsed)
    }


    fn read_db() -> Result<Vec<Book>, Error> {
        let db_content = std::fs::read_to_string(&DB_PATH).unwrap();
        let parsed: Vec<Book> = serde_json::from_str(&db_content).unwrap();
        Ok(parsed)
    }

    fn render_add_book() -> Paragraph<'a> {
        return Paragraph::new(vec![
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled("Title: ", Style::default().fg(Color::LightRed))]),
            Line::from(vec![Span::styled("Author: ", Style::default().fg(Color::LightRed))]),
            Line::from(vec![Span::styled("Pages: ", Style::default().fg(Color::LightRed))]),
            Line::from(vec![Span::styled("Volume: ", Style::default().fg(Color::LightBlue))]),
            Line::from(vec![Span::styled("Edition: ", Style::default().fg(Color::LightBlue))]),
            Line::from(vec![Span::styled("Series: ", Style::default().fg(Color::LightBlue))]),
            Line::from(vec![Span::styled("Note: ", Style::default().fg(Color::LightBlue))]),
            Line::from(vec![Span::styled("Year: ", Style::default().fg(Color::LightRed))]),
            Line::from(vec![Span::styled("Publisher: ", Style::default().fg(Color::LightRed))]),
            Line::from(vec![Span::styled("", Style::default())]),
            Line::from(vec![Span::styled("", Style::default())]),
            Line::from(vec![Span::styled("", Style::default())]),
            Line::from(vec![Span::styled("Required input is red ", Style::default().fg(Color::LightRed))]),
            Line::from(vec![Span::styled("Optional input is blue ", Style::default().fg(Color::LightBlue))]),
            Line::from(vec![Span::styled("", Style::default())]),
            Line::from(vec![Span::styled("", Style::default())]),
            Line::from(vec![Span::styled("", Style::default())]),
            Line::from(vec![Span::styled("Press Alt-I to start editing ", Style::default().fg(Color::Cyan))]),
            Line::from(vec![Span::styled("Press Alt-X to stop editing ", Style::default().fg(Color::Cyan))]),
            Line::from(vec![Span::styled("Press Ctrl-P to save to database ", Style::default().fg(Color::Cyan))]),
        ])
            .alignment(Alignment::Right);
    }

    fn render_books(book_list_state: Arc<Mutex<ListState>>) -> (List<'a>, Table<'a>) {
        // todo! currently using dummy db.json file to render books ---> implement sqlite db rendering;
        let books = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Books")
            .border_type(BorderType::Plain);

        let book_list = App::read_sqlite_db().expect("can fetch book list");
        let items: Vec<_> = App::read_sqlite_db().expect("can fetch book list")
            .iter()
            .map(|book| {
                ListItem::new(Line::from(vec![Span::styled(
                    book.title.clone(),
                    Style::default(),
                )]))
            })
            .collect();

        let selected = book_list_state.lock().expect("can lock state").selected();
        let selected_book = book_list
            .get(selected.unwrap_or(0))
            .expect("exists")
            .clone();

        let list = List::new(items).block(books).highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

        let book_detail = Table::new(vec![Row::new(vec![
            Cell::from(Span::raw(selected_book.book_id)),
            Cell::from(Span::raw(selected_book.title)),
            Cell::from(Span::raw(selected_book.author)),
            Cell::from(Span::raw(selected_book.pages)),
            Cell::from(Span::raw(selected_book.volume)),
            Cell::from(Span::raw(selected_book.edition)),
            Cell::from(Span::raw(selected_book.series)),
            Cell::from(Span::raw(selected_book.note)),
        ])])
        .header(Row::new(vec![
            Cell::from(Span::styled(
                "ID",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Author",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Title",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Pages",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Volume",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Edition",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Series",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Note",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Book Detail")
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(5),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ]);

        (list, book_detail)
    }

    fn render_home() -> Paragraph<'a> {
        return Paragraph::new(vec![
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("Welcome")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("to")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                "Library DB",
                Style::default().fg(Color::LightBlue),
            )]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("To return to this Home page press 'H'")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("Press 'S' to Show a list of books")]),
            Line::from(vec![Span::raw("Press 'B' to add a new Book")]),
            Line::from(vec![Span::raw("Press 'L' to show a List of articles")]),
            Line::from(vec![Span::raw("Press 'A' to add a new Article")]),
            Line::from(vec![Span::raw("Press 'Q' to Quit")]),
        ])
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Home")
                .border_type(BorderType::Plain),
        );
    }

    fn panes(rect: Rect) -> Rc<[Rect]> {
        return Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints(
                        [
                            Constraint::Length(3), // main menu chunks[0]
                            Constraint::Min(2), // content chunks[1]
                            Constraint::Length(3), // copyright chunks[2]
                        ]
                            .as_ref(),
                    )
                    .split(rect);
    }

    fn copyright() -> Paragraph<'a> {
       return  Paragraph::new("Library DB 2023 - all rights reserved")
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

    fn menu(titles: Cloned<Iter<'_, &'a str>>, select: usize) -> Tabs<'a> {
        // todo! change underlined letter on menu bar?
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
