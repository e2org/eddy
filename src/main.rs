extern crate clap;
extern crate dirs;
extern crate ignore;
extern crate rand;
extern crate skim;

use clap::clap_app;
use ignore::WalkBuilder;
use rand::seq::SliceRandom;
use skim::prelude::{unbounded, Arc, Skim, SkimItemReceiver, SkimItemSender, SkimOptionsBuilder};
use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::process::{self, Command, Stdio};
use std::thread;

use eddy::Args;

fn main() {
    let home = dirs::home_dir()
        .expect("Error: Could not determine $HOME directory.")
        .into_os_string()
        .into_string()
        .expect("Error: Could not determine $HOME directory.");

    let args = Args::new(
        clap_app!(edit =>
            (version: env!("CARGO_PKG_VERSION"))
            (author: env!("CARGO_PKG_AUTHORS"))
            (about: env!("CARGO_PKG_DESCRIPTION"))
            (@arg QUERY: default_value("") "query string used to search for projects")
            (@arg TARGET: -t --target +takes_value default_value("") "target file used to filter directories")
            (@arg SUBDIR: -s --subdir +takes_value default_value("") "subdirectory to open files from")
            (@arg FILES: -f --files +takes_value default_value("*") "files to open")
            (@arg EDITOR: -e --editor +takes_value default_value("vim") "editor command used to open files")
            (@arg PATH: -p --path +takes_value default_value(&home) "location to search for projects in")
            (@arg DEPTH: -d --depth +takes_value default_value("4") "max depth of directory search")
            (@arg verbose: -v --verbose "output info/debugging output to terminal")
        )
        .get_matches(),
    )
    .unwrap_or_else(|error| panic!("error: {:?}", error));

    if args.verbose {
        println!("{}", args);
    }

    let colorschemes = vec![
        // Colorscheme CMYK:
        "bg+:-1,border:#0000ff,pointer:#0bc7e3,prompt:#feaf3c,info:#0000ff,fg:#0000ff,fg+:#0bc7e3,hl:#ff00ff,hl+:#ff00ff",
        // Colorscheme Outrun:
        "bg+:-1,border:#541388,pointer:#ef2b63,prompt:#0bc7e3,info:#541388,fg:#541388,fg+:#ef2b63,hl:#0bc7e3,hl+:#0bc7e3",
        // Colorscheme Submariner:
        "bg+:-1,border:#1d485f,pointer:#0bc7e3,prompt:#db662d,info:#1d485f,fg:#1d485f,fg+:#0bc7e3,hl:#db662d,hl+:#db662d",
    ];
    let colorscheme = colorschemes.choose(&mut rand::thread_rng()).unwrap();

    let thread_target = String::from(&args.target);
    let thread_path = String::from(&args.path);
    let thread_depth = args.depth; // Copy args so they may be moved into thread closure.
    let thread_home = String::from(&home);

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();
    thread::spawn(move || {
        let mut items = HashSet::new();
        for entry in WalkBuilder::new(thread_path)
            .max_depth(Some(thread_depth))
            .build()
            .flatten()
        {
            if thread_target.is_empty() || entry.file_name() == OsStr::new(thread_target.as_str()) {
                if let Some(dir) = entry.path().parent() {
                    let item = dir.to_str().unwrap_or("").replace(&thread_home, "~");
                    // Avoid sending duplicate items to skim:
                    if !items.contains(&item) {
                        items.insert(String::from(&item));
                        // Send new item to skim:
                        if tx.send(Arc::new(item)).is_err() {
                            // Ignore possible error.
                        }
                    }
                }
            }
        }
        // Tell skim to stop waiting for items:
        drop(tx);
    });

    // Run skim to allow project selection:
    let result = Skim::run_with(
        &SkimOptionsBuilder::default()
            .query(Some(&args.query))
            .color(Some(colorscheme))
            .prompt(Some("$ "))
            .margin(Some("1,2"))
            .height(Some("40%"))
            .reverse(true)
            .inline_info(true)
            .select1(true)
            .build()
            .unwrap(),
        Some(rx),
    )
    .unwrap();

    // Allow ctrl+C to abort:
    if result.is_abort {
        process::exit(0);
    }

    // If no dir selected, use query string:
    let mut choice = result.query;
    for item in result.selected_items.iter() {
        choice = String::from(item.output());
    }

    if env::set_current_dir(choice.replace("~", &home)).is_err() {
        println!("{}: Directory does not exist.", choice);
        process::exit(1);
    };

    if !args.subdir.is_empty() && env::set_current_dir(args.subdir).is_err() {
        println!("{}: Directory does not exist.", choice);
        process::exit(1);
    };

    // Run editor command on selected files in selected directory:
    Command::new("bash")
        .arg("-c")
        .arg(format!("{} {} </dev/tty", args.editor, args.files))
        // </dev/tty : necessary to restore control to terminal after exit
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Error: Failed to execute vim.");

    // Done!
    process::exit(0);
}
