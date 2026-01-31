use crate::app::{App, Focus};
use chrono::{Datelike, NaiveDate};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

fn render_error(frame: &mut Frame, area: Rect, min_width: u16, min_height: u16) {
    let error_message = vec![
        Line::from(Span::styled(
            "Terminal too small!",
            Style::default()
                .fg(Color::Rgb(205, 0, 0))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Minimum size: {}x{}", min_width, min_height),
            Style::default().fg(Color::Rgb(255, 255, 255)),
        )),
        Line::from(Span::styled(
            format!("Current size: {}x{}", area.width, area.height),
            Style::default().fg(Color::Rgb(255, 255, 255)),
        )),
    ];

    let paragraph = Paragraph::new(error_message)
        .style(Style::default().fg(Color::Rgb(255, 255, 255)))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(4),
            Constraint::Fill(1),
        ])
        .split(area);

    let centered = chunks[1];

    frame.render_widget(paragraph, centered);
}

pub fn render_input(frame: &mut Frame, app: &App) {
    const MIN_WIDTH: u16 = 30;
    const MIN_HEIGHT: u16 = 12;

    let area = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_error(frame, area, MIN_WIDTH, MIN_HEIGHT);
        return;
    }

    const INPUT_WIDTH: u16 = 50;

    let chunks = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(INPUT_WIDTH),
        Constraint::Fill(1),
    ])
    .split(area);

    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(10),
        Constraint::Fill(1),
    ])
    .split(chunks[1]);

    let centered = vertical[1];

    let layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Fill(1),
    ])
    .split(centered);

    let input_user = layout[1];
    let input_token = layout[2];

    fn calculate_view(text: &str, width: u16) -> (usize, &str) {
        let usable_width = width.saturating_sub(4) as usize;
        let len = text.len();

        let offset = len.saturating_sub(usable_width);

        (offset, &text[offset..])
    }

    let (user_offset, user_visible) = calculate_view(&app.config.username, input_user.width);

    let user_style = if app.focus == Focus::Username {
        Style::default()
            .fg(Color::Rgb(255, 255, 255))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Rgb(127, 127, 127))
    };

    let username_block = Block::bordered()
        .title(" Username ")
        .border_type(BorderType::Plain)
        .style(user_style);

    frame.render_widget(
        Paragraph::new(format!(" {}", user_visible)).block(username_block),
        input_user,
    );

    let (token_offset, token_visible) = calculate_view(&app.config.token, input_token.width);

    let token_style = if app.focus == Focus::Token {
        Style::default()
            .fg(Color::Rgb(255, 255, 255))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Rgb(127, 127, 127))
    };

    let token_block = Block::bordered()
        .title(" Token ")
        .border_type(BorderType::Plain)
        .style(token_style);

    frame.render_widget(
        Paragraph::new(format!(" {}", token_visible)).block(token_block),
        input_token,
    );

    match app.focus {
        Focus::Username => frame.set_cursor_position((
            input_user.x + 2 + (app.config.username.len() - user_offset) as u16,
            input_user.y + 1,
        )),
        Focus::Token => frame.set_cursor_position((
            input_token.x + 2 + (app.config.token.len() - token_offset) as u16,
            input_token.y + 1,
        )),
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    const MIN_WIDTH: u16 = 30;
    const MIN_HEIGHT: u16 = 12;

    let area = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_error(frame, area, MIN_WIDTH, MIN_HEIGHT);
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
        y: area.y,
        width: calendar_width + 2,
        height: calendar_height + 2,
    };

    let block = Block::default()
        .title(format!(" trexanh - @{} ", app.config.username))
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Rgb(255, 255, 255)));

    let inner = block.inner(calendar_area);
    frame.render_widget(block, calendar_area);

    let mut lines: Vec<Line> = vec![];

    lines.push(Line::from(vec![
        Span::raw("     "),
        Span::styled(
            get_month_labels(displayed_weeks),
            Style::default().fg(Color::Rgb(255, 255, 255)),
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
                Style::default().fg(Color::Rgb(255, 255, 255)),
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
        .style(Style::default().fg(Color::Rgb(255, 255, 255)))
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
