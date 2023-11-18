use std::sync::{Arc, mpsc, Mutex};
use std::{io, thread};
use std::time::{Duration, Instant};
use crate::db::{Book};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::{event};
// use crossterm::event::Event::Key;
use crossterm::terminal::{disable_raw_mode};
use ratatui::backend::{Backend};
use ratatui::{Terminal};
use ratatui::prelude::{Alignment, Color, Constraint, Direction, Layout, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, BorderType, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs};
use tui_textarea::{TextArea};
use crate::{DB_PATH};

// currently only adding new items. later add ability to search and edit items.

#[derive(Copy, Clone, PartialEq)]
enum InputMode {
    Normal,
    Editing,
}

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
    input: String,
    data: Vec<String>,
    cursor_position: usize,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
            App {
                menu_titles: vec!["Home", "Show Books", "Book Add", "List Articles", "Article Add", "Quit"],
                index: 0,
                active_menu_item: MenuItem::Home,
                book_list_state: Arc::new(Mutex::new(ListState::default())),
                input: String::new(),
                data: Vec::new(),
                cursor_position: 0,
            }
    }

    // Editor functions used from: https://github.com/ratatui-org/ratatui/blob/main/examples/user_input.rs
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    // fn move_cursor_down(&mut self) {
    //     let cursor_moved_down = self.cursor_position.
    // }


    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }
    fn submit_data(&mut self) {
        self.data.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
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
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
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
                        tx.send(AppEvent::Input(key)).expect("can send events");
                }

                if last_tick.elapsed() >= tick_rate {
                    if let Ok(_) = tx.send(AppEvent::Tick) {
                        last_tick = Instant::now();
                    }
                }
            }
        });

        // Create an empty `TextArea` instances which manages the editor state
        let mut text_area = App::render_add_book();


        loop {
            let terminal_size = terminal.size().expect("can size terminal");
            let menu_titles = self.menu_titles.iter().cloned();
            let active_menu_item = self.active_menu_item;
            let book_list_state = self.book_list_state.clone();
            let text_widget = text_area.widget();
            terminal.draw( move |frame|{
                let chunks = Layout::default()
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
                    .split(terminal_size);

                // Copyright section
                let copyright = Paragraph::new("Library DB 2023 - all rights reserved")
                    .style(Style::default().fg(Color::Rgb(35, 70, 184)))
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .style(Style::default().fg(Color::White))
                            .title("Copyright")
                            .border_type(BorderType::Plain),
                    );

                frame.render_widget(copyright, chunks[2]);

                // Main menu section
                // todo! change underlined letter on menu bar?
                let menu = menu_titles
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

                let tabs = Tabs::new(menu)
                    .select(active_menu_item as usize)
                    .block(Block::default().title("Menu").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().fg(Color::Rgb(35, 70, 184)))
                    .divider(Span::raw("|"));

                frame.render_widget(tabs, chunks[0]);

                // Change to a different menu item
                match active_menu_item {
                    MenuItem::Home => {frame.render_widget(App::render_home(), chunks[1])},
                    MenuItem::ShowBooks => {
                        book_list_state.lock().expect("can lock state").select(Some(0));
                        let book_chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints(
                                [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                            )
                            .split(chunks[1]);


                        let (left, right) = App::render_books(book_list_state.clone());
                        let mut lock = book_list_state.lock().expect("can lock state");
                        frame.render_stateful_widget(left, book_chunks[0],  &mut *lock);
                        drop(lock);
                        frame.render_widget(right, book_chunks[1]);

                    }
                    MenuItem::NewBook => {
                        //
                        // frame.render_widget(text_widget, chunks[1]);
                        //
                        // // todo! take input and save to db
                        // //     match crossterm::event::read().expect("can read input") {
                        // //         // Input { key: Key::Esc, .. } => break,
                        // //         input => {
                        // //             text_area.input(input);
                        // //         }
                        // //     }
                        //
                    }
                    MenuItem::ListArticles => {}
                    MenuItem::InsertArticle => {}
                }

            })?;

            match rx.recv().expect("can receive") {
                AppEvent::Input(event) => match event.code {
                    KeyCode::Char('q') if KeyModifiers::CONTROL == event.modifiers => {
                            disable_raw_mode().expect("can disable raw mode");
                            terminal.show_cursor().expect("can show cursor");
                            break;
                        },
                    KeyCode::Char('h') if KeyModifiers::ALT == event.modifiers => {self.active_menu_item = MenuItem::Home},
                    KeyCode::Char('s') if KeyModifiers::CONTROL == event.modifiers => {self.active_menu_item = MenuItem::ShowBooks},
                    KeyCode::Char('b') if KeyModifiers::CONTROL == event.modifiers => {self.active_menu_item = MenuItem::NewBook},
                    KeyCode::Char('l') if KeyModifiers::CONTROL == event.modifiers => {self.active_menu_item = MenuItem::ListArticles},
                    KeyCode::Char('a') if KeyModifiers::CONTROL == event.modifiers => {self.active_menu_item = MenuItem::InsertArticle},

                    KeyCode::Enter => self.submit_data(),
                    KeyCode::Char(to_insert) => {self.enter_char(to_insert)},
                    KeyCode::Backspace => {self.delete_char()},
                    KeyCode::Left => {self.move_cursor_left()},
                    KeyCode::Right => {self.move_cursor_right()},
                    KeyCode::Esc => {self.active_menu_item = MenuItem::Home},

                    // KeyCode::Down => {
                    //     // Need to hold current amount of books in shared memory and access that.
                    //     // if let Some(selected) = book_list_state.clone().selected() {
                    //     //     let amount_books = App::read_db().expect("can fetch book list").len();
                    //     //     if selected >= amount_books - 1 {
                    //     //         book_list_state.select(Some(0));
                    //     //     } else {
                    //     //         book_list_state.select(Some(selected + 1));
                    //     //     }
                    //     // }
                    // }
                    // KeyCode::Up => {
                    //     // if let Some(selected) = book_list_state.clone().selected() {
                    //     //     let amount_books = App::read_db().expect("can fetch book list").len();
                    //     //     if selected > 0 {
                    //     //         book_list_state.select(Some(selected - 1));
                    //     //     } else {
                    //     //         book_list_state.select(Some(amount_books - 1));
                    //     //     }
                    //     // }
                    // }
                    _ => {}
                },
                AppEvent::Tick => {}
            }
        }
        Ok(())

    }

    fn read_db() -> Result<Vec<Book>, io::Error> {
        let db_content = std::fs::read_to_string(&DB_PATH).unwrap();
        let parsed: Vec<Book> = serde_json::from_str(&db_content).unwrap();
        Ok(parsed)
    }

    fn render_add_book() -> TextArea<'static> {
        let new_book = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("New Book")
            .border_type(BorderType::Plain);

        let lines = vec![
            "{".to_string(),
            "    \"title\": \"\",".to_string(),
            "    \"author\": \"\",".to_string(),
            "    \"pages\": \"\",".to_string(),
            "    \"volume\": \"\",".to_string(),
            "    \"edition\": \"\",".to_string(),
            "    \"series\": \"\",".to_string(),
            "    \"note\": \"\"".to_string(),
            "},".to_string()
            ];

        let mut textarea = TextArea::new(lines);
        textarea.set_block(new_book);
        // textarea = io::BufWriter::new()
        textarea
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

        let lock = book_list_state.lock().expect("can lock state");
        let selected_book = book_list
            .get(
                lock
                    .selected()
                    .expect("there is always a selected book"),
            )
            .expect("exists")
            .clone();
        drop(lock);

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
            Line::from(vec![Span::raw("To return to this Home page pres 'Alt-M'")]),
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
}

