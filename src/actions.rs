use crate::ProgramState;
use enum_dispatch::enum_dispatch;

pub mod session;

#[enum_dispatch]
pub trait Action: std::fmt::Debug {
    fn execute(&self, program_state: &mut ProgramState);
}

// Contains Ok(()) or Err(error_message)
type ExecuteActionResult = Result<(), String>;

pub trait FallibleAction {
    fn try_execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult;
}

impl<T> FallibleAction for T
where
    T: Action,
{
    fn try_execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult {
        Ok(self.execute(program_state))
    }
}
