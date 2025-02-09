use clap::{arg, ArgAction, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "+\
")]
#[command(about = "File Utility Tool", version = "1.0", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List files and directories with sizes
    Tree(ListArgs),

    /// Search files containing specific pattern
    #[command(aliases = ["cat"])]
    Search(SearchArgs),

    /// Replace string in files
    Replace(ReplaceArgs),

    /// Count lines matching pattern
    Count(CountArgs),

    /// Rename file
    #[command(aliases = ["re"])]
    Rename(RenameArgs),

    /// List all files in current dic
    #[command(aliases = ["ls"])]
    List,

    /// Remove file or dic
    #[command(aliases = ["rm"])]
    Remove(RemoveArgs),

    /// Move file or dic
    #[command(aliases = ["mv"])]
    Move(MoveArgs),

    /// read table csv.
    #[command(aliases = ["csv-table"])]
    ReadTableCSV(ReadTableCsvArgs),

    /// Copy file or dic
    #[command(aliases = ["cp"])]
    Copy(CopyArgs),

    /// Decompress file  .zip
    #[command(aliases = ["dc-zip"])]
    DecompressZip(DecompressArgs),

    /// Decompress file  .gz
    #[command(aliases = ["dc-gz"])]
    DecompressGz(DecompressArgs),

    /// compress files zip
    CompressZip(CompressArgs),

    /// compress files .gz
    CompressGz(CompressArgsGz),

    /// Convert csv to json
    CsvToJson(CsvToJsonArgs),

    /// Convert json to csv
    JsonToCsv(JsonToCsvArgs),

    /// Convert json to csv
    #[command(aliases = ["read"])]
    ReadFile(CatFileArgs),

}

#[derive(Parser)]
pub struct ListArgs {
    pub(crate) path: PathBuf,
    #[arg(short, long)]
    pub(crate) depth: Option<usize>,
    #[arg(short = 'H', long)]
    pub(crate) human_readable: bool,
    #[arg(short, long)]
    pub(crate) color: bool,
}

#[derive(Parser)]
pub struct SearchArgs {
    pub(crate) path: String,
    pub(crate) pattern: String,
    #[arg(short = 'i', long, action = ArgAction::SetTrue, default_value_t = false)]
    pub(crate) case_insensitive: bool,
    #[arg(short = 'H', long, action = ArgAction::SetTrue, default_value_t = false)]
    pub(crate) hidden: bool,
    #[arg(short, long,action = ArgAction::SetTrue, default_value_t = true)]
    pub(crate) color: bool,
    // pub(crate) save_output: bool,
    // pub(crate) save_output_path: String,
}

#[derive(Parser)]
pub struct ReplaceArgs {
    pub(crate) path: PathBuf,
    pub(crate) old_string: String,
    pub(crate) new_string: String,
    #[arg(short = 'D', long)]
    pub(crate) dry_run: bool,
    #[arg(short = 'b', long)]
    pub(crate) backup: bool,
}

#[derive(Parser)]
pub struct CountArgs {
    pub(crate) path: PathBuf,
    pub(crate) pattern: String,
    #[arg(short = 'r', long)]
    pub(crate) regex: bool,
}

#[derive(Parser)]
pub struct RenameArgs {
    pub(crate) old_file_name: String,
    pub(crate) new_file_name: String,
}

#[derive(Parser)]
pub struct RemoveArgs {
    pub(crate) path: String,
    // #[arg(short = 'r', long)]
    pub(crate) option: String,
}

#[derive(Parser)]
pub struct ReadTableCsvArgs {
    pub(crate) path: PathBuf,
    #[arg(long, action = ArgAction::SetTrue, default_value_t = false)]
    pub(crate) start: bool,
    #[arg(default_value_t = 0)]
    pub(crate) row_num: usize,
    #[arg(long, action = ArgAction::SetTrue, default_value_t = false)]
    pub(crate) limit: bool,
    #[arg(default_value_t = 0)]
    pub(crate) limit_row: usize,
}

#[derive(Parser)]
pub struct CopyArgs {
    pub(crate) path_buf: PathBuf,
    pub(crate) copy_des: PathBuf,
    #[arg(short = 'r', long, action = ArgAction::SetTrue, default_value_t = false)]
    pub(crate) recursive: bool,
}

#[derive(Parser)]
pub struct MoveArgs {
    pub(crate) old_des: PathBuf,
    pub(crate) new_des: PathBuf,
    #[arg(short = 'r', long, action = ArgAction::SetTrue, default_value_t = false)]
    pub(crate) recursive: bool,
}

#[derive(Parser)]
pub struct DecompressArgs {
    pub(crate) path: PathBuf,
}

#[derive(Parser)]
pub struct CompressArgs {
    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    pub files: Vec<String>,
    pub output_compress: String,
}

#[derive(Parser)]
pub struct CompressArgsGz {
    pub output_compress: String,
    #[clap(short = 'f', long, value_parser, num_args = 1.., value_delimiter = ' ')]
    pub files: Vec<String>,
}

#[derive(Parser)]
pub struct CsvToJsonArgs {
    pub csv_path: PathBuf,
    pub output_json_path: PathBuf,
}

#[derive(Parser)]
pub struct JsonToCsvArgs {
    pub json_path: PathBuf,
    pub output_csv_path: PathBuf,
}

#[derive(Parser)]
pub struct CatFileArgs {
    pub path: PathBuf,
}



