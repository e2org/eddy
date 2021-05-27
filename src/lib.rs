use clap::value_t;
use std::error;
use std::fmt;

// Standard "error-boxing" Result type:
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct Args {
    pub query: String,
    pub target: String,
    pub subdir: String,
    pub editor: String,
    pub path: String,
    pub files: String,
    pub depth: usize,
    pub verbose: bool,
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ARGS: query={} target={} subdir={} editor={} path={} files={} depth={} verbose={}",
            format!("\"{}\"", self.query),
            format!("\"{}\"", self.target),
            format!("\"{}\"", self.subdir),
            format!("\"{}\"", self.editor),
            format!("\"{}\"", self.path),
            format!("\"{}\"", self.files),
            self.depth,
            self.verbose,
        )
    }
}

impl Args {
    pub fn new(matches: clap::ArgMatches) -> Result<Args> {
        let query = value_t!(matches, "QUERY", String)?;
        let target = value_t!(matches, "TARGET", String)?;
        let subdir = value_t!(matches, "SUBDIR", String)?;
        let mut editor = value_t!(matches, "EDITOR", String)?;
        let path = value_t!(matches, "PATH", String)?;
        let files = value_t!(matches, "FILES", String)?;
        let depth = value_t!(matches, "DEPTH", usize)?;
        let verbose = matches.is_present("verbose");

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
            target,
            subdir,
            editor,
            path,
            files,
            depth,
            verbose,
        })
    }
}
