use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::PathBuf;

use clap::Parser;
use csv::{ReaderBuilder, StringRecord};
use tabled::{Table, Tabled};
use rust_xlsxwriter::{Workbook, Worksheet, Format};

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

    /// Generate Excel report with summary, headers comparison, and data differences
    #[arg(long)]
    excel_output: Option<String>,
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

#[derive(Tabled, Clone)]
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

fn generate_excel_report(
    file1_path: &PathBuf,
    file2_path: &PathBuf,
    headers1: &[String],
    headers2: &[String],
    diffs: &[DiffRow],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut workbook = Workbook::new();
    
    // Create formats
    let header_format = Format::new().set_bold().set_background_color("CCCCCC");
    let title_format = Format::new().set_bold().set_font_size(14);
    
    // Sheet 1: General Summary
    let mut summary_sheet = workbook.add_worksheet();
    summary_sheet.set_name("Summary")?;
    
    create_summary_sheet(&mut summary_sheet, file1_path, file2_path, headers1, headers2, diffs, &title_format, &header_format)?;
    
    // Sheet 2: Headers Comparison  
    let mut headers_sheet = workbook.add_worksheet();
    headers_sheet.set_name("Headers Comparison")?;
    
    create_headers_sheet(&mut headers_sheet, headers1, headers2, &title_format, &header_format)?;
    
    // Sheet 3: Data Differences
    let mut data_sheet = workbook.add_worksheet();
    data_sheet.set_name("Data Differences")?;
    
    create_data_sheet(&mut data_sheet, diffs, &title_format, &header_format)?;
    
    workbook.save(output_path)?;
    println!("📄 Excel report generated: {}", output_path);
    
    Ok(())
}

fn create_summary_sheet(
    sheet: &mut Worksheet,
    file1_path: &PathBuf,
    file2_path: &PathBuf,
    headers1: &[String],
    headers2: &[String],
    diffs: &[DiffRow],
    title_format: &Format,
    header_format: &Format,
) -> Result<(), Box<dyn Error>> {
    let mut row = 0;
    
    // Title
    sheet.write_with_format(row, 0, "CSV Comparison Summary", title_format)?;
    row += 2;
    
    // File information
    sheet.write_with_format(row, 0, "File 1:", header_format)?;
    sheet.write(row, 1, file1_path.to_string_lossy())?;
    row += 1;
    
    sheet.write_with_format(row, 0, "File 2:", header_format)?;
    sheet.write(row, 1, file2_path.to_string_lossy())?;
    row += 2;
    
    // Statistics
    sheet.write_with_format(row, 0, "Comparison Statistics", header_format)?;
    row += 1;
    
    sheet.write(row, 0, "Total Differences:")?;
    sheet.write(row, 1, diffs.len() as f64)?;
    row += 1;
    
    sheet.write(row, 0, "File 1 Columns:")?;
    sheet.write(row, 1, headers1.len() as f64)?;
    row += 1;
    
    sheet.write(row, 0, "File 2 Columns:")?;
    sheet.write(row, 1, headers2.len() as f64)?;
    row += 1;
    
    sheet.write(row, 0, "Headers Match:")?;
    sheet.write(row, 1, if headers1 == headers2 { "Yes" } else { "No" })?;
    row += 2;
    
    // Difference breakdown
    let mut missing_in_file1 = 0;
    let mut missing_in_file2 = 0;
    let mut data_differences = 0;
    
    for diff in diffs {
        match diff.column.as_str() {
            "[missing in file1]" => missing_in_file1 += 1,
            "[missing in file2]" => missing_in_file2 += 1,
            _ => data_differences += 1,
        }
    }
    
    sheet.write_with_format(row, 0, "Difference Breakdown", header_format)?;
    row += 1;
    
    sheet.write(row, 0, "Data Differences:")?;
    sheet.write(row, 1, data_differences as f64)?;
    row += 1;
    
    sheet.write(row, 0, "Missing in File 1:")?;
    sheet.write(row, 1, missing_in_file1 as f64)?;
    row += 1;
    
    sheet.write(row, 0, "Missing in File 2:")?;
    sheet.write(row, 1, missing_in_file2 as f64)?;
    
    // Auto-fit columns
    sheet.set_column_width(0, 20)?;
    sheet.set_column_width(1, 40)?;
    
    Ok(())
}

