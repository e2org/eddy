use clap::value_t;
use std::error;
use std::fmt;

// Standard "error-boxing" Result type:
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct Args {
    pub query: String,
    pub path: String,
    pub depth: usize,
    pub verbose: bool,
    pub quiet: bool,
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ARGS: query={} path={} depth={} verbose={} quiet={}",
            format!("\"{}\"", self.query),
            format!("\"{}\"", self.path),
            self.depth,
            self.verbose,
            self.quiet
        )
    }
}

impl Args {
    pub fn new(matches: clap::ArgMatches) -> Result<Args> {
        let query = value_t!(matches, "QUERY", String)?;
        let path = value_t!(matches, "PATH", String)?;
        let depth = value_t!(matches, "DEPTH", usize)?;
        let verbose = matches.is_present("verbose");
        let quiet = matches.is_present("quiet");
        Ok(Args {
            query,
            path,
            depth,
            verbose,
            quiet,
        })
    }
}
