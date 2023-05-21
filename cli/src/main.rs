use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter},
    path::{Path, PathBuf},
};

use clap::{Args, Parser, Subcommand, ValueEnum};
use compressing::lzw;

#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    /// The algorithm to use for compress or decompress.
    #[arg(short, value_enum)]
    algorithm: Algorithm,

    #[command(subcommand)]
    action: Action,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Algorithm {
    Lzw,
}

#[derive(Debug, Subcommand)]
enum Action {
    Compress(ActionInput),
    Decompress(ActionInput),
}

#[derive(Debug, Args)]
struct ActionInput {
    /// The file to compress or decompress.
    input: PathBuf,

    /// The output path.
    #[arg(short)]
    output: PathBuf,
}

fn main() -> io::Result<()> {
    let cmd = Cli::parse();

    match cmd.action {
        Action::Compress(data) => {
            let (mut reader, mut writer) = rw_pair(&data.input, &data.output)?;
            match cmd.algorithm {
                Algorithm::Lzw => {
                    lzw::enc(&mut reader, &mut writer)?;
                }
            }
            println!("done");
        }
        Action::Decompress(data) => {
            let (mut reader, mut writer) = rw_pair(&data.input, &data.output)?;
            match cmd.algorithm {
                Algorithm::Lzw => {
                    lzw::dec(&mut reader, &mut writer)?;
                }
            }
            println!("done");
        }
    }

    Ok(())
}

fn rw_pair(input: &Path, output: &Path) -> io::Result<(BufReader<File>, BufWriter<File>)> {
    let in_file = File::open(input)?;
    let reader = BufReader::new(in_file);

    let out_file = OpenOptions::new().create(true).write(true).open(output)?;
    let writer = BufWriter::new(out_file);

    Ok((reader, writer))
}
