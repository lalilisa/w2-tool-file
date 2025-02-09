use crate::command::{CatFileArgs, CompressArgs, CompressArgsGz, CopyArgs, CountArgs, CsvToJsonArgs, DecompressArgs, JsonToCsvArgs, ListArgs, ReadTableCsvArgs, RemoveArgs, RenameArgs, ReplaceArgs, SearchArgs};
use anyhow::{anyhow, Context};
use csv::{ReaderBuilder, Writer};
use flate2::read::{GzDecoder, GzEncoder};
use flate2::Compression;
use ignore::WalkBuilder;
use regex::{Regex, RegexBuilder};
use serde_json::{json, Value};
use std::fs::{DirEntry, File, Metadata};
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io};
use tar::Builder;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub fn run_tree(args: &ListArgs) -> anyhow::Result<()> {
    let mut stdout = StandardStream::stdout(if args.color {
        ColorChoice::Always
    } else {
        ColorChoice::Never
    });

    let depth = args.depth.unwrap_or(usize::MAX);

    let walker = WalkBuilder::new(&args.path)
        .max_depth(Option::from(depth))
        .build();

    for result in walker {
        let entry = result?;
        let path = entry.path();
        let depth = entry.depth();

        let metadata = path.metadata()?;
        let size = if metadata.is_dir() {
            "DIR".to_string()
        } else {
            format_size(metadata.len(), args.human_readable)
        };

        let display_path = path.strip_prefix(&args.path)?.display().to_string();

        stdout.set_color(ColorSpec::new().set_fg(Some(if metadata.is_dir() {
            Color::Blue
        } else {
            Color::Green
        })))?;

        println!(
            "{:indent$}├─ {} ({})",
            "",
            display_path,
            size,
            indent = depth * 2
        );
    }

    stdout.reset()?;
    Ok(())
}

pub fn run_search(args: &SearchArgs) -> anyhow::Result<()> {
    let (search_dir, file_name_pattern) = extract_path_and_pattern(&args.path)?;

    println!("Search dir: {}", search_dir.display());
    println!("File pattern: {}", file_name_pattern);

    let file_regex = Regex::new(&file_name_pattern)
        .with_context(|| format!("Invalid regex pattern for filename: {}", file_name_pattern))?;

    let regex_search = RegexBuilder::new(&args.pattern)
        .case_insensitive(args.case_insensitive)
        .build()
        .with_context(|| format!("Invalid regex pattern: {}", args.pattern))?;

    let mut stdout = if args.color {
        StandardStream::stdout(ColorChoice::Always)
    } else {
        StandardStream::stdout(ColorChoice::Never)
    };

    let mut stdout_file_name = StandardStream::stdout(ColorChoice::Always);

    let mut binding = ColorSpec::new();

    let color_spec = binding.set_fg(Some(Color::Red)).set_bold(true);

    let walker = WalkBuilder::new(&search_dir)
        .hidden(args.hidden)
        .ignore(false)
        .build();

    for result in walker {
        let entry = result?;
        let path = entry.path();

        if path.is_dir() || path.is_symlink() {
            continue;
        }

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if !file_regex.is_match(file_name) {
                // println!("{} is not a file", path.display());
                continue;
            }
        }

        let file =
            File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;

        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(line) => line,
                Err(err) => match err.kind() {
                    _ => {
                        eprintln!("Error this: {} at line {}", err, line_num + 1);
                        continue;
                    }
                },
            };

            let mut last_match = 0;

            if regex_search.is_match(&line) {
                stdout_file_name.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
                write!(
                    &mut stdout_file_name,
                    "{}:{}: ",
                    path.display(),
                    line_num + 1
                )?;

                stdout_file_name.reset()?;
            }

            for mat in regex_search.find_iter(&line) {
                write!(&mut stdout, "{}", &line[last_match..mat.start()])?;

                if args.color {
                    stdout.set_color(&color_spec)?;
                }
                write!(&mut stdout, "{}", &line[mat.start()..mat.end()])?;
                if args.color {
                    stdout.reset()?;
                }

                last_match = mat.end();
            }

            if regex_search.is_match(&line) {
                writeln!(&mut stdout, "{}", &line[last_match..])?;
            }
        }
    }

    Ok(())
}

