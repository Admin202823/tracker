use chrono::{DateTime, Duration, Utc};
use std::cell::RefCell;

use crate::{config::Config, coordinates::Lla, object::Object};

/// Shared state accessible by all widgets.
#[derive(Default)]
pub struct SharedState {
    /// Collection of objects loaded from the selected satellite groups.
    pub objects: Vec<Object>,
    /// Selected satellite object.
    pub selected_object: Option<Object>,
    /// Hovered satellite object.
    pub hovered_object: Option<Object>,
    /// Simulation time state.
    pub time: TimeState,
    /// Configured ground station.
    pub ground_station: Option<Station>,
    /// Cached predicted passes for the selected satellite (interior mutable).
    /// Format: (satellite_name, list of (AOS, LOS, max_elevation) tuples)
    cached_passes: RefCell<Option<(String, Vec<(DateTime<Utc>, DateTime<Utc>, f64)>)>>,
}

impl SharedState {
    pub fn with_config(config: Config) -> Self {
        let ground_station = config.sky.ground_station.map(|station| Station {
            name: station
                .name
                .unwrap_or_else(|| station.position.country_city().1),
            position: station.position,
        });

        Self {
            ground_station,
            ..Self::default()
        }
    }

    /// Gets cached passes for the selected satellite, if available.
    pub fn get_cached_passes(&self) -> Option<Vec<(DateTime<Utc>, DateTime<Utc>, f64)>> {
        let cache = self.cached_passes.borrow();
        if let Some((name, passes)) = cache.as_ref() {
            // Only return cached passes if the selected object still has the same name
            if let Some(obj) = &self.selected_object {
                if obj.name() == Some(name.as_str()) {
                    return Some(passes.clone());
                }
            }
        }
        None
    }

    /// Sets cached passes for the selected satellite.
    pub fn set_cached_passes(&self, passes: Vec<(DateTime<Utc>, DateTime<Utc>, f64)>) {
        if let Some(obj) = &self.selected_object {
            if let Some(name) = obj.name() {
                *self.cached_passes.borrow_mut() = Some((name.to_string(), passes));
            }
        }
    }
}

/// Shared time state.
#[derive(Default)]
pub struct TimeState {
    /// Time offset from the current UTC time for time simulation.
    time_offset: Duration,
}

impl TimeState {
    /// Returns the current simulation time.
    pub fn time(&self) -> DateTime<Utc> {
        Utc::now() + self.time_offset
    }

    /// Sets the current simulation time.
    pub fn set_time(&mut self, time: DateTime<Utc>) {
        self.time_offset = time - Utc::now();
    }

    /// Returns the time offset.
    pub fn time_offset(&self) -> Duration {
        self.time_offset
    }

    /// Sets the time offset directly.
    pub fn set_time_offset(&mut self, offset: Duration) {
        self.time_offset = offset;
    }

    /// Advances the simulation time.
    pub fn advance_time(&mut self, delta: Duration) {
        self.time_offset += delta;
    }

    /// Rewinds the simulation time.
    pub fn rewind_time(&mut self, delta: Duration) {
        self.time_offset -= delta;
    }
}

/// Ground station.
pub struct Station {
    pub name: String,
    pub position: Lla,
}
