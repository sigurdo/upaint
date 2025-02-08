use super::Keymaps;

use crate::config::ConfigInputMode;
use crate::input_mode::InputMode;
use crate::ProgramState;
use std::collections::HashSet;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum BaseKeymapsIter<'a> {
    Current {
        input_mode: &'a InputMode,
        config_input_mode: &'a ConfigInputMode,
        bases_recursed: HashSet<&'a InputMode>,
        program_state: &'a ProgramState,
    },
    Recursing {
        self_recursing: Box<BaseKeymapsIter<'a>>,
        bases_to_recurse: core::slice::Iter<'a, InputMode>,
        program_state: &'a ProgramState,
    },
    Finished {
        bases_recursed: HashSet<&'a InputMode>,
    },
}

impl<'a> BaseKeymapsIter<'a> {
    pub fn new(program_state: &'a ProgramState) -> Self {
        let input_mode = &program_state.input_mode;
        let bases_recursed = HashSet::new();
        if let Some(config_input_mode) = program_state.config.input_mode.get(input_mode) {
            Self::Current {
                input_mode,
                config_input_mode,
                bases_recursed,
                program_state,
            }
        } else {
            Self::Finished { bases_recursed }
        }
    }
    fn recurse_next_base(
        mut bases_to_recurse: core::slice::Iter<'a, InputMode>,
        bases_recursed: HashSet<&'a InputMode>,
        program_state: &'a ProgramState,
    ) -> Self {
        while let Some(base) = bases_to_recurse.next() {
            if let Some(config_base) = program_state.config.input_mode.get(base) {
                return BaseKeymapsIter::Recursing {
                    self_recursing: Box::new(Self::Current {
                        input_mode: base,
                        config_input_mode: config_base,
                        bases_recursed,
                        program_state,
                    }),
                    bases_to_recurse,
                    program_state,
                };
            }
        }
        Self::Finished { bases_recursed }
    }
    fn take_bases_recursed(self) -> HashSet<&'a InputMode> {
        match self {
            Self::Current { bases_recursed, .. } => bases_recursed,
            Self::Recursing { self_recursing, .. } => self_recursing.take_bases_recursed(),
            Self::Finished { bases_recursed } => bases_recursed,
        }
    }
}
impl<'a> Iterator for BaseKeymapsIter<'a> {
    type Item = &'a Keymaps;
    fn next(&mut self) -> Option<&'a Keymaps> {
        let mut ret;
        let self_local = std::mem::replace(
            self,
            BaseKeymapsIter::Finished {
                bases_recursed: HashSet::new(),
            },
        );
        *self = match self_local {
            Self::Current {
                input_mode,
                config_input_mode,
                mut bases_recursed,
                program_state,
            } => {
                if bases_recursed.insert(input_mode) {
                    let bases_recursed = bases_recursed;
                    ret = Some(&config_input_mode.keymaps);
                    let bases_to_recurse = config_input_mode.base_keymaps.iter();
                    Self::recurse_next_base(bases_to_recurse, bases_recursed, program_state)
                } else {
                    ret = None;
                    Self::Finished { bases_recursed }
                }
            }
            Self::Recursing {
                mut self_recursing,
                bases_to_recurse,
                program_state,
            } => {
                ret = self_recursing.next();
                if ret.is_none() {
                    let bases_recursed = self_recursing.take_bases_recursed();
                    let mut new_self =
                        Self::recurse_next_base(bases_to_recurse, bases_recursed, program_state);
                    ret = new_self.next();
                    new_self
                } else {
                    Self::Recursing {
                        self_recursing,
                        bases_to_recurse,
                        program_state,
                    }
                }
            }
            other => {
                ret = None;
                other
            }
        };
        ret
    }
}
