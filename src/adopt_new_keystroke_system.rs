use crate::canvas::raw::iter::CanvasIndexIterator;
use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::raw::iter::StopCondition;
use crate::canvas::raw::iter::WordBoundaryType;
use crate::canvas::CanvasIndex;
use crate::DirectionFree;
use crate::ProgramState;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use keystrokes_parsing::from_keystrokes_by_preset_keymap;
use keystrokes_parsing::impl_from_keystrokes_by_preset_keymap;
use keystrokes_parsing::FromKeystrokes;
use keystrokes_parsing::GetKeymap;
use keystrokes_parsing::Keymap;
use keystrokes_parsing::Keystroke;
use keystrokes_parsing::Presetable;
use nestify::nest;
// use keystrokes_parsing::PresetDerive;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub struct TestA {
    a: Keymap<u32>,
    b: Keymap<u64>,
}
impl GetKeymap<u32> for TestA {
    fn get_keymap<'a>(&'a self) -> &'a Keymap<u32> {
        &self.a
    }
}
impl GetKeymap<u64> for TestA {
    fn get_keymap<'a>(&'a self) -> &'a Keymap<u64> {
        &self.b
    }
}
fn testa() {
    let a = TestA {
        a: Keymap::new(),
        b: Keymap::new(),
    };
    let keymap = <TestA as GetKeymap<u32>>::get_keymap(&a);
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum U32KeymapEntry {
    TypeDecimal,
    #[serde(untagged)]
    U32(u32),
}
impl Presetable<Config> for u32 {
    type Preset = U32KeymapEntry;
    fn from_keystrokes_by_preset(
        preset: U32KeymapEntry,
        _keystrokes: &mut keystrokes_parsing::KeystrokeIterator,
        _config: &Config,
    ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
        match preset {
            U32KeymapEntry::U32(value) => Ok(value),
            U32KeymapEntry::TypeDecimal => panic!("Not implemented"),
        }
    }
}
#[derive(Presetable)]
pub struct ActionA {
    a: u32,
}
// #[derive(Presetable)]
pub enum ActionEnum {
    A(ActionA),
}

macro_rules! keymaps {
    ($($ident:ident : $type:ty,)*) => {
        pub struct Keymaps {
            $(
                $ident: Keymap<<$type as Presetable<Config>>::Preset>,
            )*
        }
        $(impl FromKeystrokes<Config> for $type {
            fn from_keystrokes(keystrokes: &mut keystrokes_parsing::KeystrokeIterator, config: &Config) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
                from_keystrokes_by_preset_keymap(
                    &config.keymaps.$ident,
                    keystrokes,
                    config,
                )
            }
        })*
    }
}
macro_rules! impl_presetable_by_self {
    ($type:ty) => {
        impl Presetable<Config> for $type {
            type Preset = Self;
            fn from_keystrokes_by_preset(
                preset: Self::Preset,
                _keystrokes: &mut keystrokes_parsing::KeystrokeIterator,
                _config: &Config,
            ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
                Ok(preset)
            }
        }
    };
}
// keymaps!(
//     keymap_u32: u32,
//     action_a: ActionA,
// );
// pub struct Config {
//     keymaps: Keymaps,
// }
// fn test() {
//     let k = Keymaps {
//         keymap_u32: Keymap::new(),
//         action_a: Keymap::new(),
//     };
//     // k.action_a.current.get().unwrap().
// }
// #[derive(GetKeymap, FromKeystrokes)]
// #[from_keystrokes(config = "Config")]
// pub struct ConfigKeymaps {
//     #[preset_for(u32)]
//     keymap_u32: Keymap<U32KeymapEntry>,
//     #[preset_for(ActionA)]
//     keymap_action_a: Keymap<ActionAPreset>,
//     // #[preset_for(ActionEnum)]
//     // keymap_action: Keymap<ActionEnumPreset>,
//     // #[preset_for(MotionEnum)]
//     // motions: Keymap<MotionEnumPreset>,
// }
//
// pub struct Config {
//     keymaps: ConfigKeymaps,
// }
// impl<T: Clone> GetKeymap<T> for Config {
//     fn get_keymap<'a>(&'a self) -> &'a Keymap<T> {
//         self.keymaps.get_keymap()
//     }
// }
// impl<T: GetKeymap<ConfigKeymaps>> GetKeymap<Config> for T {
//     fn get_keymap<'a>(config: &'a Config) -> &'a Keymap<Self> {
//         T::get_keymap(&config.keymaps)
//     }
// }
// impl GetKeymap<Config> for U32KeymapEntry {
//     fn get_keymap<'a>(config: &'a Config) -> &'a Keymap<Self> {
//         <U32KeymapEntry as GetKeymap<ConfigKeymaps>>::get_keymap(&config.keymaps)
//     }
// }

