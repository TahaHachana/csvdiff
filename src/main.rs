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

    /// Maximum number of rows to display (default: 20)
    #[arg(long, default_value = "20")]
    max_rows: usize,

    /// Maximum width for cell content (default: 30)
    #[arg(long, default_value = "30")]
    max_cell_width: usize,

    /// Show all differences without truncation
    #[arg(long, default_value = "false")]
    no_truncate: bool,
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

fn truncate_string(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else {
        format!("{}...", &s[..max_width.saturating_sub(3)])
    }
}

fn create_summary_table(diffs: Vec<DiffRow>, max_rows: usize, max_cell_width: usize, no_truncate: bool) -> String {
    if no_truncate {
        return Table::new(diffs).to_string();
    }

    let total_diffs = diffs.len();
    
    if total_diffs == 0 {
        return "✅ No differences found.".to_string();
    }

    // Truncate cell content
    let mut truncated_diffs: Vec<DiffRow> = diffs
        .into_iter()
        .map(|diff| DiffRow {
            key: truncate_string(&diff.key, max_cell_width),
            column: truncate_string(&diff.column, max_cell_width),
            file1: truncate_string(&diff.file1, max_cell_width),
            file2: truncate_string(&diff.file2, max_cell_width),
        })
        .collect();

    // Handle row truncation
    let mut result = String::new();
    
    if total_diffs <= max_rows {
        result.push_str(&Table::new(truncated_diffs).to_string());
    } else {
        // Take first half and last few rows, with separator in between
        let head_rows = max_rows / 2;
        let tail_rows = max_rows - head_rows - 1; // -1 for the separator row
        
        let mut display_rows = Vec::new();
        
        // Add head rows
        display_rows.extend(truncated_diffs.drain(..head_rows));
        
        // Add separator row
        display_rows.push(DiffRow {
            key: "...".to_string(),
            column: format!("... ({} more rows) ...", total_diffs - max_rows),
            file1: "...".to_string(),
            file2: "...".to_string(),
        });
        
        // Add tail rows
        if tail_rows > 0 && truncated_diffs.len() >= tail_rows {
            let start_index = truncated_diffs.len() - tail_rows;
            display_rows.extend(truncated_diffs.drain(start_index..));
        }
        
        result.push_str(&Table::new(display_rows).to_string());
    }
    
    // Add summary information
    if total_diffs > max_rows {
        result.push_str(&format!("\n\n📊 Summary: {} total differences found", total_diffs));
        result.push_str(&format!("\n   Showing {} rows (use --max-rows to adjust or --no-truncate to show all)", max_rows));
    } else {
        result.push_str(&format!("\n\n📊 Total differences: {}", total_diffs));
    }
    
    result
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
                let preview = r1
                    .iter()
                    .collect::<Vec<_>>()
                    .join(",")
                    .chars()
                    .take(50)
                    .collect::<String>();
                let preview = if preview.len() >= 50 { 
                    format!("{}...", &preview[..47]) 
                } else { 
                    preview 
                };
                
                diffs.push(DiffRow {
                    key: key.clone(),
                    column: "[missing in file2]".into(),
                    file1: preview,
                    file2: "".into(),
                });
            }
            (None, Some(r2)) => {
                let preview = r2
                    .iter()
                    .collect::<Vec<_>>()
                    .join(",")
                    .chars()
                    .take(50)
                    .collect::<String>();
                let preview = if preview.len() >= 50 { 
                    format!("{}...", &preview[..47]) 
                } else { 
                    preview 
                };
                
                diffs.push(DiffRow {
                    key: key.clone(),
                    column: "[missing in file1]".into(),
                    file1: "".into(),
                    file2: preview,
                });
            }
            (None, None) => unreachable!(),
        }
    }

    if diffs.is_empty() {
        println!("✅ No differences found.");
    } else {
        println!("{}", create_summary_table(diffs, args.max_rows, args.max_cell_width, args.no_truncate));
    }

    Ok(())
}
