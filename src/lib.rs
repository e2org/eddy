extern crate rand;

use clap::value_t;
use rand::seq::SliceRandom;
use std::error;
use std::fmt;

// Standard "error-boxing" Result type:
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct Args {
    pub query: String,
    pub source: String,
    pub target: String,
    pub paths: String,
    pub editor: String,
    pub color: String,
    pub depth: usize,
    pub fullscreen: bool,
    pub nopreview: bool,
    pub verbose: bool,
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ARGS: query={} source={} target={} paths={} editor={} color={} depth={} fullscreen={} nopreview={} verbose={}",
            format!("\"{}\"", self.query),
            format!("\"{}\"", self.source),
            format!("\"{}\"", self.target),
            format!("\"{}\"", self.paths),
            format!("\"{}\"", self.editor),
            format!("\"{}\"", self.color),
            self.depth,
            self.fullscreen,
            self.nopreview,
            self.verbose,
        )
    }
}

impl Args {
    pub fn new(matches: clap::ArgMatches) -> Result<Args> {
        let query = value_t!(matches, "QUERY", String)?;
        let source = value_t!(matches, "SOURCE", String)?;
        let target = value_t!(matches, "TARGET", String)?;
        let mut editor = value_t!(matches, "EDITOR", String)?;
        let color = value_t!(matches, "COLOR", String)?;
        let depth = value_t!(matches, "DEPTH", usize)?;
        let fullscreen = matches.is_present("fullscreen");
        let nopreview = matches.is_present("nopreview");
        let verbose = matches.is_present("verbose");

        // Allow specifying multiple --paths argments:
        let paths = match matches.values_of("PATHS") {
            Some(arr) => arr.collect::<Vec<&str>>().join(" "),
            None => String::from(""),
        };

        // Expand editor arg into full command:
        editor = match editor.as_str() {
            "vim" => String::from("vim --not-a-term -c redraw! -O"),
            // --not-a-term : avoid warnings due to programmatic invocation
            //           -O : open files in vertical split panes
            //   -c redraw! : avoid "Press ENTER or type command to continue"
            _ => editor,
        };

        Ok(Args {
            query,
            source,
            target,
            paths,
            editor,
            color,
            depth,
            fullscreen,
            nopreview,
            verbose,
        })
    }
}

pub fn colorscheme(args: &Args) -> &'static str {
    let colorschemes = vec![
        // Colorscheme CMYK:
        "bg+:-1,border:#0000ff,pointer:#0bc7e3,prompt:#feaf3c,\
            info:#0000ff,fg:#0000ff,fg+:#0bc7e3,hl:#ff00ff,hl+:#ff00ff",
        // Colorscheme Outrun:
        "bg+:-1,border:#541388,pointer:#ef2b63,prompt:#0bc7e3,\
            info:#541388,fg:#541388,fg+:#ef2b63,hl:#0bc7e3,hl+:#0bc7e3",
        // Colorscheme Submariner:
        "bg+:-1,border:#1d485f,pointer:#0bc7e3,prompt:#db662d,\
            info:#1d485f,fg:#1d485f,fg+:#0bc7e3,hl:#db662d,hl+:#db662d",
    ];

    // Return colorscheme matching provided --color arg:
    match args.color.as_str() {
        "cmyk" | "c" => colorschemes[0],
        "outrun" | "o" => colorschemes[1],
        "submariner" | "s" => colorschemes[2],
        "random" | "r" => colorschemes.choose(&mut rand::thread_rng()).unwrap(),
        _ => colorschemes[2], // default Submariner
    }
}