fn extract_path_and_pattern(input: &str) -> anyhow::Result<(PathBuf, String)> {
    if let Some((dir, pattern)) = input.rsplit_once('/') {
        let dir_path = PathBuf::from(dir);
        let file_name_pattern = wildcard_to_regex(pattern)?;
        Ok((dir_path, file_name_pattern))
    } else {
        anyhow::bail!("Invalid path pattern. Example: /var/logs/trc*");
    }
}

fn wildcard_to_regex(pattern: &str) -> anyhow::Result<String> {
    let regex_str = regex::escape(pattern)
        .replace("\\*", ".*")
        .replace("\\?", ".")
        .replace(r"\-", "-"); // Keep `-` unescaped so it behaves naturally

    Ok(format!("^{}$", regex_str))
}
pub fn run_replace(args: &ReplaceArgs) -> anyhow::Result<()> {
    let mut builder = WalkBuilder::new(&args.path);
    builder.hidden(true).ignore(false);

    for result in builder.build() {
        let entry = result?;
        let path = entry.path();

        if path.is_dir() || path.is_symlink() {
            continue;
        }

        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        if contents.contains(&args.old_string) {
            if args.backup {
                let backup_path = path.with_extension("bak");
                fs::copy(path, &backup_path)
                    .with_context(|| format!("Failed to create backup for {}", path.display()))?;
            }

            let new_contents = contents.replace(&args.old_string, &args.new_string);

            if args.dry_run {
                println!("Would replace '{}' in {}", args.old_string, path.display());
            } else {
                let mut file = File::create(path)
                    .with_context(|| format!("Failed to open {} for writing", path.display()))?;
                file.write_all(new_contents.as_bytes())?;

                println!("Replaced '{}' in {}", args.old_string, path.display());
            }
        }
    }

    Ok(())
}

pub fn run_count(args: &CountArgs) -> anyhow::Result<()> {
    let regex = if args.regex {
        Regex::new(&args.pattern)?
    } else {
        Regex::new(&regex::escape(&args.pattern))?
    };

    let mut builder = WalkBuilder::new(&args.path);
    builder.hidden(true).ignore(false);

    let mut total = 0;
    let mut file_counts = Vec::new();

    for result in builder.build() {
        let entry = result?;
        let path = entry.path();

        if path.is_dir() || path.is_symlink() {
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            if regex.is_match(&line) {
                count += 1;
            }
        }

        if count > 0 {
            file_counts.push((path.display().to_string(), count));
            total += count;
        }
    }

    for (file, count) in &file_counts {
        println!("{}: {}", file, count);
    }

    println!("Total matches across all files: {}", total);
    Ok(())
}

pub fn rename(args: &RenameArgs) -> anyhow::Result<()> {
    let path = Path::new(&args.old_file_name);

    match fs::rename(path, &args.new_file_name) {
        Ok(_) => {
            println!(
                "Successfully renamed '{}' to '{}'",
                &args.old_file_name, &args.new_file_name
            );
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                eprintln!("Error: Source file '{}' not found.", &args.old_file_name)
            }

            ErrorKind::AlreadyExists => {
                eprintln!(
                    "Error: Destination '{}' already exists.",
                    &args.new_file_name
                );
            }
            ErrorKind::PermissionDenied => {
                eprintln!(
                    "Error: Permission denied when renaming '{}'.",
                    &args.old_file_name
                );
            }
            _ => eprintln!(
                "Error:
                Failed to rename '{}': {}",
                &args.old_file_name, &args.new_file_name
            ),
        },
    }

    Ok(())
}

pub fn list_current_dir() -> anyhow::Result<()> {
    // Read the entries of the current directory
    let entries = fs::read_dir(".")?;

    // Print a header
    println!(
        "{:<10} {:<20} {:<8} {}",
        "Mode", "LastWriteTime", "Length", "Name"
    );
    println!("{:-<10} {:-<20} {:-<8} {:-<1}", "", "", "", "");

    for entry_result in entries {
        let entry = entry_result?;
        let md = entry.metadata()?;
        print_entry(&entry, &md)?;
    }

    Ok(())
}

