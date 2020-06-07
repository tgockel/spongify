use structopt::{clap::ArgGroup, StructOpt};
use std::{
    fs,
    io,
    path::PathBuf,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(StructOpt, Debug)]
struct InputOpt {
    /// Inline text to SpOnGiFy. If specified as `-`, SpOnGiFy will read from standard input.
    #[structopt(group = "input")]
    inline: Option<String>,
    /// The text to SpOnGiFy. This can be useful if your text is `-`.
    #[structopt(long, group = "input")]
    text: Option<String>,
    /// Load text from a file.
    #[structopt(long, short, group = "input")]
    file: Option<PathBuf>,
    /// Read from standard input.
    #[structopt(long, group = "input")]
    stdin: bool,
}

struct StringReader {
    pub source: String,
    pub offset: usize,
}

impl io::Read for StringReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let src = &self.source.as_bytes()[self.offset..];
        let count = std::cmp::min(src.len(), buf.len());
        for (d, s) in buf.iter_mut().zip(src.iter()) {
            *d = *s;
        }
        self.offset += count;
        Ok(count)
    }
}

impl InputOpt {
    pub fn get_reader(&self) -> Result<Box<dyn io::BufRead>> {
        if self.stdin || self.inline.as_ref().map(|x| &x[..] == "-").unwrap_or(false) {
            Ok(Box::new(io::BufReader::new(io::stdin())))
        } else if let Some(ref path) = self.file {
            let f = fs::File::open(path)?;
            Ok(Box::new(io::BufReader::new(f)))
        } else {
            let text = self.text.as_ref().or(self.inline.as_ref()).unwrap().clone();
            Ok(Box::new(io::BufReader::new(StringReader{ source: text, offset: 0 })))
        }
    }
}

#[derive(StructOpt, Debug)]
struct OutputOpt {
    #[structopt(group = "output")]
    output_file: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
#[structopt(group = ArgGroup::with_name("input").required(true))]
struct Opt {
    #[structopt(flatten)]
    input: InputOpt,

    #[structopt(flatten)]
    output: OutputOpt,
}

fn main() -> Result<()> {
    use io::BufRead;

    let opt = Opt::from_args();

    let input = opt.input.get_reader()?;

    for line in input.lines() {
        let line = line?;

        for (idx, c) in line.chars().enumerate() {
            if idx % 2 == 0 {
                print!("{}", c.to_uppercase());
            } else {
                print!("{}", c.to_lowercase());
            }
        }
        println!();
    }

    Ok(())
}
