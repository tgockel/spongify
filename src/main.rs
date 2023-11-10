mod capital;

use capital::CapitalizationStrategy;
use clap::{Args, Parser};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    string::ToString,
};

type Result<T, E = Box<dyn std::error::Error + Send + Sync>> = std::result::Result<T, E>;

#[derive(Args, Debug)]
struct InputOpt {
    /// The meaning of the inline parameter is guessed at by SpOnGiFy. If this names a file, that file is read. If
    /// specified as `-`, SpOnGiFy will read from standard input. Use `--text` or `--file` to disambiguate.
    inline: Option<String>,

    /// The text to SpOnGiFy. This can be useful if your text is `-` or names a file.
    #[arg(long, group = "input")]
    text: Option<String>,

    /// Load text from a file.
    #[arg(long, short, group = "input")]
    file: Option<PathBuf>,

    /// Read from standard input.
    #[arg(long, group = "input")]
    stdin: bool,
}

enum InputSpec {
    Stdin,
    Text(String),
    File(PathBuf),
}

impl From<InputOpt> for InputSpec {
    fn from(value: InputOpt) -> Self {
        if value.stdin {
            Self::Stdin
        } else if let Some(text) = value.text {
            Self::Text(text)
        } else if let Some(file) = value.file {
            Self::File(file)
        } else if let Some(inline) = value.inline {
            if &inline[..] == "-" {
                Self::Stdin
            } else {
                let path = Path::new(&inline);
                if path.exists() {
                    Self::File(PathBuf::from(inline))
                } else {
                    Self::Text(inline)
                }
            }
        } else {
            unreachable!("Unhandled conversion from {value:?}");
        }
    }
}

impl InputSpec {
    pub fn into_reader(self) -> Result<Box<dyn io::BufRead + Send + Sync>> {
        match self {
            Self::Stdin => Ok(Box::new(io::BufReader::new(io::stdin()))),
            Self::Text(text) => Ok(Box::new(io::Cursor::new(text))),
            Self::File(path) => Ok(Box::new(io::BufReader::new(fs::File::open(path)?))),
        }
    }
}

#[derive(Args, Debug)]
struct OutputOpt {
    /// Output to a file.
    #[arg(short, long, group = "output")]
    output_file: Option<PathBuf>,

    /// Copy result to the clipboard.
    #[arg(short, long, group = "output")]
    clip: bool,
}

impl OutputOpt {
    /// # Return
    /// A tuple containing an output to write to and a boolean indicating if a newline should be appended to the output.
    pub fn get_writer(&self) -> Result<(Box<dyn io::Write>, bool)> {
        if let Some(ref path) = self.output_file {
            let f = fs::File::create(path)?;
            Ok((Box::new(f), true))
        } else if self.clip {
            Ok((Box::new(ClipWriter::new()), false))
        } else {
            Ok((Box::new(io::stdout()), true))
        }
    }
}

struct ClipWriter {
    contents: Vec<u8>,
}

impl ClipWriter {
    pub fn new() -> Self {
        Self {
            contents: Vec::with_capacity(1024),
        }
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

#[derive(Parser, Debug)]
struct Opt {
    #[command(flatten)]
    input: InputOpt,

    #[command(flatten)]
    output: OutputOpt,

    /// The capitalization style to use. Can be "LiKe tHiS", "LiKe ThIs", "lIkE ThIs", "lIkE tHiS", or "RaNDOmlY"
    /// (capitalization matters for everything but "raNdOMLy"). Is this an annoying way to specify an argument? Yes.
    #[arg(long, default_value_t = CapitalizationStrategy::AlternatingInitialUppercase)]
    style: CapitalizationStrategy,
}

impl fmt::Display for Opt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

fn main() -> Result<()> {
    use io::BufRead;

    let opt = Opt::parse();

    let input = InputSpec::from(opt.input).into_reader()?;
    let (mut output, newline) = opt.output.get_writer()?;
    let mut capitalizer = opt.style.create_engine();

    let mut first = true;
    for line in input.lines() {
        let line = line?;

        if !newline {
            if first {
                first = false;
            } else {
                write!(output, " ")?;
            }
        }

        for (idx, c) in line.chars().enumerate() {
            if capitalizer.should_capitalize(idx, c) {
                write!(output, "{}", c.to_uppercase())?;
            } else {
                write!(output, "{}", c.to_lowercase())?;
            }
        }

        if newline {
            writeln!(output)?;
        }
    }

    Ok(())
}