fn print_entry(entry: &DirEntry, md: &Metadata) -> io::Result<()> {
    let file_type = md.file_type();
    let mode = if file_type.is_dir() {
        "d-----"
    } else {
        "-a----"
    };

    let datetime_str = match md.modified() {
        Ok(time) => format_system_time(time),
        Err(_) => String::from("Unknown"),
    };

    let length = if file_type.is_file() {
        md.len().to_string()
    } else {
        String::from("")
    };

    let name = entry.file_name().to_string_lossy().to_string();
    println!("{:<10} {:<20} {:<8} {}", mode, datetime_str, length, name);

    Ok(())
}

pub fn remove(args: &RemoveArgs) -> anyhow::Result<()> {
    let path = Path::new(&args.path);

    if !path.exists() {
        return Err(anyhow::anyhow!(
            "Path '{}' does not exist.",
            path.display()
        ));
    }

    if path.is_file() {
        match fs::remove_file(path) {
            Ok(_) => {
                println!("Remove '{}' success", path.display())
            }
            Err(e) => {
                return match e.kind() {
                    ErrorKind::PermissionDenied => Err(anyhow::anyhow!(
                        "Permission denied when remove '{}'.",
                        path.display()
                    )),
                    _ => Err(anyhow::anyhow!(
                        "Error when removing file '{}': {}",
                        path.display(),
                        e
                    )),
                }
            }
        };
        return Ok(());
    }

    if path.is_dir() {
        // only remove file
        if args.option == "-f" {
            remove_all_files_in_dir(path)
                .expect(format!("Failed to remove '{}'", path.display()).as_str());
        }

        // only remove folder
        if args.option == "-r" {
            remove_all_dir_in_dir(path)
                .expect(format!("Failed to remove '{}'", path.display()).as_str());
        }

        // remove file and folder
        if args.option == "-rf" {
            match fs::remove_dir_all(path) {
                Ok(_) => {
                    println!("Remove '{}' success", path.display())
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Error: Failed to remove '{}': {}",
                        path.display(),
                        e
                    ));
                }
            }
            return Ok(());
        }
    }

    Ok(())
}

pub fn copy(args: &CopyArgs) -> anyhow::Result<()> {
    let mut copy_path = PathBuf::new();

    if !args.path_buf.exists() {
        return Err(anyhow::anyhow!(
            "Error: Source file {:?} does not exist.",
            args.path_buf.display()
        ));
    }

    if args.path_buf.is_file() {
        if args.copy_des.is_file() {
            if args.copy_des.exists() {
                print!(
                    "File {:?} already exists. Overwrite? (y/n): ",
                    args.copy_des.display()
                );
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim().to_lowercase();

                if input != "y" {
                    return Err(anyhow::anyhow!("Error: File copy aborted."));
                }
            }

            copy_path = PathBuf::from(&args.copy_des);
        }

        if args.copy_des.is_dir() && args.copy_des.exists() {
            let copy_path = match args.path_buf.file_name() {
                Some(file_name) => args.copy_des.join(file_name),
                None => {
                    return Err(anyhow::anyhow!("Error: The source path has no file name."));
                }
            };

            if copy_path.exists() {
                print!(
                    "File {:?} already exists. Overwrite? (y/n): ",
                    args.copy_des.display()
                );
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim().to_lowercase();

                if input != "y" {
                    eprintln!("Error: File copy aborted.");
                    return Ok(());
                }
            }
        }

        fs::copy(&args.path_buf, &copy_path)?;
        println!(
            "Copied file: {:?} -> {:?}",
            args.path_buf.display(),
            args.copy_des.display()
        );
    } else {
        if args.recursive {
            if let Err(err) = copy_recursive(&args.path_buf, &args.copy_des) {
                return Err(anyhow::anyhow!("Error: copying folder: {}", err));
            }
        } else {
            return Err(anyhow::anyhow!(
                "Error: '{:?}' is a directory. Use '-r' to copy recursively.",
                args.path_buf
            ));
        }
    }
    Ok(())
}
// #[warn(dead_code)]
// pub fn move_file() {}