//
// macro_rules! error {
//     ($fmt: expr $(, $args:tt)*) => {{
//         Err(io::Error::new(io::ErrorKind::Other, format!($fmt $(, $args)*)))
//     }};
// }
//
// struct Buffer<'a> {
//     textarea: TextArea<'a>,
//     path: PathBuf,
//     modified: bool,
// }
//
// impl<'a> Buffer<'a> {
//     fn new() -> io::Result<Self> {
//         // let mut stream = BufWriter::new(stdout()); // todo! maybe change to terminal?
//         let mut textarea = if let Ok(md) = path.metadata() {
//             if md.is_file() {
//                 let mut textarea = io::BufReader::new(fs::File::open(&path)?)
//                     .lines()
//                     .collect::<io::Result<_>>()?;
//                 if textarea.lines().iter().any(|l| l.starts_with('\t')) {
//                     textarea.set_hard_tab_indent(true);
//                 }
//                 textarea
//             } else {
//                 return error!("{:?} is not a file", path);
//             }
//         } else {
//             TextArea::default()
//         };
//         textarea.set_line_number_style(Style::default().fg(Color::Blue));
//         Ok(Self {
//             textarea,
//             path,
//             modified: false,
//         })
//     }
//
//     fn save(&mut self) -> io::Result<()> {
//         if !self.modified {
//             return Ok(());
//         }
//         let mut f = io::BufWriter::new(fs::File::create(&self.path)?);
//         for line in self.textarea.lines() {
//             f.write_all(line.as_bytes())?;
//             f.write_all(b"\n")?;
//         }
//         self.modified = false;
//         Ok(())
//     }
//
// }