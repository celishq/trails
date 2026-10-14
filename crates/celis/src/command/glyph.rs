//! Big digits, bars, columns and strips, as the cards of stats and report draw them.

/// The three rows of each digit.
const DIGITS: [[&str; 3]; 10] = [
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

/// The blocks that fill an eighth more of a character, left to right.
const WIDE: [&str; 8] = [" ", "▏", "▎", "▍", "▌", "▋", "▊", "▉"];

/// The blocks that fill an eighth more of a row, bottom to top.
const TALL: [&str; 9] = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

/// The shade of each part of a strip, in turn.
const SHADES: [&str; 4] = ["█", "▓", "▒", "░"];

/// `text` three rows high: each digit big, a space between two, any other character on the last row.
pub(super) fn digits(text: &str) -> [String; 3] {
    let mut rows: [Vec<String>; 3] = Default::default();
    for c in text.chars() {
        for (r, row) in rows.iter_mut().enumerate() {
            row.push(match c.to_digit(10) {
                Some(d) => DIGITS[d as usize][r].to_string(),
                None if r == 2 => c.to_string(),
                None => " ".to_string(),
            });
        }
    }
    rows.map(|r| r.join(" "))
}

/// A bar `width` wide that fills the share of `n` to `most`, in eighths of a character.
pub(super) fn bar(n: usize, most: usize, width: usize) -> String {
    let eighths = (n * width * 8).checked_div(most).unwrap_or(0);
    let mut bar = "█".repeat(eighths / 8);
    if eighths / 8 < width {
        bar.push_str(WIDE[eighths % 8]);
    }
    format!("{bar:<width$}")
}

/// `rows` lines of columns, a value a column and a space between two, the largest value the full height,
/// in eighths of a row; the top line first.
pub(super) fn columns(values: &[usize], rows: usize) -> Vec<String> {
    let most = values.iter().copied().max().unwrap_or(0);
    let heights: Vec<usize> = values
        .iter()
        .map(|v| (v * rows * 8).checked_div(most).unwrap_or(0))
        .collect();
    (0..rows)
        .rev()
        .map(|r| {
            let line = heights.iter().map(|h| TALL[h.saturating_sub(r * 8).min(8)]);
            line.collect::<Vec<_>>().join(" ")
        })
        .collect()
}

/// A strip `width` wide that each of `parts` fills in its share, a shade a part; blank when all are
/// zero.
pub(super) fn strip(parts: &[usize], width: usize) -> String {
    let filled: String = shares(parts, width)
        .iter()
        .enumerate()
        .map(|(i, &n)| SHADES[i % SHADES.len()].repeat(n))
        .collect();
    format!("{filled:<width$}")
}

/// The characters of `width` each of `parts` takes, in its share, the largest remainders first.
fn shares(parts: &[usize], width: usize) -> Vec<usize> {
    let total: usize = parts.iter().sum();
    if total == 0 {
        return vec![0; parts.len()];
    }
    let mut shares: Vec<usize> = parts.iter().map(|p| p * width / total).collect();
    let mut order: Vec<usize> = (0..parts.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(parts[i] * width % total));
    let left = width - shares.iter().sum::<usize>();
    for &i in order.iter().take(left) {
        shares[i] += 1;
    }
    shares
}

#[cfg(test)]
mod tests;
