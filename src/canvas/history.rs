use std::collections::LinkedList;

use crate::{ErrorCustom, ResultCustom};

use super::raw::{
    ansi_import::AnsiImportError, operations::CanvasOperation, rendering::CanvasWidget, RawCanvas,
};

#[derive(Debug, Default, Clone)]
pub struct CanvasCommit {
    revision: u64,
    operations: Vec<CanvasOperation>,
}

#[derive(Debug, Default, Clone)]
pub struct UndoRedoCanvas {
    initial: RawCanvas,
    current: RawCanvas,
    commits: LinkedList<CanvasCommit>,
    commits_unapplied: LinkedList<CanvasCommit>,
    revision_counter: u64,
}

impl UndoRedoCanvas {
    pub fn delete_history(&mut self) -> &mut Self {
        self.initial = self.current.clone();
        self.commits = LinkedList::new();
        self.commits_unapplied = LinkedList::new();
        self.revision_counter = 0;
        self
    }

    fn apply_commit(&mut self, commit: &CanvasCommit) {
        for operation in &commit.operations {
            self.current.apply_operation(operation);
        }
    }

    /// Rebuilds `self.cells` from `self.cells_initial` by applying all commits in `self.commits`
    fn rebuild(&mut self) {
        self.current = self.initial.clone();
        for commit in self.commits.clone() {
            self.apply_commit(&commit);
        }
    }

    pub fn create_commit(&mut self, operations: Vec<CanvasOperation>) -> &mut Self {
        self.revision_counter += 1;
        let commit = CanvasCommit {
            revision: self.revision_counter,
            operations: operations,
        };
        self.apply_commit(&commit);
        self.commits.push_back(commit);
        self.commits_unapplied = LinkedList::new();
        self
    }

    pub fn undo(&mut self) {
        if let Some(last_commit) = self.commits.pop_back() {
            self.commits_unapplied.push_front(last_commit);
            self.rebuild();
        }
    }

    pub fn redo(&mut self) {
        if let Some(next_commit) = self.commits_unapplied.pop_front() {
            self.apply_commit(&next_commit);
            self.commits.push_back(next_commit);
        }
    }

    pub fn get_current_revision(&self) -> u64 {
        if let Some(last_commit) = self.commits.back() {
            last_commit.revision
        } else {
            0
        }
    }

    pub fn to_ansi(&self) -> ResultCustom<String> {
        self.current.to_ansi(true)
    }

    pub fn from_ansi(ansi: String) -> ResultCustom<Self>
    where
        Self: Sized,
    {
        match RawCanvas::from_ansi(ansi) {
            Ok(canvas) => {
                let mut result = Self::default();
                result.initial = canvas.clone();
                result.current = canvas;
                Ok(result)
            }
            Err(e) => match e {
                AnsiImportError::IllegalCharacter((row, column)) => {
                    Err(ErrorCustom::String(format!(
                        "ANSI file contains an illegal character on line {row}, column {column}"
                    )))
                }
                AnsiImportError::IllegalEscapeSequence((row, column)) => {
                    Err(ErrorCustom::String(format!(
                        "ANSI file contains an illegal escape sequence on line {row}, collumn {column}"
                    )))
                }
                AnsiImportError::UnfinishedEscapeSequence((row, column)) => {
                    Err(ErrorCustom::String(format!(
                        "ANSI file contains an unfinished escape sequence on line {row}, collumn {column}"
                    )))
                }
                AnsiImportError::BadSgrSequence((row, column)) => {
                    Err(ErrorCustom::String(format!(
                        "ANSI file contains a bad SGR escape sequence on line {row}, collumn {column}"
                    )))
                }
                AnsiImportError::UnsupportedSgrSequence((row, column)) => {
                    Err(ErrorCustom::String(format!(
                        "ANSI file contains an unsupported SGR escape sequence on line {row}, collumn {column}"
                    )))
                }
            },
        }
    }

    pub fn widget(&self) -> CanvasWidget {
        CanvasWidget::from_canvas(&self.current)
    }

    pub fn raw(&self) -> &RawCanvas {
        &self.current
    }
}
