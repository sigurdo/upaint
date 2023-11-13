use serde::{Deserialize, Serialize};

use crate::{config::config_file_value::ConfigFileValue, config_struct_pair};

use super::{ConfigFileColor, ConfigFileStyle};

config_struct_pair!(
    ColorThemeCanvasStandardColors,
    ConfigFileColorThemeCanvasStandardColors,
    black: ConfigFileColor,
    red: ConfigFileColor,
    green: ConfigFileColor,
    yellow: ConfigFileColor,
    blue: ConfigFileColor,
    magenta: ConfigFileColor,
    cyan: ConfigFileColor,
    white: ConfigFileColor,
    bright_black: ConfigFileColor,
    bright_red: ConfigFileColor,
    bright_green: ConfigFileColor,
    bright_yellow: ConfigFileColor,
    bright_blue: ConfigFileColor,
    bright_magenta: ConfigFileColor,
    bright_cyan: ConfigFileColor,
    bright_white: ConfigFileColor,
);

config_struct_pair!(
    ColorThemeCanvas,
    ConfigFileColorThemeCanvas,
    default_style: ConfigFileStyle,
    standard_colors: ConfigFileColorThemeCanvasStandardColors,
);
