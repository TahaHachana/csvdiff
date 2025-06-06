use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::PathBuf;

use clap::Parser;
use csv::{ReaderBuilder, StringRecord};
use tabled::{Table, Tabled};

/// Compare two CSV files based on key column(s), with options to ignore some columns.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// First CSV file path
    #[arg(long)]
    file1: PathBuf,

    /// Second CSV file path
    #[arg(long)]
    file2: PathBuf,

    /// Key columns (repeat for composite keys)
    #[arg(short, long)]
    key: Vec<String>,

    /// Columns to ignore when comparing
    #[arg(short = 'i', long)]
    ignore: Vec<String>,
}

fn read_csv_to_map(
    path: PathBuf,
    key_columns: &[String],
) -> Result<(Vec<String>, HashMap<String, StringRecord>), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().from_path(path)?;
    let headers = rdr.headers()?.clone();

    let key_indexes: Vec<usize> = key_columns
        .iter()
        .map(|key| {
            headers
                .iter()
                .position(|h| h == key)
                .ok_or_else(|| format!("Key column '{}' not found", key))
        })
        .collect::<Result<_, _>>()?;

    let mut map = HashMap::new();
    for result in rdr.records() {
        let record = result?;
        let key_parts: Vec<&str> = key_indexes
            .iter()
            .map(|&i| record.get(i).unwrap_or(""))
            .collect();
        let key = key_parts.join("|");
        map.insert(key, record);
    }

    Ok((headers.iter().map(|s| s.to_string()).collect(), map))
}

#[derive(Tabled)]
struct DiffRow {
    key: String,
    column: String,
    file1: String,
    file2: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let (headers1, map1) = read_csv_to_map(args.file1.clone(), &args.key)?;
    let (headers2, map2) = read_csv_to_map(args.file2.clone(), &args.key)?;

    if headers1 != headers2 {
        eprintln!("Warning: header mismatch between files. Proceeding with file1's headers.");
    }

    let mut diffs = Vec::new();

    let all_keys: HashSet<_> = map1.keys().chain(map2.keys()).collect();

    for key in all_keys {
        match (map1.get(key), map2.get(key)) {
            (Some(r1), Some(r2)) => {
                for (i, col_name) in headers1.iter().enumerate() {
                    if args.key.contains(col_name) || args.ignore.contains(col_name) {
                        continue;
                    }

                    let v1 = r1.get(i).unwrap_or("");
                    let v2 = r2.get(i).unwrap_or("");
                    if v1 != v2 {
                        diffs.push(DiffRow {
                            key: key.clone(),
                            column: col_name.clone(),
                            file1: v1.to_string(),
                            file2: v2.to_string(),
                        });
                    }
                }
            }
            (Some(r1), None) => {
                diffs.push(DiffRow {
                    key: key.clone(),
                    column: "[missing in file2]".into(),
                    file1: r1
                        .iter()
                        .collect::<Vec<_>>()
                        .join(",")
                        .chars()
                        .take(20)
                        .collect::<String>()
                        + "...",
                    file2: "".into(),
                });
            }
            (None, Some(r2)) => {
                diffs.push(DiffRow {
                    key: key.clone(),
                    column: "[missing in file1]".into(),
                    file1: "".into(),
                    file2: r2
                        .iter()
                        .collect::<Vec<_>>()
                        .join(",")
                        .chars()
                        .take(20)
                        .collect::<String>()
                        + "...",
                });
            }
            (None, None) => unreachable!(),
        }
    }

    if diffs.is_empty() {
        println!("✅ No differences found.");
    } else {
        println!("{}", Table::new(diffs));
    }

    Ok(())
}
