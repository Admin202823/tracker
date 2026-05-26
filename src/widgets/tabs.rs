use std::fmt::{Display, Formatter};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::Block};
use rust_i18n::t;

use crate::{
    app::States,
    config::UiConfig,
    event::Event,
    shared_state::SharedState,
    widgets::{
        information::{Information, InformationState},
        sky::{Sky, SkyState},
    },
};

/// Tabs enum for the right-side panel.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Info,
    Sky,
}

impl Tab {
    /// Returns an iterator over all tabs.
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Info, Self::Sky].into_iter()
    }

    /// Returns the next tab.
    fn next(&self) -> Self {
        match self {
            Tab::Sky => Tab::Info,
            Tab::Info => Tab::Sky,
        }
    }

    /// Returns the previous tab.
    fn previous(&self) -> Self {
        self.next()
    }
}

impl Display for Tab {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Tab::Info => write!(f, "{}", t!("info.title")),
            Tab::Sky => write!(f, "{}", t!("sky.title")),
        }
    }
}

pub struct Tabs<'a> {
    pub state: &'a mut TabsState,
    pub shared: &'a SharedState,
    pub sky_state: &'a mut SkyState,
    pub information_state: &'a mut InformationState,
    pub ui: &'a UiConfig,
}

#[derive(Default)]
pub struct TabsState {
    pub selected: Tab,
}

impl Widget for Tabs<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);
        let [top_area, bottom_area] = vertical.areas(area);

        self.block().render(top_area, buf);
        self.render_tab(bottom_area, buf);
    }
}

impl Tabs<'_> {
    fn block(&self) -> Block<'static> {
        let mut block = Block::bordered();
        for tab in Tab::iter() {
            let color = if tab == self.state.selected {
                self.ui.tab_selected_color.0
            } else {
                self.ui.tab_unselected_color.0
            };
            block = block.title(Line::from(Span::styled(
                tab.to_string(),
                Style::default().fg(color),
            )));
        }
        block
    }

    fn render_tab(self, area: Rect, buf: &mut Buffer) {
        match self.state.selected {
            Tab::Sky => {
                let sky = Sky {
                    state: self.sky_state,
                    shared: self.shared,
                };
                sky.render(area, buf);
            }
            Tab::Info => {
                let information = Information {
                    state: self.information_state,
                    shared: self.shared,
                };
                information.render(area, buf);
            }
        }
    }
}

pub fn handle_event(event: Event, states: &mut States) -> Result<()> {
    match event {
        Event::Key(event) => handle_key_event(event, states),
        _ => Ok(()),
    }
}

fn handle_key_event(event: KeyEvent, states: &mut States) -> Result<()> {
    let state = &mut states.tab_state;

    if event.code == KeyCode::Tab {
        if event.modifiers == KeyModifiers::SHIFT {
            state.selected = state.selected.previous();
        } else {
            state.selected = state.selected.next();
        }
    }

    Ok(())
}
