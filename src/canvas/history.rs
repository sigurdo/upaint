use std::collections::LinkedList;
use std::mem;

use crate::{config::Config, file_formats::FileFormat, ErrorCustom, ResultCustom};

use super::raw::{
    ansi_import::AnsiImportError, operations::CanvasModification, rendering::CanvasWidget, Canvas,
};

#[derive(Debug, Default, Clone)]
pub struct CanvasCommit {
    revision: u64,
    modifications: Vec<CanvasModification>,
}

#[derive(Debug, Default, Clone)]
pub struct VersionControlledCanvas {
    initial: Canvas,
    current: Canvas,
    commits: LinkedList<CanvasCommit>,
    commits_unapplied: LinkedList<CanvasCommit>,
    staging_area: Vec<CanvasModification>,
    revision_counter: u64,
}

impl VersionControlledCanvas {
    pub fn delete_history(&mut self) -> &mut Self {
        self.initial = self.current.clone();
        self.commits = LinkedList::new();
        self.commits_unapplied = LinkedList::new();
        self.revision_counter = 0;
        self
    }

    fn apply_modifications(&mut self, modifications: &Vec<CanvasModification>) {
        for modification in modifications {
            self.current.apply_operation(modification);
        }
    }

    /// Rebuilds `self.cells` from `self.cells_initial` by applying all commits in `self.commits`
    fn rebuild(&mut self) {
        self.current = self.initial.clone();
        for commit in self.commits.clone() {
            self.apply_modifications(&commit.modifications);
        }
    }

    /// Creates a commit from the provided modifications, but doesn't apply it, as it is assumed
    /// that the modifications are already applied to the current canvas.
    fn create_applied_commit(&mut self, modifications: Vec<CanvasModification>) {
        self.revision_counter += 1;
        let commit = CanvasCommit {
            revision: self.revision_counter,
            modifications,
        };
        self.commits.push_back(commit);
        self.commits_unapplied = LinkedList::new();
    }

    /// Creates and applies a new commit from provided modifications.
    /// Modifications in staging area will be committed first if any.
    pub fn create_commit(&mut self, modifications: Vec<CanvasModification>) -> &mut Self {
        self.commit_staged();
        self.apply_modifications(&modifications);
        self.create_applied_commit(modifications);
        self
    }

    /// Panics if no latest commit exists in canvas
    pub fn amend(&mut self, operations: Vec<CanvasModification>) -> &mut Self {
        let Some(commit_latest) = self.commits.iter_mut().next_back() else {
            panic!("No commits exist in UndoRedoCanvas.amend()")
        };
        for operation in operations {
            self.current.apply_operation(&operation);
            commit_latest.modifications.push(operation);
        }
        self
    }

    /// Adds a modification to the staging area
    pub fn stage(&mut self, modification: CanvasModification) {
        self.current.apply_operation(&modification);
        self.staging_area.push(modification);
    }

    pub fn clear_staged(&mut self) {
        self.staging_area = Vec::new();
        self.rebuild();
    }

    /// Creates a commit from modifications in staging area
    pub fn commit_staged(&mut self) {
        if self.staging_area.len() > 0 {
            let staged = mem::replace(&mut self.staging_area, Vec::new());
            self.create_applied_commit(staged);
        }
    }

    pub fn undo(&mut self) {
        if let Some(last_commit) = self.commits.pop_back() {
            self.commits_unapplied.push_front(last_commit);
            self.rebuild();
        }
    }

    pub fn redo(&mut self) {
        if let Some(next_commit) = self.commits_unapplied.pop_front() {
            self.apply_modifications(&next_commit.modifications);
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

    pub fn export(&self, format: FileFormat) -> ResultCustom<String> {
        self.current.export(format)
    }

    pub fn export_lossy(&self, format: FileFormat) -> ResultCustom<String> {
        self.current.export_lossy(format)
    }

    pub fn from_ansi(ansi: String) -> ResultCustom<Self>
    where
        Self: Sized,
    {
        match Canvas::from_ansi(ansi) {
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

    pub fn widget<'a>(&'a self, config: &'a Config) -> CanvasWidget {
        CanvasWidget::from_canvas(&self.current, config)
    }

    pub fn raw(&self) -> &Canvas {
        &self.current
    }
}
