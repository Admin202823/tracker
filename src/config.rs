use ratatui::style::Color;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

use crate::coordinates::Lla;

/// Wrapper for configurable colors in TOML.
///
/// Supports regular ANSI color names, common aliases, and RGB hex codes like
/// `#FFA500`.
#[derive(Clone, Copy, Debug)]
pub struct ConfigColor(pub Color);

impl Default for ConfigColor {
    fn default() -> Self {
        Self(Color::White)
    }
}

impl<'de> Deserialize<'de> for ConfigColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input = String::deserialize(deserializer)?;
        let normalized = input.to_lowercase();

        if let Ok(color) = Color::from_str(&normalized) {
            return Ok(Self(color));
        }

        if let Some(color) = parse_color_alias(&normalized) {
            return Ok(Self(color));
        }

        Err(serde::de::Error::custom(format!("Invalid color: {input}")))
    }
}

impl From<ConfigColor> for Color {
    fn from(config_color: ConfigColor) -> Self {
        config_color.0
    }
}

fn parse_color_alias(input: &str) -> Option<Color> {
    match input
        .replace([' ', '-', '_'], "")
        .replace("bright", "light")
        .replace("grey", "gray")
        .replace("silver", "gray")
        .as_str()
    {
        "orange" => Some(Color::Rgb(255, 165, 0)),
        "pink" => Some(Color::Rgb(255, 192, 203)),
        "purple" => Some(Color::Rgb(128, 0, 128)),
        "teal" => Some(Color::Rgb(0, 128, 128)),
        "brown" => Some(Color::Rgb(165, 42, 42)),
        "navy" => Some(Color::Rgb(0, 0, 128)),
        "lime" => Some(Color::Rgb(0, 255, 0)),
        "olive" => Some(Color::Rgb(128, 128, 0)),
        "maroon" => Some(Color::Rgb(128, 0, 0)),
        "aqua" => Some(Color::Rgb(0, 255, 255)),
        _ => None,
    }
}

/// Configuration for the application.
#[derive(Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub ui: UiConfig,
    pub world_map: WorldMapConfig,
    pub satellite_groups: SatelliteGroupsConfig,
    pub sky: SkyConfig,
    pub timeline: TimelineConfig,
    pub predicted_passes: PredictedPassesConfig,
}

/// Configuration for application UI colors.
#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UiConfig {
    pub panel_title_color: ConfigColor,
    pub tab_selected_color: ConfigColor,
    pub tab_unselected_color: ConfigColor,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            panel_title_color: ConfigColor(Color::White),
            tab_selected_color: ConfigColor(Color::White),
            tab_unselected_color: ConfigColor(Color::Gray),
        }
    }
}

/// Configuration for the world map widget.
#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WorldMapConfig {
    pub follow_object: bool,
    pub follow_smoothing: f64,
    pub show_terminator: bool,
    pub show_visibility_area: bool,
    pub lon_delta_deg: f64,
    pub map_color: ConfigColor,
    pub trajectory_color: ConfigColor,
    pub terminator_color: ConfigColor,
    pub visibility_area_color: ConfigColor,
    pub predicted_passes_count: u32,
}

impl Default for WorldMapConfig {
    fn default() -> Self {
        Self {
            follow_object: true,
            follow_smoothing: 0.3,
            show_terminator: true,
            show_visibility_area: false, // Disabled by default for better debug performance
            lon_delta_deg: 10.0,
            map_color: ConfigColor(Color::Gray),
            trajectory_color: ConfigColor(Color::LightBlue),
            terminator_color: ConfigColor(Color::DarkGray),
            visibility_area_color: ConfigColor(Color::Yellow),
            predicted_passes_count: 1,
        }
    }
}

/// Configuration for satellite groups widget.
#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SatelliteGroupsConfig {
    pub cache_lifetime_mins: u64,
    pub groups: Vec<GroupConfig>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupConfig {
    pub label: String,
    pub id: Option<String>,
    /// NORAD Catalog ID (e.g., 25544 for ISS).
    /// Mutually exclusive with `id` and `group`.
    pub norad_id: Option<u64>,
    pub group: Option<String>,
}

impl GroupConfig {
    fn with_id(label: String, cospar_id: String) -> Self {
        Self {
            label,
            id: Some(cospar_id),
            norad_id: None,
            group: None,
        }
    }

    fn with_group(label: String, group_name: String) -> Self {
        Self {
            label,
            id: None,
            norad_id: None,
            group: Some(group_name),
        }
    }
}

impl Default for SatelliteGroupsConfig {
    fn default() -> Self {
        Self {
            cache_lifetime_mins: 2 * 60,
            groups: vec![
                GroupConfig::with_id("ISS".into(), "1998-067A".into()),
                GroupConfig::with_id("CSS".into(), "2021-035A".into()),
                GroupConfig::with_group("Weather".into(), "weather".into()),
                GroupConfig::with_group("NOAA".into(), "noaa".into()),
                GroupConfig::with_group("GOES".into(), "goes".into()),
                GroupConfig::with_group("Earth resources".into(), "resource".into()),
                GroupConfig::with_group("Search & rescue".into(), "sarsat".into()),
                GroupConfig::with_group("Disaster monitoring".into(), "dmc".into()),
                GroupConfig::with_group("GPS".into(), "gps-ops".into()),
                GroupConfig::with_group("GLONASS".into(), "glo-ops".into()),
                GroupConfig::with_group("Galileo".into(), "galileo".into()),
                GroupConfig::with_group("Beidou".into(), "beidou".into()),
                GroupConfig::with_group("Space & Earth Science".into(), "science".into()),
                GroupConfig::with_group("Geodetic".into(), "geodetic".into()),
                GroupConfig::with_group("Engineering".into(), "engineering".into()),
                GroupConfig::with_group("Education".into(), "education".into()),
                GroupConfig::with_group("Military".into(), "military".into()),
                GroupConfig::with_group("Radar calibration".into(), "radar".into()),
                GroupConfig::with_group("CubeSats".into(), "cubesat".into()),
            ],
        }
    }
}

/// Configuration for the sky widget.
#[derive(Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SkyConfig {
    pub ground_station: Option<GroundStationConfig>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundStationConfig {
    pub name: Option<String>,
    pub position: Lla,
}

/// Configuration for the timeline widget.
#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TimelineConfig {
    pub time_delta_mins: i64,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self { time_delta_mins: 1 }
    }
}

/// Configuration for the predicted passes widget.
#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PredictedPassesConfig {
    pub min_elevation_deg: f64,
}

impl Default for PredictedPassesConfig {
    fn default() -> Self {
        Self { min_elevation_deg: 30.0 }
    }
}
