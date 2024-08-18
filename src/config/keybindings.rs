use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

use super::{keys::KeyCodeToml, TomlValue};

pub mod deserialize;
pub mod serialize;

// Temporarily re-export Keystroke struct from here for compatibility
// TODO: Remove re-export and fix issues.
pub use crate::keystrokes::Keystroke;
