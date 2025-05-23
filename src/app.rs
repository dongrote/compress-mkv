use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{
        palette::tailwind::{BLUE, GREEN, PURPLE, RED, SLATE, YELLOW}, Color, Style, Stylize
    },
    symbols, 
    text::{Line, Text}, 
    widgets::{Block, Borders, LineGauge, List, ListItem, Padding, Paragraph, StatefulWidget, TableState, Widget},
    DefaultTerminal
};
use std::{path::PathBuf, sync::mpsc, thread::JoinHandle, time::Duration};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use humanize_bytes::humanize_bytes_decimal;

use crate::{
    codecs::Codec,
    containers::Container,
    filelist::FileList,
    filelistitem::{FileListItem, FileListItemStatus},
    filescanner::FileScanner,
    quality::Quality,
    queue_processor::{QueueProcessor, QueueProcessorMessage},
    transcode_state::{TranscodeState, TranscodeStatus},
    transcode_task::TranscodeTask,
    components::FileList as FileListWidget,
};

const HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const ROW_BG_COLOR: Color = SLATE.c950;
const TEXT_FG_COLOR: Color = SLATE.c200;
const FILELISTITEM_UNKNOWN_FG_COLOR: Color = SLATE.c50;
const FILELISTITEM_INVALID_FG_COLOR: Color = RED.c800;
const FILELISTITEM_CANDIDATE_FG_COLOR: Color = SLATE.c200;
const FILELISTITEM_ENQUEUED_FG_COLOR: Color = PURPLE.c500;
const FILELISTITEM_TRANSCODING_FG_COLOR: Color = YELLOW.c500;
const FILELISTITEM_TRANSCODED_FG_COLOR: Color = GREEN.c500;
const FILELISTITEM_ANALYZING_FG_COLOR: Color = PURPLE.c300;


pub struct App {
    base_path: PathBuf,
    stop: Arc<Mutex<bool>>,
    file_list: Arc<Mutex<FileList>>,
    queue: Arc<Mutex<VecDeque<TranscodeTask>>>,
    transcode_state: Arc<Mutex<TranscodeState>>,
    files_state: TableState,
    queue_processor_thread: Option<JoinHandle<()>>,
    quality: Quality,
}

impl Default for App {
    fn default() -> Self {
        App::excellent(PathBuf::from("."))
    }
}

impl App {
    pub fn insane(path: PathBuf) -> Self {
        App::new(path, Quality::Insane)
    }

    pub fn excellent(path: PathBuf) -> Self {
        App::new(path, Quality::Excellent)
    }

    pub fn great(path: PathBuf) -> Self {
        App::new(path, Quality::Great)
    }

    pub fn good(path: PathBuf) -> Self {
        App::new(path, Quality::Good)
    }

    pub fn fast(path: PathBuf) -> Self {
        App::new(path, Quality::Fast)
    }

    pub fn new(path: PathBuf, quality: Quality) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let stop = Arc::new(Mutex::new(false));
        App {
            queue,
            stop,
            quality,
            base_path: path,
            file_list: Arc::new(Mutex::new(FileList::new())),
            transcode_state: Arc::new(Mutex::new(TranscodeState::new())),
            files_state: TableState::default().with_selected(0),
            queue_processor_thread: None,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        self.scan_for_files();
        self.queue_processor();
        while !self.should_stop() {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
                // check system resources
            }
        }

