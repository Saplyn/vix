use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    widgets::Widget,
};

#[derive(Debug, Default)]
pub struct Document {
    rows: Vec<DocLine>,
}

#[derive(Debug)]
pub struct DocLine {
    content: String,
}

impl DocLine {
    pub fn from_str(ln: &str) -> Self {
        Self {
            content: String::from(ln),
        }
    }
}

impl Document {
    pub fn hello_world() -> Self {
        let rows = vec![
            DocLine::from_str("Hello World!"),
            DocLine::from_str("Hello World!"),
            DocLine::from_str("Hello World!"),
        ];
        Self { rows }
    }

    pub fn get_line(&self, ind: usize) -> Option<&str> {
        self.rows.get(ind).map(|ln| ln.content.as_str())
    }

    pub fn get_styled_line(&self, ind: usize) -> Option<(&str, Style)> {
        self.rows
            .get(ind)
            .map(|ln| (ln.content.as_str(), Style::default())) // TODO: Actual styling
    }

    pub fn line_count(&self) -> usize {
        self.rows.len()
    }
}

impl Widget for &Document {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for row in 0..area.height {
            if let Some((ln, style)) = self.get_styled_line(row as usize) {
                buf.set_string(0, row, ln, style);
            } else {
                buf.set_string(0, row, "~", Style::default().dark_gray())
            }
        }
    }
}
