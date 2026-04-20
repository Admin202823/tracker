use chrono::{Duration, Local, Utc};
use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph},
};
use rust_i18n::t;
use anyhow::Result;

use crate::{config::PredictedPassesConfig, event::Event, shared_state::SharedState, utils::calculate_pass_times};

/// State for the predicted passes widget.
#[derive(Clone)]
pub struct PredictedPassesState {
    pub config: PredictedPassesConfig,
    pub show_hidden: bool,
}

impl Default for PredictedPassesState {
    fn default() -> Self {
        Self {
            config: PredictedPassesConfig::default(),
            show_hidden: false,
        }
    }
}

impl PredictedPassesState {
    pub fn with_config(config: PredictedPassesConfig) -> Self {
        Self {
            config,
            show_hidden: false,
        }
    }
}

/// A widget that displays predicted passes for a selected satellite.
pub struct PredictedPasses<'a> {
    pub shared: &'a SharedState,
    pub state: &'a PredictedPassesState,
}

impl Widget for PredictedPasses<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(selected_object) = &self.shared.selected_object else {
            return;
        };
        let Some(ground_station) = &self.shared.ground_station else {
            return;
        };

        // Try to get cached passes first
        let all_pass_segments = if let Some(cached) = self.shared.get_cached_passes() {
            cached
        } else {
            // Get current local time for calculating future passes
            let current_local_time = Local::now();
            
            // Calculate passes for the next 24 hours from the current local time
            let start_time = current_local_time.with_timezone(&Utc);
            let end_time = start_time + Duration::hours(24);

            let calculated = calculate_pass_times(
                selected_object,
                &ground_station.position,
                &start_time,
                &end_time,
            );
            
            // Cache the result
            self.shared.set_cached_passes(calculated.clone());
            calculated
        };

        // Build the content lines in chronological order.
        let mut lines = Vec::new();
        let mut shown_any = false;
        let min_el = self.state.config.min_elevation_deg;

        for (aos, los, max_el) in all_pass_segments {
            let hidden = max_el < min_el;
            if hidden && !self.state.show_hidden {
                continue;
            }

            shown_any = true;
            let aos_local = aos.with_timezone(&Local);
            let los_local = los.with_timezone(&Local);
            let date_str = aos_local.format("%Y-%m-%d").to_string();
            let aos_str = aos_local.format("%H:%M:%S").to_string();
            let los_str = los_local.format("%H:%M:%S").to_string();
            let duration = los - aos;
            let duration_str = format!("{}m", duration.num_minutes());
            let max_el_str = format!("{:.1}°", max_el);

            let (aos_color, los_color, meta_color) = if hidden {
                (Color::Gray, Color::DarkGray, Color::Gray)
            } else {
                (Color::Green, Color::Red, Color::Yellow)
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{} AOS: {}", date_str, aos_str),
                    Style::default().fg(aos_color),
                ),
                Span::raw(" - "),
                Span::styled(
                    format!("LOS: {}", los_str),
                    Style::default().fg(los_color),
                ),
                Span::raw(" ("),
                Span::styled(
                    duration_str,
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(", Max: "),
                Span::styled(
                    max_el_str,
                    Style::default().fg(meta_color),
                ),
                Span::raw(")"),
            ]));
        }

        if !shown_any {
            lines.push(Line::raw(t!("predicted_passes.no_passes").to_string()));
        }

        let content = format!(
            "{} - {} (Min: {:.0}°{})",
            selected_object.name().unwrap_or("Unknown"),
            ground_station.name,
            self.state.config.min_elevation_deg,
            if self.state.show_hidden { " [H]" } else { "" }
        );

        let block = Block::bordered()
            .title(content.blue());

        let inner_width = 70_u16;
        let inner_height = (lines.len() as u16 + 1).max(5);
        let popup_area = centered_rect(
            inner_width,
            inner_height,
            area,
        );

        Clear.render(popup_area, buf);
        Paragraph::new(lines)
            .block(block)
            .render(popup_area, buf);
    }
}

/// Handle events for the predicted passes widget.
pub fn handle_event(event: Event, states: &mut crate::app::States) -> Result<()> {
    match event {
        Event::Key(key_event) => match key_event.code {
            KeyCode::Char('h') => {
                states.predicted_passes_state.show_hidden = !states.predicted_passes_state.show_hidden;
                Ok(())
            }
            _ => Ok(()),
        },
        _ => Ok(()),
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
