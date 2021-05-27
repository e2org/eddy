extern crate clap;
extern crate dirs;
extern crate ignore;
extern crate rand;
extern crate regex;
extern crate skim;

use clap::clap_app;
use ignore::WalkBuilder;
use rand::seq::SliceRandom;
use skim::prelude::{unbounded, Arc, Skim, SkimItemReceiver, SkimItemSender, SkimOptionsBuilder};
use std::env;
use std::ffi::OsStr;
use std::process::{self, Command, Stdio};
use std::thread;

use eddy::Args;

fn main() {
    let home = match dirs::home_dir() {
        Some(dir) => format!("{}", dir.display()),
        None => String::from(""),
    };

    let args = Args::new(
        clap_app!(edit =>
            (version: env!("CARGO_PKG_VERSION"))
            (author: env!("CARGO_PKG_AUTHORS"))
            (about: env!("CARGO_PKG_DESCRIPTION"))
            (@arg QUERY: default_value("") "query string used to search for projects")
            (@arg PATH: -p --path default_value(&home) "location to search for projects in")
            (@arg DEPTH: -d --depth +takes_value default_value("4") "max depth of directory search")
            (@arg verbose: -v --verbose "output info/debugging output to terminal")
            (@arg quiet: -q --quiet "suppress all output -- run silently")
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

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();
    let path = String::from(&args.path);
    let depth = args.depth; // Copy args so they may be moved into thread closure.
    thread::spawn(move || {
        for entry in WalkBuilder::new(path)
            .max_depth(Some(depth))
            .build()
            .flatten()
        {
            if entry.file_name() == OsStr::new("Cargo.toml") {
                if let Some(dir) = entry.path().parent() {
                    // Send new item to skim:
                    if tx
                        .send(Arc::new(format!("{}", dir.display()).replace(&home, "~")))
                        .is_err()
                    {
                        // Ignore possible error.
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

    // Run vim in selected project:
    Command::new("bash")
        .arg("-c")
        .arg(format!(
            "vim --not-a-term -O {0}/src/main.rs {0}/src/lib.rs -c redraw! < /dev/tty",
            // --not-a-term : tell vim it's being invoked programmatically to avoid warnings
            //           -O : open files in vertical split panes
            //   -c redraw! : avoid vim's "Press ENTER or type command to continue" message
            &choice
        ))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Error: Failed to execute vim.");
}
