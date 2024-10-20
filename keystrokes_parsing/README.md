# Keystrokes parsing for upaint

## Base code

```rust
pub trait FromKeystrokes<P> {
    fn from_keystrokes(preset: P, keystrokes: &mut Keystrokeiter, config: &Config) -> Result<Self, FromKeystrokesError>;
}

pub fn from_keystrokes_by_keymap<P, C: FromKeystrokes<P>>(keymap: Keymap<P>, keystrokes: &mut Keystrokeiter, config: &Config) -> Result<C, FromKeystrokesError> {
    match ... {
        ... => from_keystrokes_by_keymap(sub_keymap, keystrokes, config),
        ... => Ok(complete),
        ... => Err(...),
    }
}

pub struct Preset
```

## Usage

```rust
use keystrokes_parsing::{FromKeystrokes};
use keystrokes_parsing::{GetKeymap};
use keystrokes_parsing::{Keymap};
use keystrokes_parsing::{impl_from_keystrokes_by_keymap};
use enum_dispatch::enum_dispatch;

#[derive(GetKeymap)]
pub struct Config {
    keymap_u32: Keymap<u32>,
    keymap_i16_i16: Keymap<(i16, i16)>,
    keymap_action: Keymap<(ActionPresetEnum)>,
}

impl GetKeymap<'a, u32> for Config {
    fn get_keymap(&'a self) -> &'a Keymap<u32> {
        &self.keymap_u32
    }
}

impl_from_keystrokes_by_keymap!(u32);
impl_from_keystrokes_by_keymap!((i16, i16));

#[derive(FromKeystrokes)]
#[]
pub struct ActionA {
    count: u32,
    direction: (i16, i16),
}

#[enum_dispatch]
pub trait Action {
    fn execute(&self, &mut program_state);
}

impl Action for ActionA { ... }

#[derive(FromKeystrokes)]
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

impl FromKeystrokes<Keymap<u32>> for u32 {

}

pub struct ActionAPreset {
    count: Preset<u32>,
    direction: Preset<(i16, i16)>,
}

impl FromKeystrokes<ActionAPreset> for ActionA {
    fn from_keystrokes(preset: ActionAPreset, keystrokes: &mut KeystrokeIterator, config: &Config) -> u32 {
        ActionA {
            count: u32::from_keystrokes((), keystrokes, config)?,
            direction: (i16, i16)::from_keystrokes((), keystrokes, config)?,
        }
    }
}
impl FromKeystrokes<Keymap<ActionAPreset>> for ActionA {...}
impl FromKeystrokes<KeymapEntry<ActionAPreset>> for ActionA {...}

pub enum ActionEnumPreset {
    A(Preset<ActionA>)
}

impl FromKeystrokes<ActionEnumPreset> for ActionEnum {...}
impl FromKeystrokes<Keymap<ActionEnumPreset>> for ActionEnum {...}
impl FromKeystrokes<KeymapEntry<ActionEnumPreset>> for ActionEnum {...}

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