pub trait Motion: Debug {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex>;
}
#[derive(Presetable)]
#[presetable(fields_required)]
pub enum MotionEnum {
    Stay(Stay),
    FixedNumberOfCells(FixedNumberOfCells),
    WordBoundary(WordBoundary),
}

#[derive(Debug, Presetable)]
pub struct Stay {}
impl Motion for Stay {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        vec![start]
    }
}

#[derive(Debug, Presetable)]
pub struct FixedNumberOfCells {
    direction: DirectionFree,
    number_of_cells: u16,
    jump: CanvasIterationJump,
}
impl Motion for FixedNumberOfCells {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        let canvas = program_state.canvas.raw();
        let it = CanvasIndexIterator::new(
            canvas,
            start,
            self.direction,
            Some(self.jump),
            StopCondition::NthCell(self.number_of_cells),
        );
        it.collect()
    }
}

#[derive(Debug, Presetable)]
pub struct WordBoundary {
    direction: DirectionFree,
    boundary_type: WordBoundaryType,
}
impl Motion for WordBoundary {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        let canvas = program_state.canvas.raw();
        let it = CanvasIndexIterator::new(
            canvas,
            start,
            self.direction,
            Some(CanvasIterationJump::Diagonals),
            StopCondition::WordBoundary(self.boundary_type),
        );
        it.collect()
    }
}

impl_presetable_by_self!(u16);
impl_presetable_by_self!(DirectionFree);
impl_presetable_by_self!(CanvasIterationJump);
impl_presetable_by_self!(WordBoundaryType);

keymaps!(
    keymap_u32: u32,
    keymap_u16: u16,
    motions: MotionEnum,
    directions: DirectionFree,
    canvas_iteration_jumps: CanvasIterationJump,
    word_boundary_type: WordBoundaryType,
);
nest! {
    pub struct Config {
        keymaps: Keymaps,
    }
}

#[test]
fn test() {
    let keymaps = Keymaps {
        keymap_u32: Keymap::new(),
        keymap_u16: Keymap::new(),
        motions: Keymap::new(),
        directions: Keymap::new(),
        canvas_iteration_jumps: Keymap::new(),
        word_boundary_type: Keymap::new(),
        // keymap_action_a: Keymap::new(),
        // keymap_action: Keymap::new(),
        // motions: Keymap::new(),
    };
    /*
    let config = Config { keymaps };
    let res = U32KeymapEntry::get_keymap(&config);
    let a = vec![Keystroke {
        code: KeyCode::Up,
        modifiers: KeyModifiers::NONE,
    }];
    let action_a = ActionEnum::from_keystrokes(&mut a.iter(), &config);

    let config_toml = r###"
        "abc" = { a = 65 }
        "###;
    let keymap: Keymap<ActionAPreset> = toml::from_str(config_toml).unwrap();

    dbg!(keymap
        .next
        .get(&Keystroke {
            modifiers: KeyModifiers::NONE,
            code: KeyCode::Char('a'),
        })
        .unwrap()
        .next
        .get(&Keystroke {
            modifiers: KeyModifiers::NONE,
            code: KeyCode::Char('b'),
        })
        .unwrap()
        .next
        .get(&Keystroke {
            modifiers: KeyModifiers::NONE,
            code: KeyCode::Char('c'),
        })
        .unwrap()
        .current
        .clone()
        .unwrap());
    */
}
