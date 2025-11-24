use chrono::Local;
use std::io::{self, Write};

/// Helper to format color codes like "1;32" from vec![1, 32]
fn format_color_codes(codes: &[i32]) -> String {
    codes
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<String>>()
        .join(";")
}

/// Most common usage: print a single string with a color
/// Example: printc("Hello", &[1, 32]);
pub fn printc(text: &str, color: &[i32]) {
    printc_full(&[text], &[color.to_vec()], " ", "", "\n", true);
}

/// Print multiple strings with a single color
/// Example: printc_multi(&["Hello", "World"], &[1, 32]);
pub fn printc_multi(strings: &[&str], color: &[i32]) {
    printc_full(strings, &[color.to_vec()], " ", "", "\n", true);
}

/// Print multiple strings with different colors for each
/// Example: printc_colors(&["Status:", "OK"], &[vec![1, 33], vec![1, 32]]);
pub fn printc_colors(strings: &[&str], colors: &[Vec<i32>]) {
    printc_full(strings, colors, " ", "", "\n", true);
}

/// Print without timestamp
/// Example: printc_no_time("Hello", &[1, 32]);
pub fn printc_no_time(text: &str, color: &[i32]) {
    printc_full(&[text], &[color.to_vec()], " ", "", "\n", false);
}

/// Print with custom end character (like Python's end='')
/// Example: printc_end("Hello", &[1, 32], "\r");
pub fn printc_end(text: &str, color: &[i32], end: &str) {
    printc_full(&[text], &[color.to_vec()], " ", "", end, true);
}

/// Full-featured printc with all options (internal implementation)
///
/// # Arguments
/// * `strings` - Slice of strings to print
/// * `colors` - Color codes for each string (can be single color or one per string)
/// * `sep` - Separator between strings
/// * `start` - String to print at the beginning
/// * `end` - String to print at the end
/// * `show_time` - Whether to show [HH:MM:SS] timestamp
pub fn printc_full(
    strings: &[&str],
    colors: &[Vec<i32>],
    sep: &str,
    start: &str,
    end: &str,
    show_time: bool,
) {
    print!("{}", start);

    // Print timestamp if requested
    if show_time {
        let now = Local::now();
        print!("\x1b[1;30m[{}]\x1b[0m ", now.format("%H:%M:%S"));
    }

    // Print each string with its color
    for (i, string) in strings.iter().enumerate() {
        if i > 0 {
            print!("{}", sep);
        }

        // Determine which color to use
        let color_codes = if colors.is_empty() {
            // No color
            String::new()
        } else if colors.len() == 1 {
            // Single color for all strings
            format_color_codes(&colors[0])
        } else {
            // Different color for each string
            if i < colors.len() {
                format_color_codes(&colors[i])
            } else {
                String::new()
            }
        };

        if color_codes.is_empty() {
            print!("{}", string);
        } else {
            print!("\x1b[{}m{}\x1b[0m", color_codes, string);
        }
    }

    print!("{}", end);
    io::stdout().flush().unwrap();
}

pub fn back(n: usize) {
    // Move cursor up n lines
    print!("\r\x1b[{}A", n);
    io::stdout().flush().unwrap();
}

pub fn next(n: usize) {
    // Print n newlines
    print!("{}", "\n".repeat(n)); // Repeat "\n" n times
    io::stdout().flush().unwrap();
}

pub fn clear() {
    // Clear current line
    print!("\r\x1b[K\r");
    io::stdout().flush().unwrap();
}

pub fn save() {
    print!("\r\x1b[s");
    io::stdout().flush().unwrap();
}

pub fn recover() {
    print!("\r\x1b[u");
    io::stdout().flush().unwrap();
}

/// Scale a number to human-readable format with units
///
/// # Arguments
/// * `n` - Number to scale
/// * `size` - Divisor (typically 1024 for bytes)
/// * `digit` - Number of decimal places
///
/// # Returns
/// Formatted string like "1.23 MB"
pub fn scale(n: i64, size: f64, digit: usize) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

    let mut count = 0;
    let mut value = n as f64;

    // Recursive scaling logic converted to loop
    while value > size && count < units.len() - 1 {
        value /= size;
        count += 1;
    }

    // Format with specified decimal places
    format!("{:.width$}{}", value, units[count], width = digit)
}