pub fn read_csv_table(args: ReadTableCsvArgs) -> anyhow::Result<()> {
    let csv_file = CsvFile {
        file_path: args.path,
        has_headers: true,
        skip: if args.start { args.row_num } else { 0 },
        limit: if args.limit { args.limit_row } else { 1000 },
    };

    println!("{:?}", args.limit);

    match csv_file.read_file() {
        Ok(_) => {
            println!("File successfully read.");
        }
        Err(e) => {
            if let Some(io_err) = e.downcast_ref::<io::Error>() {
                match io_err.kind() {
                    ErrorKind::NotFound => {
                        eprintln!("Error: File not found.");
                    }
                    ErrorKind::PermissionDenied => {
                        eprintln!("Error: Permission denied.");
                    }
                    other_kind => {
                        eprintln!("Error: Some other I/O error: {:?}", other_kind);
                    }
                }
            } else {
                eprintln!("Error: Some other error: {}", e);
            }
        }
    }
    Ok(())
}

pub fn decompress_zip(args: &DecompressArgs) -> anyhow::Result<()> {
    let path = &args.path;

    if !path.exists() {
        return Err(anyhow::anyhow!("Error: File {:?} does not exists.", path));
    }

    let file = File::open(path)?;

    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        let outpath = Path::new(".").join(file.name());
        println!("outpath: {:?}", outpath);
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    println!("Decompress {} successfully", path.display());
    Ok(())
}

pub fn decompress_gz(args: &DecompressArgs) -> anyhow::Result<()> {
    let path = &args.path;

    if !path.exists() {
        return Err(anyhow::anyhow!("Error: File {:?} does not exists.", path));
    }

    let file = File::open(path)?;

    let mut decoder = GzDecoder::new(file);
    let parent_path = path
        .parent()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "Path has no parent directory"))?;

    let mut out_file = File::create(parent_path)?;

    io::copy(&mut decoder, &mut out_file)?;
    Ok(())
}

pub fn compress_to_zip(args: &CompressArgs) -> anyhow::Result<()> {
    let paths = &args.files;
    let zip_file = File::create(&args.output_compress)?;
    let mut zip_writer = ZipWriter::new(zip_file);

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    for path in paths {
        let path_buf = PathBuf::from(path);

        if path_buf.is_dir() {
            for entry in WalkDir::new(&path_buf) {
                let entry = entry.map_err(|e| anyhow!("Error reading directory: {}", e))?;
                let entry_path = entry.path();

                if entry_path.is_dir() {
                    continue;
                }

                let relative_path = path_buf
                    .parent()
                    .map(|parent| entry_path.strip_prefix(parent));

                if let Some(relative_path) = relative_path {
                    let relative_path_str = relative_path?
                        .to_str()
                        .ok_or_else(|| anyhow!("Invalid UTF-8 path"))?;
                    add_file_to_zip(&mut zip_writer, entry_path, relative_path_str, &options)?;
                }
            }
        } else {
            let file_name = path_buf
                .file_name()
                .ok_or_else(|| anyhow!("Invalid file name"))?
                .to_str()
                .ok_or_else(|| anyhow!("Invalid UTF-8 file name"))?;

            add_file_to_zip(&mut zip_writer, &path_buf, file_name, &options)?;
        }
    }

    zip_writer.finish()?;
    println!("Compressed successfully to {}", args.output_compress);
    Ok(())
}

fn add_file_to_zip<W: Write + io::Seek>(
    zip_writer: &mut ZipWriter<W>,
    file_path: &Path,
    zip_entry_name: &str,
    options: &FileOptions,
) -> anyhow::Result<()> {
    let mut f = File::open(file_path).map_err(|err| match err.kind() {
        ErrorKind::NotFound => anyhow!("Error: File {} not found.", file_path.display()),
        ErrorKind::PermissionDenied => {
            anyhow!("Error: Permission denied: {}.", file_path.display())
        }
        other_kind => anyhow!("Error: Some other I/O error: {:?}", other_kind),
    })?;

    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)
        .map_err(|e| anyhow!("Error reading file {}: {}", file_path.display(), e))?;

    zip_writer.start_file(zip_entry_name, *options)?;
    zip_writer.write_all(&buffer)?;

    Ok(())
}

