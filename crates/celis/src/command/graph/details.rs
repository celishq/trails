//! The details of one step, as the node view opens them over its graph.

use crate::terminal::canvas::{BOLD, DIM, GREEN, RED, YELLOW};
use crate::terminal::look::look;
use celis_trails::{Mark, State, Step};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Widget, Wrap};

/// Draws the details of `step` as it stands, `state`, over `area` of `buf`: its whole text, where it
/// works and its check, how many of its steps are done, and its proof or why it was dropped.
pub(super) fn details(step: Step, state: State, area: Rect, buf: &mut Buffer) {
    let area = Rect::new(
        area.x + 2,
        area.y + 1,
        area.width.saturating_sub(4),
        area.height.saturating_sub(1),
    );
    let (dim, bold) = (DIM, BOLD);
    let field = |name: &str, value: String, style: Style| {
        Line::from(vec![
            Span::styled(format!("{name:<8}"), dim),
            Span::styled(value, style),
        ])
    };
    let record = step.record();
    let mut lines: Vec<Line> = record
        .content
        .lines()
        .enumerate()
        .map(|(n, line)| match n {
            0 => Line::styled(line.to_string(), bold),
            _ => Line::raw(line.to_string()),
        })
        .collect();
    lines.push(Line::raw(""));
    lines.push(field("state", state.text().to_string(), look(state).text));
    if !record.places.is_empty() {
        lines.push(field("at", record.places.join(", "), Style::new()));
    }
    if let Some(check) = &record.check {
        lines.push(field("check", check.clone(), Style::new()));
    }
    let under = step.tree().len() - 1;
    if under > 0 {
        let done = step.tree()[1..]
            .iter()
            .filter(|s| s.state() == State::Done)
            .count();
        lines.push(field("steps", format!("{done}/{under} done"), Style::new()));
    }
    match step.close().map(|(_, mark)| mark) {
        Some(Mark::Done { proof, changed }) if state == State::Done => {
            lines.push(field("proof", proof, GREEN));
            if let Some(changed) = changed {
                lines.push(field("changed", changed, YELLOW));
            }
        }
        Some(Mark::Drop { reason }) if state == State::Dropped => {
            lines.push(field("why", reason, RED));
        }
        _ => {}
    }
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new())
        .title(format!(" {} ", &step.id()[..8]))
        .title_bottom(Line::styled(" enter esc close ", dim).right_aligned());
    Clear.render(area, buf);
    Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .render(area, buf);
}
