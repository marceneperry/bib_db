mod ui {
    pub fn new_book() -> Block {
       // Create an empty `TextArea` instance for a book which manages the editor state
        let new_book = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightCyan))
            .title(
                "New Book:     Press 'Alt-i' to enter edit mode and 'Alt-x' to exit edit mode     ",
            )
            .border_type(BorderType::Plain);
    }
}