pub fn compress_to_tar_gz(args: &CompressArgsGz) -> anyhow::Result<()> {
    let paths = &args.files;

    let tar_gz = File::create(&args.output_compress)?;

    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar_builder = Builder::new(enc);

    for path in paths.iter() {
        tar_builder.append_path(path)?;
    }

    tar_builder.finish()?;
    println!("Compressed successfully to {}", args.output_compress);
    Ok(())
}

pub fn csv_to_json(args: &CsvToJsonArgs) -> anyhow::Result<()> {
    let csv_path = &args.csv_path;

    if !csv_path.exists() {
        return Err(anyhow::anyhow!(
            "File {:?} does not exists.",
            csv_path
        ));
    }

    if !csv_path.is_file() || !is_csv(csv_path) {
        return Err(anyhow::anyhow!("{:?} is not a csv file", csv_path));
    }

    let csv_file = CsvFile {
        file_path: csv_path.clone(),
        has_headers: true,
        skip: 0,
        limit: 0,
    };
    csv_file.parse_to_json(args.output_json_path.display().to_string())?;
    Ok(())
}

pub fn json_to_csv(args: &JsonToCsvArgs) -> anyhow::Result<()> {
    let json_path = &args.json_path;

    if !json_path.exists() {
        return Err(anyhow::anyhow!(
            "File {:?} does not exists.",
            json_path
        ));
    }

    if !json_path.is_file() || !is_json_file(json_path) {
        return Err(anyhow::anyhow!("{:?} is not a json file", json_path));
    }

    let json_file = JsonFile {
        file_path: json_path.clone(),
    };
    json_file.parse_to_csv(&args.output_csv_path)?;
    Ok(())
}
fn is_json_file(path: &PathBuf) -> bool {
    match path.extension() {
        Some(ext) if ext == "json" => true,
        _ => false,
    }
}

fn is_csv(path: &PathBuf) -> bool {
    match path.extension() {
        Some(ext) if ext == "csv" => true,
        _ => false,
    }
}

pub fn read_file(args: &CatFileArgs) -> anyhow::Result<()> {

    let path = &args.path;

    if !path.exists() {
        return Err(anyhow::anyhow!(
            "File {:?} does not exists.",
            path.display()
        ));
    }

    let file = File::open(path)?;

    let mut  buf_reader = BufReader::new(file);

    let mut buffer = [0; 1024];

    loop {
        let bytes_read = buf_reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        let chunk = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("{}", chunk);
    }

    Ok(())
}
fn format_system_time(time: SystemTime) -> String {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            use chrono::{Local, TimeZone};

            let secs = duration.as_secs() as i64;
            let nsecs = duration.subsec_nanos();
            let datetime = Local
                .timestamp_opt(secs, nsecs)
                .single()
                .expect("Parsing system time failed");
            datetime.format("%-m/%-d/%Y %-I:%M %p").to_string()
        }
        Err(_) => "Unknown".into(),
    }
}

pub fn format_size(size: u64, human_readable: bool) -> String {
    if human_readable {
        const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_idx = 0;

        while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_idx])
    } else {
        format!("{} B", size)
    }
}

fn remove_all_files_in_dir(path: &Path) -> anyhow::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_file() {
                fs::remove_file(&entry_path)?;
                println!("Removed file: {:?}", entry_path);
            } else if entry_path.is_dir() {
                remove_all_files_in_dir(&entry_path)?; // Recursively remove files in subdirectories
            }
        }
    }
    Ok(())
}
fn remove_all_dir_in_dir(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                fs::remove_dir(&entry_path)?;
                println!("Removed file: {:?}", entry_path);
            }
        }
    }
    Ok(())
}

