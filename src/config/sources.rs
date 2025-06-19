use anyhow::bail;
use std::path::PathBuf;

use super::Config;
use super::ErrorLoadConfig;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum BaseConfigIncluded {
    #[default]
    Standard,
    Vim,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConfigSource {
    Included(BaseConfigIncluded),
    Path(PathBuf),
}

impl Default for ConfigSource {
    fn default() -> Self {
        Self::Included(BaseConfigIncluded::default())
    }
}

impl BaseConfigIncluded {
    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        let ok = match s {
            "standard" => Self::Standard,
            "vim" => Self::Vim,
            _ => bail!("No included base config named {}", s),
        };
        Ok(ok)
    }
    pub fn ron_str(self) -> &'static str {
        match self {
            Self::Standard => include_str!("../../upaint_standard_config/upaint_standard.ron"),
            Self::Vim => include_str!("../../upaint_standard_config/upaint_standard.ron"),
        }
    }
    pub fn ron_string(self) -> String {
        self.ron_str().to_string()
    }
}

impl ConfigSource {
    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        if let Ok(included) = BaseConfigIncluded::from_str(s) {
            Ok(Self::Included(included))
        } else {
            Ok(Self::Path(s.try_into()?))
        }
    }
    pub fn load_config(&self) -> Result<Config, super::ErrorLoadConfig> {
        let ron = match self {
            Self::Included(included) => included.ron_string(),
            Self::Path(path) => match std::fs::read_to_string(path) {
                Ok(ok) => ok,
                Err(err) => return Err(ErrorLoadConfig::Any(err.into())),
            },
        };
        let config = match ron::from_str::<Config>(ron.as_str()) {
            Ok(ok) => ok,
            Err(err) => return Err(ErrorLoadConfig::ConfigInvalid(err.into())),
        };
        if !config.color_themes.contains_key(&config.color_theme) {
            return Err(ErrorLoadConfig::ConfigInvalid(anyhow::anyhow!(
                "Chosen color theme {} not found in color themes table",
                config.color_theme
            )));
        }
        Ok(config)
    }
}
