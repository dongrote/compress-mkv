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
use file_path_handler::{FilePathHandler, FilePathHandlerOptions};
use rustop::opts;
use compressor::Compressor;
use signal_hook::{consts::{SIGINT, SIGHUP, SIGTERM}, iterator::Signals};

fn main() -> ExitCode {
    let (args, _rest) = opts! {
        synopsis "Compress mkv files for use by Jellyfin/Emby/etc";
        version env!("CARGO_PKG_VERSION");
        opt sample:bool=false, desc:"Transcode a small sample. (not implemented)";
        opt dry_run:bool=false, desc:"Describe what would be done, but don't actually do anything.";
        opt recursive:bool=true, desc:"Do not recurse into subdirectories.";
        opt codec:String=String::from("av1"), desc:"Codec to use for compression. [av1, hevc]";
        opt container:String=String::from("mkv"), desc:"Container";
        opt fast:bool=false, desc:"Use faster encoding parameters.";
        opt extreme:bool=false, desc:"Compress with extreme high quality.";
        param infiles:Vec<String>, desc:"Input files/directories";
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
        extreme: args.extreme,
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

    let mut exit_code = ExitCode::SUCCESS;
    let rc_compressor = Rc::new(Box::new(compressor));
    for infile in args.infiles {
        let handler = FilePathHandler::for_pathbuf(
            PathBuf::from(&infile),
            FilePathHandlerOptions { recursive: args.recursive, },
            &Rc::clone(&rc_compressor));
        if handler.handle().is_err() {
            exit_code = ExitCode::FAILURE;
        }
    }

    exit_code
}
