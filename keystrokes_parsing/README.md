# Keystrokes parsing for upaint

## Base code

```rust
pub trait FromKeystrokes<P> {
    fn from_keystrokes(preset: P, keystrokes: &mut Keystrokeiter, config: &Config) -> Result<Self, FromKeystrokesError>;
}

pub trait GetKeymap<Config> {
    fn get_keymap(config: &Config) -> Keymap<Self>;
}

pub fn from_keystrokes_by_preset_keymap<P, C: FromKeystrokes<P>>(keymap: Keymap<P>, keystrokes: &mut Keystrokeiter, config: &Config) -> Result<C, FromKeystrokesError> {
    match ... {
        ... => from_keystrokes_by_preset_keymap(sub_keymap, keystrokes, config),
        ... => Ok(complete),
        ... => Err(...),
    }
}

pub struct Preset
```

## Usage

```rust
use keystrokes_parsing::{Preset};
use keystrokes_parsing::{FromKeystrokes};
use keystrokes_parsing::{GetKeymap};
use keystrokes_parsing::{Keymap};
use keystrokes_parsing::{impl_from_keystrokes_by_keymap};
use enum_dispatch::enum_dispatch;

#[derive(FromKeystrokesByPresetKeymap)]
pub struct Config {
    keymap_u32: Keymap<u32>,
    keymap_i16_i16: Keymap<(i16, i16)>,
    keymap_action: Keymap<ActionEnumPreset>,
}

impl_from_keystrokes_by_preset_keymap!(u32 => u32);

#[derive(Preset)]
pub struct ActionA {
    count: u32,
    direction: (i16, i16),
}

#[enum_dispatch]
pub trait Action {
    fn execute(&self, &mut program_state);
}

impl Action for ActionA { ... }

#[derive(Preset)]
#[enum_dispatch(Action)]
pub enum ActionEnum {
    A(ActionA),
}
```

Will generate

```rust
impl FromKeystrokesByPreset<Keymap<u32>> for u32 {
    fn from_keystrokes_by_preset(preset: Keymap<u32>, keystrokes: &mut KeystrokeIterator, config: &Config) -> u32 {
        match ... {
            ... => Self::from_keystrokes_by_preset(sub, keystrokes, config),
            ... => Ok(complete),
            ... => Err(...),
        }
    }
}

impl GetKeymap for u32 {
    fn get_keymap(config: &'a Config) -> &'a Keymap<Self> {
        config.keymap_u32
    }
}
impl GetKeymap for (u16, u16) {
    fn get_keymap(config: &'a Config) -> &'a Keymap<Self> {
        config.keymap_u16_u16
    }
}
impl GetKeymap for ActionEnumPreset {
    fn get_keymap(config: &'a Config) -> &'a Keymap<Self> {
        config.keymap_action
    }
}
impl FromKeystrokes<()> for u32 {
    fn from_keystrokes(preset: (), keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<ActionEnum, FromKeystrokesError> {
        ::keystrokes_parsing::from_keystrokes_by_preset_keymap(u32::get_keymap(config), keystrokes, config)
    }
}
impl FromKeystrokes<()> for (u16, u16) {
    fn from_keystrokes(preset: (), keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<ActionEnum, FromKeystrokesError> {
        ::keystrokes_parsing::from_keystrokes_by_preset_keymap((u16, u16)::get_keymap(config), keystrokes, config)
    }
}
impl FromKeystrokes<()> for ActionEnum {
    fn from_keystrokes(preset: (), keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<ActionEnum, FromKeystrokesError> {
        ::keystrokes_parsing::from_keystrokes_by_preset_keymap(ActionEnumPreset::get_keymap(config), keystrokes, config)
    }
}

pub struct ActionAPreset {
    count: Preset<u32>,
    direction: Preset<(i16, i16)>,
}
impl FromKeystrokes<ActionAPreset, Config> for ActionA {
    fn from_keystrokes(preset: ActionAPreset, keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<ActionA, FromKeystrokesError> {
        ActionA {
            count: u32::from_keystrokes(preset.count, keystrokes, config)?,
            direction: (i16, i16)::from_keystrokes(preset.direction, keystrokes, config)?,
        }
    }
}

pub enum ActionEnumPreset {
    A(Preset<ActionA>)
}
impl FromKeystrokes<ActionEnumPreset, Config> for ActionEnum {
    fn from_keystrokes(preset: ActionAPreset, keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<ActionEnum, FromKeystrokesError> {
        match preset {
            A(preset) => Self::A(ActionA::from_keystrokes(preset, keystrokes, config)),
        }
    }
}

// From enum_dispatch:
impl Action for ActionEnum {
    fn execute(&self, &mut program_state) {
        match self {
            Self::A(inner) => inner.execute(program_state),
        }
    }
}
```

And you can do

```rust
fn main() {
    let mut program_state = ...;
    let keymap = toml{
        "a": { A = {} },
    };
    let mut keystrokes = vec![];
    loop {
        let keystroke = ...;
        keystrokes.push(keystroke);
        match ActionEnum::from_keystrokes(keystroke, &keymap) {
            Ok(action) => {
                keystrokes = vec![],
                action.execute(&mut program_state),
            },
            Err(MissingKeystrokes) => (),
            Err(_) => keystrokes = vec![],
        }
    }
}
```