        self.wait_for_threads();
        Ok(())
    }

    fn wait_for_threads(&mut self) {
        if let Some(handle) = self.queue_processor_thread.take() {
            print!("waiting for queue_processor thread ...");
            match handle.join() {
                Ok(()) => println!("joined"),
                Err(err) => println!("error: {:?}", err),
            }
        }
    }

    fn should_stop(&self) -> bool { *self.stop.lock().unwrap() }

    fn trigger_stop(&mut self) {
        let mut s = self.stop.lock().unwrap();
        *s = true;
    }

    fn queue_processor(&mut self) {
        let file_list = Arc::clone(&self.file_list);
        let queue = Arc::clone(&self.queue);
        let stop = Arc::clone(&self.stop);
        let xcode_state = Arc::clone(&self.transcode_state);
        let mut processor = QueueProcessor::new(queue, stop);
        let queue_processor_messages = processor.subscribe();
        self.queue_processor_thread = Some(thread::spawn(move || processor.forever()));
        thread::spawn(move || {
            for msg in queue_processor_messages {
                match msg {
                    QueueProcessorMessage::Idle => {
                        let mut state = xcode_state.lock().unwrap();
                        state.status = TranscodeStatus::Idle;
                    },
                    QueueProcessorMessage::TranscodeStart(task) => {
                        {
                            let mut state = xcode_state.lock().unwrap();
                            state.status = TranscodeStatus::Transcoding;
                            state.path = Some(task.source.clone());
                            state.progress = None;
                            state.source_size = task.metadata.file_size;
                        }
                        {
                            let fl = file_list.lock().unwrap();
                            let opt = fl.items.iter().find(|i| i.lock().unwrap().path == task.source);
                            if let Some(item) = opt {
                                let mut i = item.lock().unwrap();
                                i.status = FileListItemStatus::Transcoding;
                            }
                        }
                    },
                    QueueProcessorMessage::Transcoding(xmsg) => {
                        let mut state = xcode_state.lock().unwrap();
                        state.progress = Some(xmsg.progress);
                        state.current_transcoding_size = Some(xmsg.current_size);
                        state.predicted_transcoded_size = Some(xmsg.predicted_size);
                        state.frames = Some(xmsg.frames);
                        state.total_frames = Some(xmsg.total_frames);
                    },
                    QueueProcessorMessage::TranscodeEnd(task, _final_size) => {
                        {
                            let mut state = xcode_state.lock().unwrap();
                            state.status = TranscodeStatus::Idle;
                            state.path = None;
                            state.progress = None;
                            state.source_size = None;
                            state.current_transcoding_size = None;
                            state.predicted_transcoded_size = None;
                            state.frames = None;
                            state.total_frames = None;
                        }
                        {
                            let fl = file_list.lock().unwrap();
                            let opt = fl.items.iter().find(|i| i.lock().unwrap().path == task.source);
                            if let Some(item) = opt {
                                let mut i = item.lock().unwrap();
                                i.status = FileListItemStatus::Transcoded;
                            }
                        }
                    },
                }
            }
        });
    }

    fn scan_for_files(&mut self) {
        let files = Arc::clone(&self.file_list);
        let path = self.base_path.clone();
        let (tx, rx) = mpsc::channel();
        let recursive = true;
        let scanner = FileScanner::new(recursive);
        thread::spawn(move || scanner.scan(path, tx));
        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Err(_) => break,
                    Ok(file_list_item) => {
                        let mut file_list = files.lock().unwrap();
                        file_list.insert(file_list_item);
                    },
                }
            }
        });
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.trigger_stop(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char(' ') | KeyCode::Right => self.toggle_enqueue_selected(),
            //KeyCode::Enter => self.analyze_selected(),
            _ => {},
        }
    }

    fn select_next(&mut self) {
        self.files_state.select_next();
    }

    fn select_prev(&mut self) {
        self.files_state.select_previous();
    }

    fn toggle_enqueue_selected(&mut self) {
        if let Some(i) = self.files_state.selected() {
            let maybe_item = {
                let file_list = self.file_list.lock().unwrap();
                file_list.get(i)
            };
            if let Some(item) = maybe_item {
                let (path, status) = {
                    let i = item.lock().unwrap();
                    (i.path.clone(), i.status.clone())
                };
                match status {
                    FileListItemStatus::Enqueued => {
                        // user wants to remove this FileListItem from the
                        // processing queue
                        {
                            let mut q = self.queue.lock().unwrap();
                            let index = {
                                let mut index = 0;
                                for i in &*q {
                                    if *i.source == path {
                                        break;
                                    }
                                    index = index + 1;
                                }
                                index
                            };
                            q.remove(index);
                        }
                        {
                            let mut i = item.lock().unwrap();
                            i.set_candidate();
                        }
                    },
                    FileListItemStatus::Candidate => {
                        // only permit enqueueing if this is a Candidate,
                        // Candidate means it can be transocded,
                        // it is not already enqueued, and it is not being
                        // transcoded
                        let codec = Codec::AV1;
                        let container = Container::Matroska;
                        let quality = self.quality.clone();
                        let (metadata, source) = {
                            let i = item.lock().unwrap();
                            (i.avmetadata.clone(), i.path.clone())
                        };
                        let mut destination = source.clone();
                        destination.set_extension(format!("{}.{}", &codec, Container::extension(container)));
                        let task = TranscodeTask {
                            source,
                            destination,
                            metadata,
                            codec,
                            container,
                            quality,
                        };
                        {
                            let mut q = self.queue.lock().unwrap();
                            q.push_back(task);
                        }
                        {
                            item.lock().unwrap().set_enqueued();
                        }
                    },
                    FileListItemStatus::Transcoding => {
                        // item is in the process of transcoding so we need
                        // to kill ffmpeg and then remove this item from
                        // the queue
                    },
                    _ => {},
                }
            }
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where Self: Sized {
        let [header_area, main_area, status_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .areas(area);

        let [list_area, queue_area] = Layout::vertical([
            Constraint::Fill(2),
            Constraint::Fill(1),
        ]).areas(main_area);

        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_queue(queue_area, buf);
        self.render_transcoding_status(status_area, buf);
    }
}

impl App {
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Video Transcoder")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new(Text::from(vec![
                Line::raw("🤷 Unknown 🚫 Invalid 👍 Candidate → Enqueued 🚧 Transcoding ✅ Transcoded 🔎 Analyzing"),
                Line::raw("Press q/esc to exit; j to move down, k to move up, space to enqueue/dequeue"),
            ]))
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let items = {
            let fl = self.file_list.lock().unwrap();
            fl.snapshot()
        };

        StatefulWidget::render(FileListWidget::new(items).widget(), area, buf, &mut self.files_state)
    }

    fn render_queue(&mut self, area: Rect, buf: &mut Buffer) {
        let queue_snapshot: Vec<TranscodeTask> = {
            let q = self.queue.lock().unwrap();
            q.iter().map(|i| i.clone()).collect()
        };
        let items: Vec<ListItem> = queue_snapshot
            .iter()
            .map(|t| ListItem::from(t.source.to_string_lossy()).fg(SLATE.c300))
            .collect();

        let block = Block::new()
            .title(Line::raw("Queue").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(HEADER_STYLE)
            .bg(ROW_BG_COLOR)
            .padding(Padding::horizontal(1));

        let list = List::new(items)
            .block(block)
            .fg(TEXT_FG_COLOR);

        Widget::render(list, area, buf);
    }

    //fn render_logs(&mut self, area: Rect, buf: &mut Buffer) {
    //    let mut logs = vec![];
    //    {
    //        let start = area.height as usize;
    //        let l = self.logs.lock().unwrap();
    //        for log in if l.len() > (start<<1) { &l[start..] } else { &l[..] } {
    //            logs.push(log.clone());
    //        }
    //    };
    //
    //    logs.reverse();
    //    let items: Vec<ListItem> = logs
    //        .into_iter()
    //        .map(|l| ListItem::from(l).fg(SLATE.c300))
    //        .collect();
    //
    //    let block = Block::new()
    //        .title(Line::raw("Logs").centered())
    //        .borders(Borders::TOP)
    //        .border_set(symbols::border::EMPTY)
    //        .border_style(HEADER_STYLE)
    //        .bg(ROW_BG_COLOR)
    //        .padding(Padding::horizontal(1));
    //
    //    let list = List::new(items)
    //        .block(block)
    //        .fg(TEXT_FG_COLOR);
    //
    //    Widget::render(list, area, buf);
    //}

    fn render_transcoding_status(&mut self, area: Rect, buf: &mut Buffer) {
        let label = {
            let x = self.transcode_state.lock().unwrap();
            match (*x).status {
                TranscodeStatus::Idle => String::from("Idle"),
                TranscodeStatus::Transcoding => match &(*x).path {
                    None => String::from("undefined"),
                    Some(p) => PathBuf::from(p.file_name().unwrap_or(p.as_os_str())).display().to_string(),
                },
            }
        };
        let ratio = {
            let x = self.transcode_state.lock().unwrap();
            match (*x).progress {
                None => 0.0,
                Some(r) => r,
            }
        };
        let title = {
            let x = self.transcode_state.lock().unwrap();
            format!(
                "{:?} || current size {} || predicted size {} || ({}/{})",
                x.status,
                humanize_bytes_decimal!(x.current_transcoding_size.unwrap_or(0)),
                humanize_bytes_decimal!(x.predicted_transcoded_size.unwrap_or(0)),
                x.frames.unwrap_or(0), x.total_frames.unwrap_or(0))
        };

        let block = Block::new().title(Line::raw(title));

        let gauge = LineGauge::default()
            .label(label)
            .block(block)
            .filled_style(Style::new().light_green().bold())
            .unfilled_style(Style::new().white())
            .ratio(ratio);

        Widget::render(gauge, area, buf);
    }
}

impl From<&FileListItem> for ListItem<'_> {
    fn from(value: &FileListItem) -> Self {
        let line = match value.status {
            FileListItemStatus::Unknown => Line::styled(format!("{}", &value), FILELISTITEM_UNKNOWN_FG_COLOR),
            FileListItemStatus::Invalid => Line::styled(format!("{}", &value), FILELISTITEM_INVALID_FG_COLOR),
            FileListItemStatus::Candidate => Line::styled(format!("{}", &value), FILELISTITEM_CANDIDATE_FG_COLOR),
            FileListItemStatus::Enqueued => Line::styled(format!("{}", &value), FILELISTITEM_ENQUEUED_FG_COLOR),
            FileListItemStatus::Transcoding => Line::styled(format!("{}", &value), FILELISTITEM_TRANSCODING_FG_COLOR),
            FileListItemStatus::Transcoded => Line::styled(format!("{}", &value), FILELISTITEM_TRANSCODED_FG_COLOR),
            FileListItemStatus::Analyzing => Line::styled(format!("{}", &value), FILELISTITEM_ANALYZING_FG_COLOR),
        };

        ListItem::new(line)
    }
}
