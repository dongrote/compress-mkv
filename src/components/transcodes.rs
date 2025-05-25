use std::cmp::max;
use humanize_bytes::humanize_bytes_decimal;
use crate::transcode_state::{TranscodeState, TranscodeStatus};
use ratatui::{
    layout::Constraint,
    style::{
        palette::tailwind::{BLUE, SLATE}, Modifier, Style
    },
    text::Text,
    widgets::{Cell, Row, Table},
};

const HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);

pub struct TranscodesList {
    items: Vec<TranscodeState>,
}

struct Widths {
    status: u16,
    progress: u16,
    source_codec: u16,
    transcode_codec: u16,
    source_size: u16,
    transcode_size: u16,
    filename: u16,
}

impl Default for Widths {
    fn default() -> Self {
        Widths {
            status: 6, // "Status".len()
            progress: 8, // "Progress".len()
            source_codec: 12, // "Source Codec".len()
            transcode_codec: 15, // "Transcode Codec".len()
            source_size: 11, // "Source Size".len()
            transcode_size: 14, // "Transcode Size".len()
            filename: 8, // "Filename".len()
        }
    }
}

impl Widths {
    pub fn from_item(item: &TranscodeState) -> Self {
        let status_str = format!("{}", item.status);
        let progress_str = format!("{:.2}%", match item.progress {
            None => 0.0,
            Some(p) => p * 100.0,
        });
        let source_codec_str = match &item.source_codec {
            None => String::from("???"),
            Some(c) => format!("{}", c),
        };
        let transcode_codec_str = match &item.transcode_codec {
            None => String::from("???"),
            Some(c) => format!("{}", c),
        };
        let source_size_str = humanize_bytes_decimal!(match item.source_size {
            None => 0,
            Some(s) => s,
        });
        let transcode_size_str = humanize_bytes_decimal!(match item.current_transcoding_size {
            None => 0,
            Some(s) => s,
        });
        let filename_str = match &item.path {
            None => String::from("/"),
            Some(p) => p.display().to_string(),
        };

        Widths {
            status: status_str.len() as u16,
            progress: progress_str.len() as u16,
            source_codec: source_codec_str.len() as u16,
            transcode_codec: transcode_codec_str.len() as u16,
            source_size: source_size_str.len() as u16,
            transcode_size: transcode_size_str.len() as u16,
            filename: filename_str.len() as u16,
        }
    }

    pub fn from_items(items: &[TranscodeState]) -> Self {
        let mut widths = Self::default();
        for item in items {
            let item_widths = Self::from_item(item);
            widths.status = max(widths.status, item_widths.status);
            widths.progress = max(widths.progress, item_widths.progress);
            widths.source_codec = max(widths.source_codec, item_widths.source_codec);
            widths.transcode_codec = max(widths.transcode_codec, item_widths.transcode_codec);
            widths.source_size = max(widths.source_size, item_widths.source_size);
            widths.transcode_size = max(widths.transcode_size, item_widths.transcode_size);
            widths.filename = max(widths.filename, item_widths.filename);
        }

        widths
    }
}

impl TranscodesList {
    pub fn new(items: Vec<TranscodeState>) -> Self {
        TranscodesList { items, }
    }

    pub fn widget(&self) -> Table {
        let header_style = HEADER_STYLE;
        let header = Row::new([
                Cell::from(Text::from("Status").centered()),
                Cell::from(Text::from("Progress").right_aligned()),
                Cell::from(Text::from("Source Codec").right_aligned()),
                Cell::from(Text::from("Transcode Codec").right_aligned()),
                Cell::from(Text::from("Source Size").right_aligned()),
                Cell::from(Text::from("Transcoded Size").right_aligned()),
                Cell::from(Text::from("Filename")),
            ])
            .style(header_style)
            .height(1);

        let widths = Widths::from_items(&self.items);
        let rows = self.items.iter().map(|item| {
            let style = match item.status {
                TranscodeStatus::Transcoding => Style::default().bg(SLATE.c500).add_modifier(Modifier::BOLD),
                _ => Style::default(),
            };
            let progress = match item.progress {
                None => String::from("0.00%"),
                Some(p) => format!("{:.2}%", p * 100.0),
            };
            let source_codec = match &item.source_codec {
                Some(c) => format!("{}", c),
                None => String::from("Unknown"),
            };
            let transcode_codec = match &item.transcode_codec {
                Some(c) => format!("{}", c),
                None => String::from("Unknown"),
            };
            let source_size = humanize_bytes_decimal!(match item.source_size {
                None => 0,
                Some(s) => s,
            });
            let transcode_size = humanize_bytes_decimal!(match item.current_transcoding_size {
                None => 0,
                Some(s) => s,
            });
            let filename = match &item.path {
                None => String::from("???"),
                Some(p) => p.display().to_string(),
            };
            Row::new([
                    Cell::from(Text::from(format!("{}", item.status)).centered()),
                    Cell::from(Text::from(format!("{}", progress)).right_aligned()),
                    Cell::from(Text::from(format!("{}", source_codec)).right_aligned()),
                    Cell::from(Text::from(format!("{}", transcode_codec)).right_aligned()),
                    Cell::from(Text::from(format!("{}", source_size)).right_aligned()),
                    Cell::from(Text::from(format!("{}", transcode_size)).right_aligned()),
                    Cell::from(Text::from(filename)),
                ])
                .style(style)
                .height(1)
        });

        Table::new(
            rows,
            [
                Constraint::Length(widths.status + 2),
                Constraint::Length(widths.progress + 2),
                Constraint::Length(widths.source_codec + 2),
                Constraint::Length(widths.transcode_codec + 2),
                Constraint::Length(widths.source_size + 2),
                Constraint::Length(widths.transcode_size + 2),
                Constraint::Length(widths.filename + 2),
            ])
            .header(header)
    }
}