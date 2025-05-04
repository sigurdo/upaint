use crate::canvas::raw::operations::CanvasDiff;
use crate::canvas::raw::operations::CanvasModification;
use crate::canvas::Canvas;

pub trait CanvasAction {
    fn get_modifications(&self, canvas: &Canvas) -> impl Iterator<Item = CanvasModification>;
}
