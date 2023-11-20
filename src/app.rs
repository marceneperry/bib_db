use std::sync::{Arc, mpsc, Mutex};
use std::{io, thread};
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
use tui_textarea::{TextArea};
use crate::{DB_PATH};

// currently only adding new items. later add ability to search and edit items.

struct TextState {

}

#[derive(Copy, Clone, PartialEq)]
enum InputMode {
    Normal,
    Editing,
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
    NewBook,
    ListArticles,
    InsertArticle,
}


pub struct App<'a> {
    pub menu_titles: Vec<&'a str>,
    pub index: usize,
    active_menu_item: MenuItem,
    pub book_list_state: Arc<Mutex<ListState>>,
    // input: String,
    // data: Vec<String>,
    // cursor_position: usize,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            menu_titles: vec!["Home", "Show Books", "Book Add", "List Articles", "Article Add", "Quit"],
            index: 0,
            active_menu_item: MenuItem::Home,
            book_list_state: Arc::new(Mutex::new(ListState::default())),
            // input: String::new(),
            // data: Vec::new(),
            // cursor_position: 0,
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
            // let mut counter = 0;
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

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
            .style(Style::default().fg(Color::White))
            .title("New Book")
            .border_type(BorderType::Plain);
        let mut text_area = TextArea::default();
        text_area.set_block(new_book);


        loop {
            let terminal_size = terminal.size().expect("can size terminal");
            let menu_titles = self.menu_titles.iter().cloned();
            let active_menu_item = self.active_menu_item;
            let book_list_state = self.book_list_state.clone();
            let text_widget = text_area.widget();

            terminal.draw( move |frame|{
                let chunks = App::panes(terminal_size);

                // Main menu section
                frame.render_widget(App::menu(menu_titles, active_menu_item as usize), chunks[0]);

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
                    MenuItem::NewBook => {
                        let book_panes = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints(
                                [Constraint::Percentage(10), Constraint::Percentage(90)].as_ref(),
                            )
                            .split(chunks[1]);


                        frame.render_widget(App::render_add_book(), book_panes[0]);
                        frame.render_widget(text_widget, book_panes[1]);
                    }
                    MenuItem::ListArticles => {}
                    MenuItem::InsertArticle => {}
                }

                // Copyright section
                frame.render_widget(App::copyright(), chunks[2]);
            })?;

            match rx.recv().unwrap() {
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('q'), modifiers,  ..})) if KeyModifiers::CONTROL == modifiers => {
                            disable_raw_mode().expect("can disable raw mode");
                            terminal.show_cursor().expect("can show cursor");
                            break;
                },
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('h'), modifiers,  ..})) if KeyModifiers::ALT == modifiers => {self.active_menu_item = MenuItem::Home}
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('s'), modifiers,  ..})) if KeyModifiers::CONTROL == modifiers => {self.active_menu_item = MenuItem::ShowBooks},
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('b'), modifiers,  ..})) if KeyModifiers::CONTROL == modifiers => {self.active_menu_item = MenuItem::NewBook},
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('l'), modifiers,  ..})) if KeyModifiers::CONTROL == modifiers => {self.active_menu_item = MenuItem::ListArticles},
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('a'), modifiers,  ..})) if KeyModifiers::CONTROL == modifiers => {self.active_menu_item = MenuItem::InsertArticle},
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('s'), modifiers,  ..})) if KeyModifiers::ALT == modifiers => {println!("Lines: {:?}", text_area.lines())},
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Down,  ..})) => {
                    let mut lock = self.book_list_state.lock().expect("can lock state");
                    if let Some(selected) = lock.selected() {
                        let amount_books = App::read_db().expect("can fetch book list").len();
                        if selected >= amount_books - 1 {
                            lock.select(Some(0));
                        } else {
                            lock.select(Some(selected + 1));
                        }
                    }
                    drop(lock);
                },
                AppEvent::Input(Event::Key(KeyEvent { code: KeyCode::Up, ..})) => {
                    let mut lock = self.book_list_state.lock().expect("can lock state");
                    if let Some(selected) = lock.selected() {
                        let amount_books = App::read_db().expect("can fetch book list").len();
                        if selected > 0 {
                            lock.select(Some(selected - 1));
                        } else {
                            lock.select(Some(amount_books - 1));
                        }
                    }
                },
                AppEvent::Tick => {},
                AppEvent::Input(input) => { text_area.input(input); },
            };
        }
        Ok(())

    }

    fn read_db() -> Result<Vec<Book>, io::Error> {
        let db_content = std::fs::read_to_string(&DB_PATH).unwrap();
        let parsed: Vec<Book> = serde_json::from_str(&db_content).unwrap();
        Ok(parsed)
    }

    fn render_add_book() -> Paragraph<'a> {
        let index = Paragraph::new(vec![
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled("Title: ", Style::default().fg(Color::LightBlue))]),
            Line::from(vec![Span::styled("Author: ", Style::default().fg(Color::LightBlue))]),
            Line::from(vec![Span::styled("Pages: ", Style::default().fg(Color::LightBlue))]),
            Line::from(vec![Span::styled("Volume: ", Style::default().fg(Color::LightBlue))]),
            Line::from(vec![Span::styled("Edition: ", Style::default().fg(Color::LightBlue))]),
            Line::from(vec![Span::styled("Series: ", Style::default().fg(Color::LightBlue))]),
            Line::from(vec![Span::styled("Note: ", Style::default().fg(Color::LightBlue))]),
        ])
            .alignment(Alignment::Right);

        index
    }

    fn render_books(book_list_state: Arc<Mutex<ListState>>) -> (List<'a>, Table<'a>) {
        let books = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Books")
            .border_type(BorderType::Plain);

        let book_list = App::read_db().expect("can fetch book list");
        let items: Vec<_> = App::read_db().expect("can fetch book list")
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
                "Title",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Author",
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
            Constraint::Percentage(10),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(10),
            Constraint::Percentage(20),
        ]);

        (list, book_detail)
    }

    fn render_home() -> Paragraph<'a> {
        let home = Paragraph::new(vec![
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
            Line::from(vec![Span::raw("To return to this Home page pres 'Alt-H'")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("Press 'Ctrl-S' to Show a list of books")]),
            Line::from(vec![Span::raw("Press 'Ctrl-B' to add a new Book")]),
            Line::from(vec![Span::raw("Press 'Ctrl-L' to show a List of articles")]),
            Line::from(vec![Span::raw("Press 'Ctrl-A' to add a new Article")]),
            Line::from(vec![Span::raw("Press 'Ctrl-Q' to Quit")]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Home")
                .border_type(BorderType::Plain),
        );
        home
    }

    fn render_book_page() -> Paragraph<'a> {
        let home = Paragraph::new(vec![
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                "Books Home",
                Style::default().fg(Color::LightBlue),
            )]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("Press 's' to show a list of Books")]),
            Line::from(vec![Span::raw("Press 'n' to add a new book.")]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Book options")
                .border_type(BorderType::Plain),
        );
        home
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
