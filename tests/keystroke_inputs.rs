use keystrokes_parsing::FromKeystrokes;
use keystrokes_parsing::KeystrokeSequence;
use upaint::actions;
use upaint::actions::ActionEnum;
use upaint::canvas::raw::iter::CanvasIterationJump;
use upaint::config::load_default_config;
use upaint::keystrokes::ColorOrSlotSpecification;
use upaint::keystrokes::Count;
use upaint::motions;
use upaint::motions::MotionEnum;
use upaint::motions::MotionRepeat;
use upaint::motions::MotionRepeatEnum;
use upaint::operators;
use upaint::operators::OperatorEnum;
use upaint::DirectionFree;
use upaint::Ground;
use upaint::ProgramState;

#[test]
pub fn test() {
    let config = load_default_config();
    // let config: Config = toml::from_str(CONFIG_TOML).unwrap();
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
    // keymaps_contents!(
    //     character["<C-f>"] = CharKeymapEntry::Char('f'),
    //     keymap_u32["6G"] = UnsignedIntegerKeymapEntry::Number(65),
    //     motions["<C-f>"] = MotionEnumPreset::FindChar(FindCharPreset {
    //         direction: PresetStructField::FromKeystrokes,
    //         ch: PresetStructField::FromKeystrokes,
    //     }),
    //     motions["<C-l>"] = MotionEnumPreset::FindChar(FindCharPreset {
    //         direction: PresetStructField::Preset(DirectionFree {
    //             rows: 0,
    //             columns: 1
    //         }),
    //         ch: PresetStructField::FromKeystrokes,
    //     }),
    //     motions["<C-h>"] = MotionEnumPreset::FindChar(FindCharPreset {
    //         direction: PresetStructField::Preset(DirectionFree {
    //             rows: 0,
    //             columns: -1
    //         }),
    //         ch: PresetStructField::Preset(CharKeymapEntry::Char('@')),
    //     }),
    //     directions["l"] = DirectionFree {
    //         rows: 0,
    //         columns: 1
    //     },
    //     // motions["f"] = MotionEnumPreset::FixedNumberOfCells(FixedNumberOfCellsPreset {
    //     //     direction: PresetStructField::FromKeystrokes,
    //     //     number_of_cells: 1,
    //     //     jump: PresetStructField::FromKeystrokes,
    //     // }),
    //     canvas_iteration_jumps["n"] = CanvasIterationJump::NoJump,
    //     canvas_iteration_jumps["d"] = CanvasIterationJump::Diagonals,
    //     canvas_iteration_jumps["s"] = CanvasIterationJump::DirectionAsStride,
    // );
    macro_rules! keystroke_parsing {
        ($($keystrokes:expr => $expected:expr,)*) => {
            $({
                // Assign expected value to result variable to enable type inference in next statement.
                #[allow(unused_assignments)]
                let mut result = $expected;
                let mut program_state = ProgramState::default();
                program_state.config = config.clone();
                result = <_>::from_keystrokes(
                    &mut KeystrokeSequence::try_from($keystrokes.to_string())
                        .unwrap()
                        .iter()
                        .peekable(),
                    &program_state,
                )
                .unwrap();
                assert_eq!(result, $expected,);
            })*
        };
    }
    keystroke_parsing!(
        "k" => MotionEnum::Repeat(MotionRepeat {
            count: Count(1),
            motion: MotionRepeatEnum::FixedNumberOfCells(motions::FixedNumberOfCells {
            direction: DirectionFree {
                rows: -1,
                columns: 0
            },
            jump: CanvasIterationJump::DirectionAsStride,
        })}),
        "o" => MotionEnum::Repeat(MotionRepeat {
            count: Count(1),
            motion: MotionRepeatEnum::FixedNumberOfCells(motions::FixedNumberOfCells {
                direction: DirectionFree {
                    rows: -1,
                    columns: 2
                },
                jump: CanvasIterationJump::DirectionAsStride,
            }),
        }),
        "flx" => MotionEnum::Repeat(MotionRepeat {
            count: Count(1),
            motion: MotionRepeatEnum::FindChar(motions::FindChar {
                direction: DirectionFree {
                    rows: 0,
                    columns: 1,
                },
                ch: 'x',
            }),
        }),
        "ch" => ActionEnum::Operation(actions::Operation {
            operator: OperatorEnum::Colorize(operators::Colorize {
                ground: Ground::Foreground,
                color: ColorOrSlotSpecification::Active,
            }),
            motion: MotionEnum::Repeat(MotionRepeat {
                count: Count(1),
                motion: MotionRepeatEnum::FixedNumberOfCells(motions::FixedNumberOfCells {
                    direction: DirectionFree {
                        rows: 0,
                        columns: -1,
                    },
                    jump: CanvasIterationJump::DirectionAsStride,
                }),
            }),
        }),
    );
}
