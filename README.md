# CSV Diff Tool

A command-line utility to compare two CSV files based on specified key columns and report the differences.

## Features

*   **Flexible Comparison**: Compares two CSV files using single or composite key columns
*   **Smart Truncation**: Automatically truncates large outputs (similar to Polars DataFrames) for better readability
*   **Excel Report Generation**: Creates comprehensive Excel reports with summary, headers comparison, and data differences
*   **Column Filtering**: Allows ignoring specific columns during comparison
*   **Missing Row Detection**: Reports rows present in one file but not the other
*   **Cell-Level Differences**: Reports cells with differing values for the same key
*   **Configurable Output**: Control table size with customizable row and cell width limits
*   **Summary Statistics**: Provides clear summaries with total difference counts

## Usage

```bash
csvdiff --file1 <path_to_file1.csv> --file2 <path_to_file2.csv> --key <key_column_name> [OPTIONS]
```

### Options

*   `--file1 <PATH>`: Path to the first CSV file
*   `--file2 <PATH>`: Path to the second CSV file
*   `-k, --key <KEY_COLUMN>`: Specifies a key column. Can be repeated for composite keys (e.g., `--key id --key name`)
*   `-i, --ignore <IGNORE_COLUMN>`: Specifies a column to ignore during comparison. Can be repeated
*   `--max-rows <NUMBER>`: Maximum number of rows to display (default: 20)
*   `--max-cell-width <NUMBER>`: Maximum width for cell content (default: 30)
*   `--no-truncate`: Show all differences without truncation
*   `--excel-output <PATH>`: Generate Excel report with summary, headers comparison, and data differences
*   `--help`: Prints help information
*   `--version`: Prints version information

## Examples

### Basic Comparison
```bash
# Compare two files using a single key column
csvdiff --file1 products_old.csv --file2 products_new.csv --key product_id
```

### Composite Key Comparison
```bash
# Use multiple columns as a composite key
csvdiff --file1 inventory.csv --file2 updated_inventory.csv --key sku --key size --key color
```

### Ignoring Columns
```bash
# Ignore timestamp and description columns during comparison
csvdiff --file1 data1.csv --file2 data2.csv --key id --ignore timestamp --ignore description
```

### Controlling Output Size
```bash
# Show only 10 rows with cell content limited to 20 characters
csvdiff --file1 large_file1.csv --file2 large_file2.csv --key id --max-rows 10 --max-cell-width 20

# Show all differences without any truncation
csvdiff --file1 file1.csv --file2 file2.csv --key id --no-truncate
```

### Large Dataset Example
```bash
# Compare large CSV files with smart truncation (recommended for files with thousands of rows)
csvdiff --file1 dataset_v1.csv --file2 dataset_v2.csv --key sku --key size --key colour --max-rows 15
```

### Excel Report Generation
```bash
# Generate a comprehensive Excel report with three sheets
csvdiff --file1 data1.csv --file2 data2.csv --key id --excel-output comparison_report.xlsx

# Combine with other options for customized analysis
csvdiff --file1 large_file1.csv --file2 large_file2.csv --key sku --key size --ignore timestamp --excel-output detailed_report.xlsx
```

## Output Format

The tool displays differences in a clear tabular format:

```
+--------------------------------+----------------------------+--------------------------------+------------------------------+
| key                            | column                     | file1                          | file2                        |
+--------------------------------+----------------------------+--------------------------------+------------------------------+
| PROD001|M|Blue                 | price                      | 19.99                          | 24.99                        |
+--------------------------------+----------------------------+--------------------------------+------------------------------+
| PROD002|L|Red                  | availability               | in_stock                       | out_of_stock                 |
+--------------------------------+----------------------------+--------------------------------+------------------------------+
| PROD003|S|Green                | [missing in file2]         | Complete product data...       |                              |
+--------------------------------+----------------------------+--------------------------------+------------------------------+
| ...                            | ... (1,247 more rows) ... | ...                            | ...                          |
+--------------------------------+----------------------------+--------------------------------+------------------------------+

📊 Summary: 1,250 total differences found
   Showing 20 rows (use --max-rows to adjust or --no-truncate to show all)
```

## Excel Reports

When using `--excel-output`, the tool generates a comprehensive Excel workbook with three sheets:

### 📋 Sheet 1: Summary
- File paths and comparison metadata
- Total difference counts and statistics
- Header compatibility analysis
- Breakdown by difference type (data changes vs missing rows)

### 📊 Sheet 2: Headers Comparison
- Side-by-side comparison of all column headers
- Identification of columns unique to each file
- Clear status indicators (Match, Only in File 1, Only in File 2)

### 📈 Sheet 3: Data Differences
- Complete list of all differences (no truncation)
- Organized by key, column, and values from both files
- Proper Excel formatting with headers and auto-sized columns
- Suitable for further analysis, filtering, and sharing

**Example Excel Output:**
```bash
csvdiff --file1 products.csv --file2 updated_products.csv --key sku --excel-output product_changes.xlsx
# Generates: product_changes.xlsx with professional formatting
```

## Performance

The tool is optimized for large datasets:
- ✅ Handles CSV files with tens of thousands of rows
- ✅ Smart memory usage with streaming CSV processing
- ✅ Polars-style truncation prevents terminal overflow
- ✅ Configurable output limits for different use cases
- ✅ Efficient Excel generation for comprehensive reporting
- ✅ Tested with 45,000+ differences in production datasets

## Installation

### From Source
```bash
git clone https://github.com/TahaHachana/csvdiff.git
cd csvdiff
cargo build --release
```

The binary will be available at `target/release/csvdiff`.

### Prerequisites
- Rust 1.70 or later

## Use Cases

- **Data Migration Validation**: Compare datasets before and after migration
- **API Response Comparison**: Validate data consistency across different API versions
- **ETL Pipeline Testing**: Ensure data transformations preserve accuracy
- **Database Synchronization**: Check differences between database exports
- **Quality Assurance**: Verify data integrity after processing operations
- **Business Reporting**: Generate professional Excel reports for stakeholders
- **Audit Trails**: Document data changes with comprehensive Excel documentation
- **Data Science Workflows**: Validate model training datasets and feature engineering

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT OR Apache-2.0 license.
