use std::{fmt::Display, sync::LazyLock, time::Duration};

use tokio::fs;

use crate::config::GroupConfig;

/// The timeout duration for HTTP requests.
const HTTP_TIMEOUT_SECS: u64 = 10;

/// The shared HTTP client used for making requests.
static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .expect("failed to create HTTP client")
});

/// The `Group` type.
///
/// Type [`Group`] represents a group of satellites.
#[derive(Clone, Eq, Debug)]
pub struct Group {
    label: String,
    identifier: Identifier,
}

impl Group {
    /// Returns the label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns SGP4 elements.
    ///
    /// If cache is expired, fetches elements from <https://celestrak.org>.
    /// Otherwise, reads elements from cache.
    ///
    /// # Arguments
    ///
    /// * `cache_lifetime` - Duration for which the cache is considered valid.
    pub async fn get_elements(&self, cache_lifetime: Duration) -> Option<Vec<sgp4::Elements>> {
        let cache_path = std::env::temp_dir().join(format!(
            "tracker/{}.json",
            self.identifier.to_string().to_lowercase()
        ));
        fs::create_dir_all(cache_path.parent().unwrap())
            .await
            .unwrap();

        // Check if cache needs refresh (doesn't exist or expired)
        let needs_refresh = !fs::try_exists(&cache_path).await.unwrap()
            || fs::metadata(&cache_path)
                .await
                .unwrap()
                .modified()
                .unwrap()
                .elapsed()
                .unwrap()
                > cache_lifetime;

        if needs_refresh {
            let elements = self.fetch_elements().await?;
            let path = cache_path.clone();
            let json = serde_json::to_string(&elements).unwrap();
            fs::write(path, json).await.unwrap();
        }

        let json = fs::read_to_string(&cache_path).await.unwrap();
        serde_json::from_str(&json).expect("failed to parse cache")
    }

    /// Fetches SGP4 elements from <https://celestrak.org>.
    async fn fetch_elements(&self) -> Option<Vec<sgp4::Elements>> {
        const URL: &str = "https://celestrak.org/NORAD/elements/gp.php";
        let mut request = HTTP_CLIENT.get(URL).query(&[("FORMAT", "json")]);
        request = match &self.identifier {
            Identifier::CosparId(id) => request.query(&[("INTDES", id)]),
            Identifier::NoradId(id) => request.query(&[("CATNR", &id.to_string())]),
            Identifier::Group(group) => request.query(&[("GROUP", group)]),
        };

        let response = match request.send().await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Failed to fetch from celestrak.org: {}", e);
                return None;
            }
        };

        if !response.status().is_success() {
            eprintln!(
                "Celestrak returned HTTP {} for group '{}'",
                response.status(),
                self.label
            );
            return None;
        }

        match response.json::<Vec<sgp4::Elements>>().await {
            Ok(data) => Some(data),
            Err(e) => {
                eprintln!(
                    "Failed to parse JSON from Celestrak for '{}': {}",
                    self.label, e
                );
                None
            }
        }
    }
}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

use anyhow::{anyhow, Error};

impl TryFrom<GroupConfig> for Group {
    type Error = Error;

    fn try_from(config: GroupConfig) -> Result<Self, Self::Error> {
        let count = config.id.is_some() as u8
            + config.norad_id.is_some() as u8
            + config.group.is_some() as u8;

        if count != 1 {
            return Err(anyhow!(
                "Group '{}' must have exactly one of: id, norad-id, group",
                config.label
            ));
        }

        let identifier = if let Some(id) = config.id {
            Identifier::CosparId(id)
        } else if let Some(norad) = config.norad_id {
            Identifier::NoradId(norad)
        } else if let Some(group) = config.group {
            Identifier::Group(group)
        } else {
            unreachable!()
        };

        Ok(Self {
            label: config.label,
            identifier,
        })
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum Identifier {
    CosparId(String),
    NoradId(u64),
    Group(String),
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Identifier::CosparId(id) => write!(f, "{id}"),
            Identifier::NoradId(id) => write!(f, "{id}"),
            Identifier::Group(group) => write!(f, "{group}"),
        }
    }
}
