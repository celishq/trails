//! Big digits, as stats and report draw them.

/// The three rows of each digit.
pub(crate) const DIGITS: [[&str; 3]; 10] = [
    ["█▀█", "█ █", "▀▀▀"],
    ["▀█ ", " █ ", "▀▀▀"],
    ["▀▀█", "█▀▀", "▀▀▀"],
    ["▀▀█", " ▀█", "▀▀▀"],
    ["█ █", "▀▀█", "  ▀"],
    ["█▀▀", "▀▀█", "▀▀▀"],
    ["█▀▀", "█▀█", "▀▀▀"],
    ["▀▀█", "  █", "  ▀"],
    ["█▀█", "█▀█", "▀▀▀"],
    ["█▀█", "▀▀█", "▀▀▀"],
];

/// `text` three rows high: each digit big with a space before it, any other character on the last row
/// with a space before it after a digit, or on the middle row when `text` has no digit.
pub(crate) fn digits(text: &str) -> [String; 3] {
    let low = match text.chars().any(|c| c.is_ascii_digit()) {
        true => 2,
        false => 1,
    };
    let mut rows: [String; 3] = Default::default();
    let mut after = false;
    for (i, c) in text.chars().enumerate() {
        let digit = c.to_digit(10);
        for (r, row) in rows.iter_mut().enumerate() {
            if i > 0 && (digit.is_some() || after) {
                row.push(' ');
            }
            match digit {
                Some(d) => row.push_str(DIGITS[d as usize][r]),
                None if r == low => row.push(c),
                None => row.push(' '),
            }
        }
        after = digit.is_some();
    }
    rows
}

#[cfg(test)]
mod tests;
