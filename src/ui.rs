use crate::db::{read_sqlite_article_table, read_sqlite_book_table};
use ratatui::layout::Rect;
use ratatui::prelude::{Alignment, Color, Constraint, Direction, Layout, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Tabs};
use std::iter::Cloned;
use std::rc::Rc;
use std::slice::Iter;
use std::sync::{Arc, Mutex};

/// UI for the tui app

/// UI for Menu bar
// todo! change underlined letter on menu bar?
pub fn menu<'a>(titles: Cloned<Iter<'a, &'static str>>, select: usize) -> Tabs<'a> {
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

/// UI for `new_book`
// todo! combine block for book and article?
pub fn new_book_block(x: bool) -> Block<'static> {
    if !x {
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

/// UI for `new_article`
pub fn new_article_block(x: bool) -> Block<'static> {
    if !x {
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

/// UI for rendering the `copyright` section
pub fn copyright() -> Paragraph<'static> {
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
pub fn panes(rect: Rect) -> Rc<[Rect]> {
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
pub fn home_panes(rect: Rc<[Rect]>) -> Rc<[Rect]> {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(rect[1]);
    chunks
}

/// Define `show_` sections
pub fn show_panes(rect: Rc<[Rect]>) -> Rc<[Rect]> {
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
pub fn add_panes(rect: Rc<[Rect]>) -> Rc<[Rect]> {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(rect[1]);
    chunks
}

/// UI for rendering all books in the database
pub fn render_books(
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

    let items: Vec<_> = read_sqlite_book_table()
        .expect("should fetch book list")
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

    let book_list = read_sqlite_book_table().expect("should fetch book list");
    if book_list.is_empty() {
        let book_detail = Paragraph::default();
        return (list, header, book_detail);
    }

    let selected = book_list_state
        .lock()
        .expect("should lock state")
        .selected();
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

/// UI for rendering all articles in the database
pub fn render_articles(
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

    let items: Vec<_> = read_sqlite_article_table()
        .expect("should fetch article list")
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

    let article_list = read_sqlite_article_table().expect("should fetch article list");
    if article_list.is_empty() {
        let article_detail = Paragraph::default();
        return (list, header, article_detail);
    }

    let selected = article_list_state
        .lock()
        .expect("should lock article state")
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

/// UI for rendering the `home` section
pub fn render_home() -> (Paragraph<'static>, Paragraph<'static>) {
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

/// UI for adding a new `article`
pub fn render_add_article() -> Paragraph<'static> {
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

/// UI for adding a new `book`
pub fn render_add_book() -> Paragraph<'static> {
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
