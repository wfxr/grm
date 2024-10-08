mod build;
mod fs;

pub mod http;
pub mod temp;

pub use build::*;
pub use fs::*;

use std::{
    error::Error,
    fmt::Write,
    io,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::Result;

pub fn not_found_err(e: &(dyn Error + 'static)) -> bool {
    matches!(e.downcast_ref::<io::Error>(), Some(e) if e.kind() == io::ErrorKind::NotFound)
}

pub trait Emojify {
    fn emojify(&self) -> String;
}

impl Emojify for str {
    // Ported from https://github.com/rossmacarthur/emojis/blob/083a8f2d2882c092305b42e1a05710338a2f82b0/examples/replace.rs
    fn emojify(&self) -> String {
        let mut input = self;
        let mut output = String::new();
        // The meaning of the index values is as follows.
        //
        //  : r o c k e t :
        // ^ ^           ^ ^
        // i m           n j
        //
        // i..j gives ":rocket:"
        // m..n gives "rocket"
        while let Some((i, m, n, j)) = input
            .find(':')
            .map(|i| (i, i + 1))
            .and_then(|(i, m)| input[m..].find(':').map(|x| (i, m, m + x, m + x + 1)))
        {
            match emojis::get_by_shortcode(&input[m..n]) {
                Some(emoji) => {
                    // Output everything preceding, except the first colon.
                    write!(output, "{}", &input[..i]).unwrap();

                    // Output the emoji.
                    write!(output, "{}", emoji.as_str()).unwrap();
                    // Update the string to past the last colon.
                    input = &input[j..];
                }
                None => {
                    // Output everything preceding but not including the colon.
                    write!(output, "{}", &input[..n]).unwrap();

                    // Update the string to start with the last colon.
                    input = &input[n..];
                }
            }
        }
        // output.write_all(input.as_bytes())
        write!(output, "{}", input).unwrap();
        output
    }
}

static HOME: LazyLock<PathBuf> =
    LazyLock::new(|| home::home_dir().expect("failed to determine the current user's home directory"));

pub trait Shorten {
    fn shorten(&self) -> Result<String>;
}

impl<T> Shorten for T
where
    T: AsRef<Path>,
{
    fn shorten(&self) -> Result<String> {
        let s = match self.as_ref().strip_prefix(&*HOME) {
            Ok(p) => Path::new("$HOME").join(p).to_str().map(|s| s.to_owned()),
            Err(_) => self.as_ref().to_str().map(|s| s.to_owned()),
        };
        s.ok_or_else(|| anyhow::anyhow!("failed to convert path to string: {:?}", self.as_ref()))
    }
}
