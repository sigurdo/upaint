use std::collections::{HashMap, LinkedList};
use std::marker::PhantomData;

use crate::config::TomlValue;
use crate::config::keybindings::Keystroke;
use crate::keystrokes::KeystrokeSequence;

use serde::{Serialize, Deserialize, de};

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Keymaps<T>(HashMap<String, T>);
pub type Keymaps<T> = HashMap<KeystrokeSequence, T>;

impl<T> TomlValue for Keymaps<T> {
    type ConfigValue = Self;
    fn to_config_value(self) -> Self::ConfigValue {
        self
    }
}

pub struct ConfigFileKeymapsVisitor<T> {
    keymaps : PhantomData<T>,
}
impl<'de, T> de::Visitor<'de> for ConfigFileKeymapsVisitor<T> {
    type Value = T;
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>, {
        Err(de::Error::custom("TBI"))
    }
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a key-value map, where the keys represent keystrokes and the values represent actions"
        )
    }
}

