pub mod compressor;
pub mod error;
pub mod fstools;
pub mod ffmpeg;
pub mod file_path_handler;

use std::cell::RefCell;
use std::path::PathBuf;
use std::process::ExitCode;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

use ffmpeg::compressor::CompressorOptions;
use file_path_handler::FilePathHandler;
use rustop::opts;
use compressor::Compressor;
use signal_hook::{consts::{SIGINT, SIGHUP, SIGTERM}, iterator::Signals};

fn main() -> ExitCode {
    let (args, _rest) = opts! {
        synopsis "Compress mkv files for use by Jellyfin/Emby/etc";
        opt sample:bool=false, desc:"Transcode a small sample. (not implemented)";
        opt dry_run:bool=false, desc:"Describe what would be done, but don't actually do anything.";
        opt recursive:bool=false, desc:"Recurse into subdirectories. (not implemented)";
        opt codec:String=String::from("av1"), desc:"Codec to use for compression. [av1, hevc]";
        opt container:String=String::from("mkv"), desc:"Container";
        opt fast:bool=false, desc:"Use faster encoding parameters.";
        param infile:String, desc:"Input file/directory";
        param outfile:Option<String>, desc:"Output file (not implemented)";
    }.parse_or_exit();

    let f = ffmpeg::FFmpeg::new(); 
    if !f.is_installed() {
        println!("ffmpeg is not installed.");
        // offer to install for user
        // if install is successful, try again
        return ExitCode::FAILURE;
    }

    let (tx, rx) = mpsc::channel::<bool>();
    let rx = Rc::new(RefCell::new(rx));
    let compressor = Compressor::new(CompressorOptions {
        dry_run: args.dry_run,
        fast: args.fast,
        overwrite: false,
        codec: args.codec.to_lowercase(),
        container: args.container.to_lowercase(),
    }, Rc::clone(&rx));

    thread::spawn(move || {
        if let Ok(mut signals) = Signals::new(&[SIGINT, SIGHUP, SIGTERM]) {
            println!("Listening for SIGINT, SIGHUP, SIGTERM");
            for sig in signals.forever() {
                match sig {
                    SIGINT => println!("Caught SIGINT; signalling Compressor to stop."),
                    SIGHUP => println!("Caught SIGHUP; signalling Compressor to stop."),
                    SIGTERM => println!("Caught SIGTERM; signalling Compressor to stop."),
                    _ => continue,
                };

                let _ = tx.send(true);
                break;
            }
        } else {
            println!("Error registering signal handler; Ctrl-C will not save you.");
        }
    });

    let handler = FilePathHandler::for_pathbuf(
        PathBuf::from(&args.infile),
        &Rc::new(Box::new(compressor)));
    match handler.handle() {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