fn copy_recursive(src: &PathBuf, dest: &PathBuf) -> anyhow::Result<()> {
    if !src.exists() {
        return Err(anyhow::anyhow!(
            "Source folder {:?} does not exist.",
            src
        ));
    }

    for entry in WalkDir::new(src) {
        let entry = entry?;
        let relative_path = entry.path().strip_prefix(src)?;

        let dest_path = dest.join(relative_path);

        if entry.path().is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    println!("Copied folder: {:?} -> {:?}", src, dest);
    Ok(())
}

pub struct TxtFile {
    pub file_path: String,
}
#[derive(Debug)]
pub struct CsvFile {
    pub file_path: PathBuf,
    pub has_headers: bool,
    pub skip: usize,
    pub limit: usize,
}

pub struct JsonFile {
    pub file_path: PathBuf,
}

impl JsonFile {
    pub fn parse_to_csv(&self, output_file_name: &PathBuf) -> anyhow::Result<()> {

        let json_file = File::open(&self.file_path)?;

        let json_data: Value = serde_json::from_reader(json_file)?;

        let mut csv_writer = Writer::from_path(output_file_name)?;

        match json_data {
            Value::Array(array) => {
                for (i, item) in array.iter().enumerate() {
                    if let Value::Object(obj) = item {
                        if i == 0 {
                            csv_writer.write_record(obj.keys())?;
                        }
                        csv_writer.write_record(obj.values().map(|v| v.to_string()))?;
                    }
                }
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid JSON format for CSV: Expected an array of objects"
                ))
            }
        }

        csv_writer.flush().context("Failed to flush CSV writer")?;

        Ok(())
    }
}
impl FileReader for TxtFile {
    fn read_file(&self) -> anyhow::Result<()> {
        let contents = fs::read_to_string(&self.file_path)?;
        println!("File read successfully!");
        println!("Contents of the file:\n");
        println!("{}", contents);
        Ok(())
    }
}

impl FileReader for JsonFile {
    fn read_file(&self) -> anyhow::Result<()> {
        let contents = fs::read_to_string(&self.file_path)?;
        let _parsed: Value = serde_json::from_str(&contents)?;
        println!("JSON File read and validated successfully!");
        println!("{}", contents);
        Ok(())
    }
}

impl CsvFile {
    pub fn parse_to_json(&self, output_file_name: String) -> anyhow::Result<()> {
        if !self.has_headers {
            eprintln!("Error: CSV file must have headers to be converted to JSON.");
            return Ok(());
        }

        let file = File::open(&self.file_path)?;

        let mut reader = ReaderBuilder::new()
            .has_headers(true) // Assuming the CSV file has headers
            .delimiter(b';')
            .from_reader(file);

        let headers = reader
            .headers()?
            .iter()
            .map(String::from)
            .collect::<Vec<String>>();

        let mut json_array = Vec::new();
        for result in reader.records() {
            let record = result?;
            let mut json_object = serde_json::Map::new();

            for (i, field) in record.iter().enumerate() {
                json_object.insert(headers[i].clone(), json!(field));
            }

            json_array.push(Value::Object(json_object));
        }

        // Convert to JSON string
        let json_output = serde_json::to_string_pretty(&json_array)?;

        let json_output_path = if output_file_name.ends_with(".json") {
            output_file_name.as_str()
        } else {
            &*format!("{}.json", output_file_name)
        };

        let mut output_file = File::create(json_output_path)?;

        output_file.write_all(json_output.to_string().as_bytes())?;

        println!(
            "CSV has been successfully converted to JSON. Output saved to {}.",
            output_file_name + ".json"
        );
        Ok(())
    }
}
impl FileReader for CsvFile {
    fn read_file(&self) -> anyhow::Result<()> {

        let file = File::open(&self.file_path)?;

        let mut reader = ReaderBuilder::new()
            .has_headers(self.has_headers)
            .delimiter(b';')
            .from_reader(file);

        println!("CSV File Contents:");

        if let Some(headers) = reader.headers().ok() {
            for header in headers.iter() {
                print!("{:<15}", header);
            }
            println!();
            println!("{}", "-".repeat(15 * headers.len()));
        }

        for result in reader.records().skip(self.skip).take(self.limit) {
            let record = result?;
            for field in record.iter() {
                print!("{:<15}", field);
            }
            println!();
        }

        Ok(())
    }
}

pub trait FileReader {
    fn read_file(&self) -> anyhow::Result<()>;
}
