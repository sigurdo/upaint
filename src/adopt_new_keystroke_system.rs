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
use keystrokes_parsing::FromKeystrokesError;
use keystrokes_parsing::GetKeymap;
use keystrokes_parsing::Keymap;
use keystrokes_parsing::Keystroke;
use keystrokes_parsing::KeystrokeIterator;
use keystrokes_parsing::KeystrokeSequence;
use keystrokes_parsing::PresetStructField;
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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
// #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// pub struct U16Preset(pub u16);
// impl Presetable<Config> for u16 {
//     type Preset = U16Preset;
//     fn from_keystrokes_by_preset(
//         preset: Self::Preset,
//         keystrokes: &mut KeystrokeIterator,
//         config: &Config,
//     ) -> Result<Self, FromKeystrokesError> {
//         Ok(preset.0)
//     }
// }
// impl Default for U16Preset {
//     fn default() -> Self {
//         Self(1),
//     }
// }

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum CharKeymapEntry {
    Type,
    #[serde(untagged)]
    Char(char),
}
impl Presetable<Config> for char {
    type Preset = CharKeymapEntry;
    fn from_keystrokes_by_preset(
        preset: Self::Preset,
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
        match preset {
            CharKeymapEntry::Char(value) => Ok(value),
            CharKeymapEntry::Type => {
                if let Some(keystroke) = keystrokes.next() {
                    if keystroke.modifiers == KeyModifiers::NONE {
                        if let KeyCode::Char(ch) = keystroke.code {
                            Ok(ch)
                        } else {
                            Err(FromKeystrokesError::Invalid)
                        }
                    } else {
                        Err(FromKeystrokesError::Invalid)
                    }
                } else {
                    Err(FromKeystrokesError::MissingKeystrokes)
                }
            }
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
    {$($ident:ident : $type:ty,)*} => {
        #[derive(Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(fields_required)]
pub enum MotionEnum {
    Stay(Stay),
    FixedNumberOfCells(FixedNumberOfCells),
    WordBoundary(WordBoundary),
    FindChar(FindChar),
}

#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct Stay {}
impl Motion for Stay {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        vec![start]
    }
}

fn default_number_of_cells() -> u16 {
    1
}
fn default_jump() -> PresetStructField<CanvasIterationJump> {
    PresetStructField::Preset(CanvasIterationJump::DirectionAsStride)
}
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct FixedNumberOfCells {
    direction: DirectionFree,
    #[presetable(required, default = "default_number_of_cells")]
    number_of_cells: u16,
    // #[presetable(default = "default_jump")]
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
            self.jump,
            StopCondition::NthCell(self.number_of_cells),
        );
        it.collect()
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
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
            CanvasIterationJump::Diagonals,
            StopCondition::WordBoundary(self.boundary_type),
        );
        it.collect()
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct FindChar {
    direction: DirectionFree,
    ch: char,
}
impl Motion for FindChar {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        let canvas = program_state.canvas.raw();
        let it = CanvasIndexIterator::new(
            canvas,
            start,
            self.direction,
            CanvasIterationJump::Diagonals,
            StopCondition::CharacterMatch(self.ch),
        );
        it.collect()
    }
}

impl_presetable_by_self!(u16);
impl_presetable_by_self!(DirectionFree);
impl_presetable_by_self!(CanvasIterationJump);
impl_presetable_by_self!(WordBoundaryType);

keymaps! {
    keymap_u32: u32,
    // keymap_u16: u16,
    character: char,
    motions: MotionEnum,
    directions: DirectionFree,
    canvas_iteration_jumps: CanvasIterationJump,
    word_boundary_type: WordBoundaryType,
}
nest! {
    #[derive(Deserialize)]
    pub struct Config {
        keymaps: Keymaps,
    }
}

const CONFIG_TOML: &str = r###"
[keymaps.keymap_u32]
"6G" = 65
[keymaps.keymap_u16]
[keymaps.character]
"<C-f>" = "f"
"<C-e>" = "e"
# Fallback to typing in character directly
"" = "Type"
[keymaps.motions]
"<C-f>" = { FindChar = { direction = "FromKeystrokes" }}
"<C-l>" = { FindChar = { direction = [0, 1] }}
"<C-h>" = { FindChar = { direction = [0, -1], ch = "@" }}
"<C-j>" = { FindChar = { ch = "@" }}
"t" = { FixedNumberOfCells = { jump = "DirectionAsStride" }}
"f" = { FixedNumberOfCells = { jump = "FromKeystrokes" }}
"" = { FixedNumberOfCells = { number_of_cells = 1, jump = "DirectionAsStride" }}
[keymaps.directions]
"h" = [0, -1]
"j" = [1, 0]
"k" = [-1, 0]
"l" = [0, 1]
[keymaps.canvas_iteration_jumps]
"n" = "NoJump"
"d" = "Diagonals"
"s" = "DirectionAsStride"
[keymaps.word_boundary_type]
"###;

#[test]
pub fn test() {
    let config: Config = toml::from_str(CONFIG_TOML).unwrap();
    // config.keymaps.character.get("abc".into())
    macro_rules! keymaps_contents {
        ($($keymap:ident[$keystrokes:expr] = $expected:expr,)*) => {
            $(
                assert_eq!(
                    config
                        .keymaps
                        .$keymap
                        .get($keystrokes.to_string().try_into().unwrap())
                        .unwrap(),
                    &$expected
                );
            )*
        };
    }
    keymaps_contents!(
        character["<C-f>"] = CharKeymapEntry::Char('f'),
        keymap_u32["6G"] = U32KeymapEntry::U32(65),
        motions["<C-f>"] = MotionEnumPreset::FindChar(FindCharPreset {
            direction: PresetStructField::FromKeystrokes,
            ch: PresetStructField::FromKeystrokes,
        }),
        motions["<C-l>"] = MotionEnumPreset::FindChar(FindCharPreset {
            direction: PresetStructField::Preset(DirectionFree {
                rows: 0,
                columns: 1
            }),
            ch: PresetStructField::FromKeystrokes,
        }),
        motions["<C-h>"] = MotionEnumPreset::FindChar(FindCharPreset {
            direction: PresetStructField::Preset(DirectionFree {
                rows: 0,
                columns: -1
            }),
            ch: PresetStructField::Preset(CharKeymapEntry::Char('@')),
        }),
        directions["l"] = DirectionFree {
            rows: 0,
            columns: 1
        },
        // motions["f"] = MotionEnumPreset::FixedNumberOfCells(FixedNumberOfCellsPreset {
        //     direction: PresetStructField::FromKeystrokes,
        //     number_of_cells: 1,
        //     jump: PresetStructField::FromKeystrokes,
        // }),
        canvas_iteration_jumps["n"] = CanvasIterationJump::NoJump,
        canvas_iteration_jumps["d"] = CanvasIterationJump::Diagonals,
        canvas_iteration_jumps["s"] = CanvasIterationJump::DirectionAsStride,
    );
    macro_rules! keystroke_parsing {
        ($($keystrokes:expr => $expected:expr,)*) => {
            $({
                // Assign expected value to result variable to enable type inference in next statement.
                #[allow(unused_assignments)]
                let mut result = $expected;
                result = <_>::from_keystrokes(
                    &mut KeystrokeSequence::try_from($keystrokes.to_string())
                        .unwrap()
                        .iter(),
                    &config,
                )
                .unwrap();
                assert_eq!(result, $expected,);
            })*
        };
    }
    keystroke_parsing!(
        "k" => MotionEnum::FixedNumberOfCells(FixedNumberOfCells {
           direction: DirectionFree {
               rows: -1,
               columns: 0
           },
           number_of_cells: 1,
           jump: CanvasIterationJump::DirectionAsStride,
        }),
        "l" => MotionEnum::FixedNumberOfCells(FixedNumberOfCells {
           direction: DirectionFree {
               rows: 0,
               columns: 1
           },
           number_of_cells: 1,
           jump: CanvasIterationJump::DirectionAsStride,
        }),
        "h" => 'h',
        "<C-e>" => 'e',
        "6G" => 65u32,
        "tl" => MotionEnum::FixedNumberOfCells(
           FixedNumberOfCells {
               direction: DirectionFree {
                   rows: 0,
                   columns: 1,
               },
               number_of_cells: 1,
               jump: CanvasIterationJump::DirectionAsStride,
           }
        ),
        // "fl" => MotionEnum::FixedNumberOfCells(
        //     FixedNumberOfCells {
        //         direction: DirectionFree {
        //             rows: 0,
        //             columns: 1,
        //         },
        //         number_of_cells: 1,
        //         jump: CanvasIterationJump::DirectionAsStride,
        //     }
        // ),
        "<C-f>l" => MotionEnum::FindChar(
            FindChar {
                direction: DirectionFree {
                    rows: 0,
                    columns: 1,
                },
                ch: 'l',
            }
        ),
    );
    // let motion = MotionEnum::from_keystrokes(
    //     &mut KeystrokeSequence::try_from("k".to_string()).unwrap().iter(),
    //     &config,
    // )
    // .unwrap();
    // assert_eq!(
    //     motion,
    //     MotionEnum::FixedNumberOfCells(FixedNumberOfCells {
    //         direction: DirectionFree {
    //             rows: -1,
    //             columns: 0
    //         },
    //         number_of_cells: 1,
    //         jump: CanvasIterationJump::NoJump,
    //     })
    // );
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
