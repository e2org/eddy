extern crate clap;
extern crate dirs;
extern crate ignore;
extern crate skim;

use clap::clap_app;
use ignore::WalkBuilder;
use skim::prelude::{unbounded, Arc, Skim, SkimItemReceiver, SkimItemSender, SkimOptionsBuilder};
use std::env;
use std::ffi::OsStr;
use std::process::{self, Command, Stdio};
use std::thread;

use eddy::{colorscheme, Args};

fn main() {
    let home_err_msg = "Error: Could not determine $HOME directory.";
    let home = dirs::home_dir()
        .expect(home_err_msg)
        .into_os_string()
        .into_string()
        .expect(home_err_msg);

    let args = Args::new(
        clap_app!(eddy =>
            (version: env!("CARGO_PKG_VERSION"))
            (author: env!("CARGO_PKG_AUTHORS"))
            (about: env!("CARGO_PKG_DESCRIPTION"))
            (@arg QUERY: default_value("")
             "query string used to search for projects")
            (@arg SOURCE: -s --source +takes_value default_value(&home)
             "location to search for projects in")
            (@arg TARGET: -t --target +takes_value default_value("")
             "file/directory to used to filter choices for fuzzy-searching")
            (@arg PATHS: -p --paths +takes_value +multiple default_value("")
             "specific files/directories to open")
            (@arg EDITOR: -e --editor +takes_value default_value("")
             "editor command used to open files/directories")
            (@arg COLOR: -c --color +takes_value default_value("")
             "choose color scheme")
            (@arg DEPTH: -d --depth +takes_value default_value("4")
             "max depth of directory search")
            (@arg fullscreen: -f --fullscreen
             "run selector ui in fullscreen mode")
            (@arg nopreview: -n --nopreview
             "do not show preview window for selections")
            (@arg verbose: -v --verbose
             "output info/debugging output to terminal")
        )
        .get_matches(),
    )
    .unwrap_or_else(|error| panic!("error: {:?}", error));

    if args.verbose {
        println!("{}", args);
    }

    let thread_paths = String::from(&args.paths);
    let thread_source = String::from(&args.source);
    let thread_target = String::from(&args.target);
    let thread_depth = args.depth; // Copy args so they may be moved into thread closure.
    let thread_home = String::from(&home);

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();
    thread::spawn(move || {
        for entry in WalkBuilder::new(thread_source)
            .max_depth(Some(thread_depth))
            .build()
            .flatten()
        {
            if thread_target.is_empty() {
                if let Some(file_type) = entry.file_type() {
                    // Paths provided --> search directories
                    // OR
                    // No paths provided --> search files
                    if (thread_paths.is_empty() && file_type.is_file())
                        || (!thread_paths.is_empty() && !file_type.is_file())
                    {
                        let item = entry.path().to_str().unwrap_or("");
                        // Send new item to skim:
                        if tx.send(Arc::new(item.replace(&thread_home, "~"))).is_err() {
                            continue; // Ignore possible error.
                        }
                    }
                } else {
                    continue; // Ignore entry.
                }
            } else if entry.file_name() == OsStr::new(thread_target.as_str()) {
                if let Some(file_type) = entry.file_type() {
                    let mut path = entry.path();
                    if file_type.is_file() {
                        if let Some(parent_dir) = entry.path().parent() {
                            path = parent_dir;
                        } else {
                            continue; // Ignore error retrieving parent directory.
                        }
                    }
                    for target_entry in WalkBuilder::new(path)
                        .max_depth(Some(thread_depth))
                        .build()
                        .flatten()
                    {
                        if let Some(file_type) = target_entry.file_type() {
                            // Paths provided --> search directories
                            // OR
                            // No paths provided --> search files
                            if (thread_paths.is_empty() && file_type.is_file())
                                || (!thread_paths.is_empty() && !file_type.is_file())
                            {
                                let item = target_entry.path().to_str().unwrap_or("");
                                // Send new item to skim:
                                if tx.send(Arc::new(item.replace(&thread_home, "~"))).is_err() {
                                    continue; // Ignore possible error.
                                }
                            }
                        } else {
                            continue; // Ignore entry.
                        }
                    }
                } else {
                    continue; // Ignore error retrieving file type.
                }
            }
        }

        // Tell skim to stop waiting for items:
        drop(tx);
    });

    let mut options = SkimOptionsBuilder::default();

    options
        .query(Some(&args.query))
        .color(Some(colorscheme(&args)))
        .prompt(Some("$ "))
        .margin(Some("1,2"))
        .height(Some("40%"))
        .reverse(true)
        .inline_info(true)
        .select1(true);

    if args.fullscreen {
        options.height(Some("100%"));
    }

    if !args.nopreview {
        options.preview(Some(
            "
            FILE={}
            FILE=\"${FILE/#~/$HOME}\"
            if ! command -v bat; then
                cat $FILE
            else
                bat --color=always --style=plain $FILE
            fi
            ",
        ));
    }

    // Run skim to allow project selection:
    let result = Skim::run_with(&options.build().unwrap(), Some(rx)).unwrap();

    // Allow ctrl+C to abort:
    if result.is_abort {
        process::exit(0);
    }

    // If no dir selected, use query string:
    let mut choice = result.query;
    for item in result.selected_items.iter() {
        choice = String::from(item.output());
    }

    let paths;
    if args.paths.is_empty() {
        // If no files specified, open choice directly as file path:
        paths = choice;
    } else {
        paths = args.paths;
        // If files specified, change dir so paths are relative to choice:
        if env::set_current_dir(choice.replace("~", &home)).is_err() {
            println!("{}: Directory does not exist.", choice);
            process::exit(1);
        };
    }

    if args.editor.is_empty() {
        println!("{}", paths);
    } else {
        // Run editor command on selected files in selected directory:
        Command::new("bash")
            .arg("-c")
            .arg(format!("{} {} </dev/tty", args.editor, paths))
            // </dev/tty : necessary to restore control to terminal after exit
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Error: Failed to invoke selected editor.");
    }

    // Done!
    process::exit(0);
}
