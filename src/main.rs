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

impl InputOpt {
    pub fn get_reader(&self) -> Result<Box<dyn io::BufRead>> {
        if self.stdin || self.inline.as_ref().map(|x| &x[..] == "-").unwrap_or(false) {
            Ok(Box::new(io::BufReader::new(io::stdin())))
        } else if let Some(ref path) = self.file {
            let f = fs::File::open(path)?;
            Ok(Box::new(io::BufReader::new(f)))
        } else {
            let text = self.text.as_ref().or(self.inline.as_ref()).unwrap().clone();
            Ok(Box::new(io::Cursor::new(text)))
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
struct OutputOpt {
    /// Output to a file.
    #[structopt(short, long, group = "output")]
    output_file: Option<PathBuf>,

    /// Copy result to the clipboard.
    #[structopt(short, long, group = "output")]
    clip: bool,
}

impl OutputOpt {
    pub fn get_writer(&self) -> Result<Box<dyn io::Write>> {
        if let Some(ref path) = self.output_file {
            let f = fs::File::create(path)?;
            Ok(Box::new(f))
        } else if self.clip {
            Ok(Box::new(ClipWriter::new()))
        } else {
            Ok(Box::new(io::stdout()))
        }
    }
}

struct ClipWriter {
    contents: Vec<u8>,
}

impl ClipWriter {
    pub fn new() -> Self {
        Self { contents: Vec::with_capacity(1024) }
    }
}

impl io::Write for ClipWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.contents.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for ClipWriter {
    fn drop(&mut self) {
        use copypasta_ext::prelude::*;
        use copypasta_ext::x11_fork::ClipboardContext;

        let goal = String::from_utf8_lossy(&self.contents[..]).to_string();
        let mut ctx = ClipboardContext::new().unwrap();
        ctx.set_contents(goal).unwrap();
    }
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
    let mut output = opt.output.get_writer()?;

    for line in input.lines() {
        let line = line?;

        for (idx, c) in line.chars().enumerate() {
            if idx % 2 == 0 {
                write!(output, "{}", c.to_uppercase())?;
            } else {
                write!(output, "{}", c.to_lowercase())?;
            }
        }
        println!();
    }

    Ok(())
}
