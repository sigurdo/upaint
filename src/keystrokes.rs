use crate::config::Config;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use enum_dispatch::enum_dispatch;
use keystrokes_parsing::Presetable;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::actions::Action;
use crate::canvas::raw::continuous_region::ContinuousRegionRelativeType;
use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::raw::iter::WordBoundaryType;
use crate::canvas::raw::CellContentType;
use crate::selections::SelectionSlotSpecification;
use crate::yank_slots::YankSlotSpecification;
use crate::DirectionFree;
use crate::Ground;
use crate::ProgramState;
use upaint_derive::Preset;

// #[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize)]
pub type ColorSlot = char;
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Presetable)]
pub enum ColorOrSlot {
    Slot(ColorSlot),
    #[serde(untagged)]
    Color(Color),
}

impl ColorOrSlot {
    pub fn as_color(&self, program_state: &ProgramState) -> Option<Color> {
        match self {
            Self::Color(color) => Some(*color),
            Self::Slot(slot) => program_state.color_slots.get(&slot).copied(),
        }
    }
}
impl Default for ColorOrSlot {
    fn default() -> Self {
        Self::Slot('a')
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColorOrSlotSpecification {
    Active,
    #[serde(untagged)]
    Specific(ColorOrSlot),
}
impl ColorOrSlotSpecification {
    pub fn as_color_or_slot(&self, program_state: &ProgramState) -> ColorOrSlot {
        match self {
            Self::Active => program_state.color_or_slot_active,
            Self::Specific(color_or_slot) => *color_or_slot,
        }
    }
}
