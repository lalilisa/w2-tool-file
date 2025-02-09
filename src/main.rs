mod command;
mod file_handler;
mod test;

use crate::command::{Cli, Commands};
use anyhow::Result;
use clap::Parser;
use file_handler as FileHandler;
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Tree(args) => FileHandler::run_tree(&args),
        Commands::Search(args) => FileHandler::run_search(&args),
        Commands::Replace(args) => FileHandler::run_replace(&args),
        Commands::Count(args) => FileHandler::run_count(&args),
        Commands::Rename(args) => FileHandler::rename(&args),
        Commands::List => FileHandler::list_current_dir(),
        Commands::ReadTableCSV(args) => FileHandler::read_csv_table(args),
        Commands::Copy(args) => FileHandler::copy(&args),
        Commands::DecompressZip(args) => FileHandler::decompress_zip(&args),
        Commands::DecompressGz(args) => FileHandler::decompress_gz(&args),
        Commands::CompressZip(args) => FileHandler::compress_to_zip(&args),
        Commands::CompressGz(args) => FileHandler::compress_to_tar_gz(&args),
        Commands::Remove(args) => FileHandler::remove(&args),
        Commands::CsvToJson(args) => FileHandler::csv_to_json(&args),
        Commands::JsonToCsv(args) => FileHandler::json_to_csv(&args),
        Commands::ReadFile(args )=> FileHandler::read_file(&args),
        _ => Ok(()),
    }
}


