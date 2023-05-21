use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use clap::{Args, Parser, Subcommand, ValueEnum};
use compressing::lzw;
use stat::Stat;

#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    /// The algorithm to use for compress or decompress.
    #[arg(short, value_enum)]
    algorithm: Algorithm,

    /// Whether the program should show statistics.
    #[arg(long)]
    stats: bool,

    #[command(subcommand)]
    action: Action,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Algorithm {
    Lzw,
}

#[derive(Debug, Subcommand)]
enum Action {
    Compress(ActionData),
    Decompress(ActionData),
}

#[derive(Debug, Args)]
struct ActionData {
    /// The file to compress or decompress.
    input: PathBuf,

    /// The output path.
    #[arg(short)]
    output: PathBuf,
}

fn main() -> io::Result<()> {
    let cmd = Cli::parse();

    let data = cmd.action.data();
    let manager = IoManager::new(&data.input, &data.output)?;

    let stats = match cmd.action {
        Action::Compress(_) => match cmd.algorithm {
            Algorithm::Lzw => manager.run(lzw::enc)?,
        },
        Action::Decompress(_) => match cmd.algorithm {
            Algorithm::Lzw => manager.run(lzw::dec)?,
        },
    };

    if cmd.stats {
        println!("done.");
        println!("    in {} ms", stats.elapsed.as_millis());

        if cmd.action.is_compress() {
            // https://en.wikipedia.org/wiki/Data_compression_ratio
            let space_saved = (1.0 - stats.written as f64 / stats.read as f64) * 100.0;
            println!("    saved {space_saved:.2}%");
        }
    }

    Ok(())
}

impl Action {
    fn data(&self) -> &ActionData {
        match self {
            Action::Compress(data) => data,
            Action::Decompress(data) => data,
        }
    }

    fn is_compress(&self) -> bool {
        matches!(self, Action::Compress(_))
    }
}

struct IoManager {
    reader: BufReader<Stat<File>>,
    writer: BufWriter<Stat<File>>,
}

impl IoManager {
    /// Opens the given files and constructs a new [`IoManager`].
    fn new(input: &Path, output: &Path) -> io::Result<Self> {
        let reader = {
            let file = File::open(input)?;
            let stat = Stat::new(file);
            BufReader::new(stat)
        };
        let writer = {
            let file = OpenOptions::new().create(true).write(true).open(output)?;
            let stat = Stat::new(file);
            BufWriter::new(stat)
        };
        Ok(Self { reader, writer })
    }

    /// Runs the provided function and collects statistics on the involved I/O
    /// operations.
    fn run<F>(mut self, f: F) -> io::Result<Stats>
    where
        F: Fn(&mut dyn Read, &mut dyn Write) -> io::Result<()>,
    {
        let start = Instant::now();
        f(&mut self.reader, &mut self.writer)?;
        let elapsed = start.elapsed();

        let stat_r = self.reader.into_inner();
        let stat_w = self.writer.into_inner()?;

        Ok(Stats {
            read: stat_r.read_count(),
            written: stat_w.write_count(),
            elapsed,
        })
    }
}

struct Stats {
    read: u64,
    written: u64,
    elapsed: Duration,
}
