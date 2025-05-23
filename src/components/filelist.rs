use std::cmp::max;
use humanize_bytes::humanize_bytes_decimal;
use crate::{
    codecs::Codec,
    filelistitem::{FileListItem, FileListItemStatus},
};
use ratatui::{
    layout::Constraint,
    style::{
        palette::tailwind::{SLATE, BLUE, YELLOW, GREEN},
        Modifier,
        Style,
    },
    text::Text,
    widgets::{Cell, Row, Table},
};

const HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const SELECTED_STYLE: Style = Style::new().bg(BLUE.c500).fg(SLATE.c100).add_modifier(Modifier::BOLD);

pub struct FileList {
    items: Vec<FileListItem>,
}

struct Widths {
    status: u16,
    resolution: u16,
    codec: u16,
    size: u16,
    filename: u16,
}

impl Default for Widths {
    fn default() -> Self {
        Widths {
            status: 6, // "Status".len(),
            resolution: 10, // "Resolution".len()
            codec: 6, // "Codec".len()
            size: 4, // "Size".len()
            filename: 8, // "Filename".len()
        }
    }
}

impl Widths {
    pub fn from_items(items: &[FileListItem]) -> Self {
        let mut widths = Self::default();
        for item in items {
            let resolution = match &item.resolution {
                Some(res) => res,
                None => &String::from("N/A"),
            };
            let codec = match &item.codec {
                Some(codec) => codec,
                None => &Codec::Unknown(String::from("???")),
            };
            widths.resolution = max(widths.resolution, resolution.len() as u16);
            widths.codec = max(widths.codec, codec.to_string().len() as u16);
            let size_str = match item.size {
                Some(size) => &format!("{}", humanize_bytes_decimal!(size)),
                None => "??? B",
            };
            widths.size = max(widths.size, size_str.len() as u16);
            widths.filename = max(widths.filename, item.path.display().to_string().len() as u16);
        }
        widths
    }
}

impl FileList {
    pub fn new(items: Vec<FileListItem>) -> Self {
        FileList { items, }
    }

    pub fn widget(&self) -> Table {
        let header_style = HEADER_STYLE;
        let selected_row_style = SELECTED_STYLE;

        let header = ["Status", "Resolution", "Codec", "Size", "Filename"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);

        let widths = Widths::from_items(&self.items);
        let rows = self.items.iter().map(|item| {
            let style = match item.status {
                FileListItemStatus::Enqueued => Style::new().fg(BLUE.c600),
                FileListItemStatus::Transcoding => Style::new().fg(YELLOW.c600),
                FileListItemStatus::Transcoded => Style::new().fg(GREEN.c600).add_modifier(Modifier::BOLD),
                _ => Style::default(),
            };
            let resolution = match &item.resolution {
                Some(res) => res,
                None => &String::from("N/A"),
            };
            let codec = match &item.codec {
                Some(codec) => codec,
                None => &Codec::Unknown(String::from("???")),
            };
            let size_str = match item.size {
                Some(size) => &format!("{}", humanize_bytes_decimal!(size)),
                None => "??? B",
            };
            Row::new([
                    Cell::from(Text::from(format!("{}", item.status))),
                    Cell::from(Text::from(format!("{}", resolution))),
                    Cell::from(Text::from(format!("{}", codec))),
                    Cell::from(Text::from(format!("{}", size_str))),
                    Cell::from(Text::from(format!("{}", &item.path.display()))),
                ])
                .style(style)
                .height(1)
        });

        Table::new(
            rows,
            [
                Constraint::Length(widths.status + 2),
                Constraint::Length(widths.resolution + 2),
                Constraint::Length(widths.codec + 2),
                Constraint::Length(widths.size + 2),
                Constraint::Length(widths.filename + 2),
            ])
            .header(header)
            .row_highlight_style(selected_row_style)
    }
}