use crate::app::App;
use chrono::{Datelike, NaiveDate};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn render(frame: &mut Frame, app: &App) {
    const MIN_WIDTH: u16 = 30;
    const MIN_HEIGHT: u16 = 12;

    let area = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        let error_message = vec![
            Line::from(Span::styled(
                "Terminal too small!",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("Minimum size: {}x{}", MIN_WIDTH, MIN_HEIGHT),
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                format!("Current size: {}x{}", area.width, area.height),
                Style::default().fg(Color::White),
            )),
        ];

        let paragraph = Paragraph::new(error_message)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
        return;
    }

    let Some(calendar) = &app.calendar else {
        return;
    };
    let weeks = &calendar.weeks;
    if weeks.is_empty() {
        return;
    }

    let available_width = area.width.saturating_sub(2) as usize;
    let left_label_width = 5;
    let week_width = 3;
    let max_weeks = (available_width - left_label_width) / week_width;

    let start_week_idx = weeks.len().saturating_sub(max_weeks);
    let displayed_weeks = &weeks[start_week_idx..];

    let calendar_width = (left_label_width + displayed_weeks.len() * week_width) as u16;
    let calendar_height = 10;

    let calendar_area = Rect {
        x: area.x + (area.width.saturating_sub(calendar_width + 2)) / 2,
        y: area.y + (area.height.saturating_sub(calendar_height + 2)) / 2,
        width: calendar_width + 2,
        height: calendar_height + 2,
    };

    let block = Block::default()
        .title(format!("trexanh - @{}", app.config.username))
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let inner = block.inner(calendar_area);
    frame.render_widget(block, calendar_area);

    let mut lines: Vec<Line> = vec![];

    lines.push(Line::from(vec![
        Span::raw("     "),
        Span::styled(
            get_month_labels(displayed_weeks),
            Style::default().fg(Color::White),
        ),
    ]));

    let left_labels = ["Mon", "Wed", "Fri"];
    let left_label_positions = [1, 3, 5];

    for day_idx in 0..7 {
        let mut line_spans = vec![];

        if left_label_positions.contains(&day_idx) {
            let label = left_labels[left_label_positions
                .iter()
                .position(|&x| x == day_idx)
                .unwrap()];
            line_spans.push(Span::styled(
                format!(" {:>3} ", label),
                Style::default().fg(Color::White),
            ));
        } else {
            line_spans.push(Span::raw("     "));
        }

        for (week_idx, week) in displayed_weeks.iter().enumerate() {
            if let Some(day) = week.contribution_days.get(day_idx) {
                let color = get_contribution_color(day.contribution_count);
                line_spans.push(Span::styled("██", Style::default().fg(color)));
            } else {
                line_spans.push(Span::raw("  "));
            }

            if week_idx < displayed_weeks.len() - 1 {
                line_spans.push(Span::raw(" "));
            }
        }

        lines.push(Line::from(line_spans));
    }

    lines.push(Line::from(""));

    let legend_levels = [0, 1, 4, 7, 11];

    let mut legend_spans = vec![Span::raw("  Less ")];

    for &level in &legend_levels {
        legend_spans.push(Span::styled(
            "██",
            Style::default().fg(get_contribution_color(level)),
        ));
        legend_spans.push(Span::raw(" "));
    }

    legend_spans.push(Span::raw("More"));

    lines.push(Line::from(legend_spans));

    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, inner);
}

fn get_contribution_color(count: u32) -> Color {
    match count {
        0 => Color::Rgb(55, 55, 55),
        1..=3 => Color::Rgb(90, 140, 120),
        4..=6 => Color::Rgb(120, 180, 130),
        7..=10 => Color::Rgb(160, 210, 150),
        _ => Color::Rgb(220, 240, 170),
    }
}

fn get_month_labels(weeks: &[crate::models::Week]) -> String {
    let mut month_label = Vec::new();
    let mut last_month: Option<(i32, u32)> = None;

    for (week_idx, week) in weeks.iter().enumerate() {
        if let Some(first_day) = week.contribution_days.first()
            && let Ok(date) = first_day.date.parse::<NaiveDate>()
        {
            let current_month = (date.year(), date.month());

            if last_month != Some(current_month) && date.day() <= 7 {
                let month = date.format("%b").to_string();
                month_label.push((week_idx, month));
                last_month = Some(current_month);
            }
        }
    }

    let mut label = String::new();

    for (week_pos, month) in month_label.iter() {
        let target_pos = week_pos * 3;
        let current_len = label.len();

        if target_pos > current_len {
            label.push_str(&" ".repeat(target_pos - current_len));
        }

        label.push_str(month);
    }

    label
}
