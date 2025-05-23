pub mod app;
pub mod codecs;
pub mod components;
pub mod containers;
pub mod error;
pub mod filelist;
pub mod filelistitem;
pub mod filescanner;
pub mod probe;
pub mod queue_processor;
pub mod quality;
pub mod transcode_state;
pub mod transcode_task;
pub mod transcoder;

use ratatui;

use crate::app::App;

fn main() {
    let terminal = ratatui::init();
    let _ = App::default().run(terminal);
    ratatui::restore();
}
