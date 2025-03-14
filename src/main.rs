pub mod compressor;
pub mod error;
pub mod fstools;
pub mod ffmpeg;

use std::cell::RefCell;
use std::path::PathBuf;
use std::process::ExitCode;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

use ffmpeg::compressor::CompressorOptions;
use rustop::opts;
use fstools::{classify_file, DirEntryCategory};
use compressor::Compressor;
use signal_hook::{consts::{SIGINT, SIGHUP, SIGTERM}, iterator::Signals};

fn main() -> ExitCode {
    let (args, _rest) = opts! {
        synopsis "Compress mkv files for use by Jellyfin/Emby/etc";
        opt sample:bool=false, desc:"Transcode a small sample. (not implemented)";
        opt dry_run:bool=false, desc:"Describe what would be done, but don't actually do anything. (not implemented)";
        opt recursive:bool=false, desc:"Recurse into subdirectories. (not implemented)";
        opt codec:String=String::from("av1"), desc:"Codec to use for compression. [av1, hevc]";
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

    // if outfile is None, use infile path with replaced extension
    // if stdout is a tty, ask for an output filename with a suggested default
    // if infile is a directory, operate on its contents
    // if infile is a file, only operate on it
    let exit_code = match classify_file(&PathBuf::from(&args.infile)) {
        DirEntryCategory::Unknown => {
            println!("Unable to classify {:?}.", args.infile);
            ExitCode::FAILURE
        },
        DirEntryCategory::DoesNotExist => {
            println!("{:?} does not exist.", args.infile);
            ExitCode::FAILURE
        },
        DirEntryCategory::SymbolicLink => {
            println!("{:?} is a symlink.", args.infile);
            ExitCode::FAILURE
        },
        DirEntryCategory::Directory => {
            println!("{:?} is a directory.", args.infile);
            ExitCode::FAILURE
        },
        DirEntryCategory::RegularFile => match compressor.compress_file(&PathBuf::from(&args.infile), &PathBuf::from("")) {
            Ok(_) => {
                println!("Success! ^__^");
                ExitCode::SUCCESS
            },
            Err(err) => {
                println!("Failure -__-\n{}", err);
                ExitCode::FAILURE
            },
        },
    };

    exit_code
}
