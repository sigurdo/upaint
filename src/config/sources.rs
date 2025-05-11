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
pub enum BaseConfig {
    Included(BaseConfigIncluded),
    Path(PathBuf),
}

impl Default for BaseConfig {
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
    pub fn toml_str(self) -> &'static str {
        match self {
            Self::Standard => include_str!("base/standard.toml"),
            Self::Vim => include_str!("base/vim.toml"),
        }
    }

    pub fn toml_string(self) -> String {
        self.toml_str().to_string()
    }
}

impl BaseConfig {
    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        if let Ok(included) = BaseConfigIncluded::from_str(s) {
            Ok(Self::Included(included))
        } else {
            Ok(Self::Path(s.try_into()?))
        }
    }
    pub fn toml_string(&self) -> anyhow::Result<String> {
        Ok(match self {
            Self::Included(included) => included.toml_string(),
            Self::Path(path) => std::fs::read_to_string(path)?,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConfigSources {
    pub base: BaseConfig,
    pub user: Vec<PathBuf>,
}

pub fn load_config_from_sources(sources: &ConfigSources) -> Result<Config, super::ErrorLoadConfig> {
    fn load_config_toml_table(sources: &ConfigSources) -> anyhow::Result<toml::Table> {
        let mut toml_table = toml::Table::new();
        let mut config_stack = vec![];
        config_stack.push(sources.base.toml_string()?);
        if sources.user.len() == 0 {
            if let Ok(user_config) = super::local_config_toml() {
                config_stack.push(user_config);
            }
        }
        for user_config in &sources.user {
            config_stack.push(std::fs::read_to_string(user_config)?);
        }

        for config_toml in config_stack {
            toml_table.extend_recurse_tables(config_toml.parse::<toml::Table>()?);
        }
        Ok(toml_table)
    }
    let table = match load_config_toml_table(sources) {
        Err(e) => return Err(ErrorLoadConfig::Any(e)),
        Ok(table) => table,
    };
    match super::load_config_from_table(table) {
        Ok(config) => Ok(config),
        Err(err) => Err(ErrorLoadConfig::ConfigInvalid(err)),
    }
}