fn create_headers_sheet(
    sheet: &mut Worksheet,
    headers1: &[String],
    headers2: &[String],
    title_format: &Format,
    header_format: &Format,
) -> Result<(), Box<dyn Error>> {
    let mut row = 0;
    
    // Title
    sheet.write_with_format(row, 0, "Headers Comparison", title_format)?;
    row += 2;
    
    // Create sets for comparison
    let set1: HashSet<&String> = headers1.iter().collect();
    let set2: HashSet<&String> = headers2.iter().collect();
    
    // Headers table
    sheet.write_with_format(row, 0, "Column Name", header_format)?;
    sheet.write_with_format(row, 1, "In File 1", header_format)?;
    sheet.write_with_format(row, 2, "In File 2", header_format)?;
    sheet.write_with_format(row, 3, "Status", header_format)?;
    row += 1;
    
    // Get all unique headers
    let all_headers: HashSet<&String> = set1.union(&set2).cloned().collect();
    let mut headers_vec: Vec<&String> = all_headers.into_iter().collect();
    headers_vec.sort();
    
    for header in headers_vec {
        let in_file1 = set1.contains(header);
        let in_file2 = set2.contains(header);
        
        sheet.write(row, 0, header)?;
        sheet.write(row, 1, if in_file1 { "Yes" } else { "No" })?;
        sheet.write(row, 2, if in_file2 { "Yes" } else { "No" })?;
        
        let status = match (in_file1, in_file2) {
            (true, true) => "Match",
            (true, false) => "Only in File 1",
            (false, true) => "Only in File 2",
            (false, false) => unreachable!(),
        };
        sheet.write(row, 3, status)?;
        row += 1;
    }
    
    // Auto-fit columns
    sheet.set_column_width(0, 25)?;
    sheet.set_column_width(1, 12)?;
    sheet.set_column_width(2, 12)?;
    sheet.set_column_width(3, 15)?;
    
    Ok(())
}

fn create_data_sheet(
    sheet: &mut Worksheet,
    diffs: &[DiffRow],
    title_format: &Format,
    header_format: &Format,
) -> Result<(), Box<dyn Error>> {
    let mut row = 0;
    
    // Title
    sheet.write_with_format(row, 0, "Data Differences", title_format)?;
    row += 2;
    
    // Headers
    sheet.write_with_format(row, 0, "Key", header_format)?;
    sheet.write_with_format(row, 1, "Column", header_format)?;
    sheet.write_with_format(row, 2, "File 1 Value", header_format)?;
    sheet.write_with_format(row, 3, "File 2 Value", header_format)?;
    row += 1;
    
    // Data rows
    for diff in diffs {
        sheet.write(row, 0, &diff.key)?;
        sheet.write(row, 1, &diff.column)?;
        sheet.write(row, 2, &diff.file1)?;
        sheet.write(row, 3, &diff.file2)?;
        row += 1;
    }
    
    // Auto-fit columns
    sheet.set_column_width(0, 30)?;
    sheet.set_column_width(1, 20)?;
    sheet.set_column_width(2, 30)?;
    sheet.set_column_width(3, 30)?;
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let (headers1, map1) = read_csv_to_map(args.file1.clone(), &args.key)?;
    let (headers2, map2) = read_csv_to_map(args.file2.clone(), &args.key)?;

    if headers1 != headers2 {
        eprintln!("Warning: header mismatch between files. Proceeding with column-name-based comparison.");
    }

    // Create column index mappings for both files
    let headers1_map: HashMap<String, usize> = headers1.iter().enumerate().map(|(i, h)| (h.clone(), i)).collect();
    let headers2_map: HashMap<String, usize> = headers2.iter().enumerate().map(|(i, h)| (h.clone(), i)).collect();

    let mut diffs = Vec::new();

    let all_keys: HashSet<_> = map1.keys().chain(map2.keys()).collect();

    for key in all_keys {
        match (map1.get(key), map2.get(key)) {
            (Some(r1), Some(r2)) => {
                // Get all unique column names from both files
                let all_columns: HashSet<String> = headers1.iter().chain(headers2.iter()).cloned().collect();
                
                for col_name in all_columns {
                    if args.key.contains(&col_name) || args.ignore.contains(&col_name) {
                        continue;
                    }

                    let v1 = headers1_map.get(&col_name).and_then(|&i| r1.get(i)).unwrap_or("");
                    let v2 = headers2_map.get(&col_name).and_then(|&i| r2.get(i)).unwrap_or("");
                    
                    // Handle cases where column exists in only one file
                    let (v1_display, v2_display) = match (headers1_map.contains_key(&col_name), headers2_map.contains_key(&col_name)) {
                        (true, true) => {
                            // Column exists in both files, compare values
                            if v1 != v2 {
                                (v1.to_string(), v2.to_string())
                            } else {
                                continue; // Values are the same, skip
                            }
                        },
                        (true, false) => {
                            // Column only exists in file1
                            (v1.to_string(), "[column not in file2]".to_string())
                        },
                        (false, true) => {
                            // Column only exists in file2
                            ("[column not in file1]".to_string(), v2.to_string())
                        },
                        (false, false) => unreachable!(), // Column came from one of the files
                    };

                    diffs.push(DiffRow {
                        key: key.clone(),
                        column: col_name.clone(),
                        file1: v1_display,
                        file2: v2_display,
                    });
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
        println!("{}", create_summary_table(diffs.clone(), args.max_rows, args.max_cell_width, args.no_truncate));
    }

    // Generate Excel report if requested
    if let Some(excel_path) = &args.excel_output {
        generate_excel_report(&args.file1, &args.file2, &headers1, &headers2, &diffs, excel_path)?;
    }

    Ok(())
}
