use chrono::{Duration, Local};
use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph},
};
use rust_i18n::t;

use crate::{shared_state::SharedState, utils::calculate_pass_times};

/// A widget that displays predicted passes for a selected satellite.
pub struct PredictedPasses<'a> {
    pub shared: &'a SharedState,
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
        let pass_segments = if let Some(cached) = self.shared.get_cached_passes() {
            cached
        } else {
            // Get current simulation time (which accounts for user's timeline offset)
            let current_time = self.shared.time.time();
            
            // Calculate passes for the next 24 hours from the current simulation time
            let start_time = current_time;
            let end_time = current_time + Duration::hours(24);

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

        // Build the content lines
        let mut lines = Vec::new();
        
        if pass_segments.is_empty() {
            lines.push(Line::raw(t!("predicted_passes.no_passes").to_string()));
        } else {
            for (aos, los, max_el) in pass_segments {
                let aos_local = aos.with_timezone(&Local);
                let los_local = los.with_timezone(&Local);
                
                let aos_str = aos_local.format("%H:%M:%S").to_string();
                let los_str = los_local.format("%H:%M:%S").to_string();
                let duration = los - aos;
                let duration_str = format!("{}m", duration.num_minutes());
                let max_el_str = format!("{:.1}°", max_el);
                
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("AOS: {}", aos_str),
                        Style::default().fg(Color::Green),
                    ),
                    Span::raw(" - "),
                    Span::styled(
                        format!("LOS: {}", los_str),
                        Style::default().fg(Color::Red),
                    ),
                    Span::raw(" ("),
                    Span::styled(
                        duration_str,
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(", Max: "),
                    Span::styled(
                        max_el_str,
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(")"),
                ]));
            }
        }

        let content = format!(
            "{} - {}",
            selected_object.name().unwrap_or("Unknown"),
            ground_station.name
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

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
