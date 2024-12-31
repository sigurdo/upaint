use std::collections::LinkedList;
use std::mem;

use crate::canvas::raw::operations::CanvasDiff;
use crate::canvas::raw::operations::CanvasDiffBuilder;
use crate::{config::Config, file_formats::FileFormat, ErrorCustom, ResultCustom};

use super::raw::{
    ansi_import::AnsiImportError, operations::CanvasModification, rendering::CanvasWidget, Canvas,
};

#[derive(Debug, Default, Clone)]
pub struct CanvasCommit {
    revision: u64,
    diff: CanvasDiff,
}

#[derive(Debug, Default, Clone)]
pub struct VersionControlledCanvas {
    initial: Canvas,
    current: Canvas,
    commits: LinkedList<CanvasCommit>,
    commits_unapplied: LinkedList<CanvasCommit>,
    staging_area: CanvasDiffBuilder,
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

    /// Creates a commit from the provided modifications, but doesn't apply it, as it is assumed
    /// that the modifications are already applied to the current canvas.
    fn create_applied_commit(&mut self, diff: CanvasDiff) {
        self.revision_counter += 1;
        let commit = CanvasCommit {
            revision: self.revision_counter,
            diff,
        };
        self.commits.push_back(commit);
        self.commits_unapplied = LinkedList::new();
    }

    /// Creates and applies a new commit from provided modifications.
    /// Modifications in staging area will be committed first if any.
    pub fn create_commit(&mut self, modifications: Vec<CanvasModification>) -> &mut Self {
        self.commit_staged();
        let mut diff =
            CanvasDiffBuilder::from_modifications(modifications, &self.current).serialize();
        self.current.apply_diff(&mut diff);
        self.create_applied_commit(diff);
        self
    }

    /// Adds a modification to the staging area
    pub fn stage(&mut self, modification: CanvasModification) {
        let mut diff = CanvasDiffBuilder::from_modifications(vec![modification], &self.current);
        self.current.apply_diff_builder(&mut diff);
        // overwrite must be false, because the staging area contains the reversing diffs, which
        // must be preserved to revert to the original state.
        self.staging_area.add_diff(diff, false);
    }

    pub fn clear_staged(&mut self) {
        self.current.apply_diff_builder(&mut self.staging_area);
        self.staging_area = CanvasDiffBuilder::default();
    }

    /// Creates a commit from modifications in staging area
    pub fn commit_staged(&mut self) {
        if !self.staging_area.entries.is_empty() {
            let staged = mem::take(&mut self.staging_area).serialize();
            self.create_applied_commit(staged);
        }
    }

    pub fn undo(&mut self) {
        if let Some(mut last_commit) = self.commits.pop_back() {
            self.current.apply_diff(&mut last_commit.diff);
            self.commits_unapplied.push_front(last_commit);
        }
    }

    pub fn redo(&mut self) {
        if let Some(mut next_commit) = self.commits_unapplied.pop_front() {
            self.current.apply_diff(&mut next_commit.diff);
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

    pub fn widget<'a>(&'a self, config: &'a Config) -> CanvasWidget<'a> {
        CanvasWidget::from_canvas(&self.current, config)
    }

    pub fn raw(&self) -> &Canvas {
        &self.current
    }
}
