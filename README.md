# CSV Diff Tool

A command-line utility to compare two CSV files based on specified key columns and report the differences.

## Features

*   Compares two CSV files.
*   Identifies differences based on one or more key columns.
*   Allows ignoring specific columns during comparison.
*   Reports rows present in one file but not the other.
*   Reports cells with differing values for the same key.
*   Outputs differences in a tabular format.

## Usage

```bash
csvdiff --file1 <path_to_file1.csv> --file2 <path_to_file2.csv> --key <key_column_name> [OPTIONS]
```

### Options

*   `--file1 <PATH>`: Path to the first CSV file.
*   `--file2 <PATH>`: Path to the second CSV file.
*   `-k, --key <KEY_COLUMN>`: Specifies a key column. This option can be repeated for composite keys (e.g., `--key id --key name`). The order matters for composite keys.
*   `-i, --ignore <IGNORE_COLUMN>`: Specifies a column to ignore during comparison. This option can be repeated (e.g., `--ignore last_updated --ignore timestamp`).
*   `--help`: Prints help information.
*   `--version`: Prints version information.
