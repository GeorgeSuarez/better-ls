use chrono::{DateTime, Utc};
use clap::Parser;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use strum::Display;
use tabled::{
    Table, Tabled,
    settings::{
        Color, Style,
        object::{Columns, Rows},
    },
};

#[derive(Debug, Display, Serialize)]
enum EntryType {
    File,
    Dir,
}

#[derive(Debug, Tabled, Serialize)]
struct FileEntry {
    #[tabled{rename="Name"}]
    name: String,
    #[tabled{rename="Type"}]
    e_type: EntryType,
    #[tabled{rename="Permissions"}]
    permissions: String,
    #[tabled{rename="Size"}]
    len_bytes: String,
    #[tabled{rename="Date Modified"}]
    modified: String,
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = "Way better ls output", arg_required_else_help = true)]
struct CLI {
    path: Option<PathBuf>,
    #[arg(short, long)]
    json: bool,
}

fn main() {
    let cli = CLI::parse();

    let path = cli.path.unwrap_or(PathBuf::from("."));

    if let Ok(does_exist) = fs::exists(&path) {
        if does_exist {
            if cli.json {
                let get_files = get_files(&path);
                println!(
                    "{}",
                    serde_json::to_string(&get_files).unwrap_or("cannot parse json".to_string())
                );
            } else {
                print_table(path);
            }
        } else {
            println!("{}", "Path does not exist.".red());
        }
    } else {
        println!("{}", "Error reading directory".red());
    }
}

fn print_table(path: PathBuf) {
    let get_files = get_files(&path);
    let mut table = Table::new(get_files);
    table.with(Style::rounded());
    table.modify(Columns::first(), Color::FG_BRIGHT_CYAN);
    table.modify(Columns::one(2), Color::FG_BRIGHT_MAGENTA);
    table.modify(Columns::one(3), Color::FG_BRIGHT_YELLOW);
    table.modify(Rows::first(), Color::FG_BRIGHT_GREEN);
    println!("{}", table);
}

fn get_files(path: &Path) -> Vec<FileEntry> {
    let mut data = Vec::default();
    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir {
            if let Ok(file) = entry {
                map_data(file, &mut data);
            }
        }
    }
    data
}

fn map_data(file: fs::DirEntry, data: &mut Vec<FileEntry>) {
    if let Ok(meta) = fs::metadata(&file.path()) {
        data.push(FileEntry {
            name: file
                .file_name()
                .into_string()
                .unwrap_or("Unknown name".into()),
            e_type: if meta.is_dir() {
                EntryType::Dir
            } else {
                EntryType::File
            },
            permissions: format_permissions(meta.permissions().mode()),
            len_bytes: format_size(meta.len()),
            modified: if let Ok(modi) = meta.modified() {
                let date: DateTime<Utc> = modi.into();
                format!("{}", date.format("%a %b %e %Y"))
            } else {
                String::default()
            },
        });
    }
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    if bytes < 1024 {
        return format!("{}{}", bytes, UNITS[0]);
    }
    let mut size = bytes as f64;
    let mut idx = 0;
    while size >= 1024.0 && idx < UNITS.len() - 1 {
        size /= 1024.0;
        idx += 1;
    }
    format!("{:.1}{}", size, UNITS[idx])
}

fn format_permissions(mode: u32) -> String {
    let mut s = String::with_capacity(10);
    s.push(file_type_char(mode));
    let triples = [
        (mode & 0o400, mode & 0o200, mode & 0o100),
        (mode & 0o040, mode & 0o020, mode & 0o010),
        (mode & 0o004, mode & 0o002, mode & 0o001),
    ];
    for (r, w, x) in triples {
        s.push(if r != 0 { 'r' } else { '-' });
        s.push(if w != 0 { 'w' } else { '-' });
        s.push(if x != 0 { 'x' } else { '-' });
    }
    s
}

fn file_type_char(mode: u32) -> char {
    match mode & 0o170000 {
        0o040000 => 'd',
        0o120000 => 'l',
        0o020000 => 'c',
        0o060000 => 'b',
        0o010000 => 'p',
        0o140000 => 's',
        _ => '-',
    }
}
