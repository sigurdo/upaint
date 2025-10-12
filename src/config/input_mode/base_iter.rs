use super::keymaps::Keymaps;
use std::marker::PhantomData;

use crate::config::mouse_actions::MouseActions;
use crate::config::ConfigInputMode;
use crate::input_mode::InputMode;
use crate::input_mode::InputModeHandler;
use crate::ProgramState;
use std::collections::HashSet;
use std::fmt::Debug;

pub trait InputModeRecurseResult<'a>: Sized {
    fn get_field(config_input_mode: &'a ConfigInputMode) -> Option<&'a Self>;
}

#[derive(Debug, Clone)]
pub enum InputModeRecursor<'a, R: InputModeRecurseResult<'a>> {
    Current {
        input_mode: &'a InputMode,
        config_input_mode: &'a ConfigInputMode,
        bases_recursed: HashSet<&'a InputMode>,
        program_state: &'a ProgramState,
    },
    Recursing {
        self_recursing: Box<Self>,
        bases_to_recurse: core::slice::Iter<'a, InputMode>,
        program_state: &'a ProgramState,
    },
    Finished {
        bases_recursed: HashSet<&'a InputMode>,
        phantom: PhantomData<R>,
    },
}

pub type BaseKeymapsIter<'a> = InputModeRecursor<'a, Keymaps>;

impl<'a> InputModeRecurseResult<'a> for Keymaps {
    fn get_field(config_input_mode: &'a ConfigInputMode) -> Option<&'a Self> {
        config_input_mode.keymaps.as_ref()
    }
}

pub type BaseMouseActionsIter<'a> = InputModeRecursor<'a, MouseActions>;

impl<'a> InputModeRecurseResult<'a> for MouseActions {
    fn get_field(config_input_mode: &'a ConfigInputMode) -> Option<&'a Self> {
        config_input_mode.mouse_actions.as_ref()
    }
}

pub type BaseInputModeHandlerIter<'a> = InputModeRecursor<'a, InputModeHandler>;

impl<'a> InputModeRecurseResult<'a> for InputModeHandler {
    fn get_field(config_input_mode: &'a ConfigInputMode) -> Option<&'a Self> {
        config_input_mode.handler.as_ref()
    }
}

impl<'a, R: InputModeRecurseResult<'a>> InputModeRecursor<'a, R> {
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
            Self::Finished {
                bases_recursed,
                phantom: PhantomData,
            }
        }
    }
    fn recurse_next_base(
        mut bases_to_recurse: core::slice::Iter<'a, InputMode>,
        bases_recursed: HashSet<&'a InputMode>,
        program_state: &'a ProgramState,
    ) -> Self {
        while let Some(base) = bases_to_recurse.next() {
            if let Some(config_base) = program_state.config.input_mode.get(base) {
                return InputModeRecursor::Recursing {
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
        Self::Finished {
            bases_recursed,
            phantom: PhantomData,
        }
    }
    fn take_bases_recursed(self) -> HashSet<&'a InputMode> {
        match self {
            Self::Current { bases_recursed, .. } => bases_recursed,
            Self::Recursing { self_recursing, .. } => self_recursing.take_bases_recursed(),
            Self::Finished { bases_recursed, .. } => bases_recursed,
        }
    }
}
impl<'a, R: InputModeRecurseResult<'a> + 'a> Iterator for InputModeRecursor<'a, R> {
    type Item = &'a R;
    fn next(&mut self) -> Option<&'a R> {
        let mut ret: Option<&'a R>;
        let self_local = std::mem::replace(
            self,
            InputModeRecursor::Finished {
                bases_recursed: HashSet::new(),
                phantom: PhantomData,
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
                    ret = <R as InputModeRecurseResult>::get_field(config_input_mode);
                    let mut new_self = if let Some(extends) = &config_input_mode.extends {
                        Self::recurse_next_base(extends.iter(), bases_recursed, program_state)
                    } else {
                        Self::Finished {
                            bases_recursed,
                            phantom: PhantomData,
                        }
                    };
                    if ret.is_none() {
                        ret = new_self.next();
                    }
                    new_self
                } else {
                    ret = None;
                    Self::Finished {
                        bases_recursed,
                        phantom: PhantomData,
                    }
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
