use crate::canvas::raw::cell_map::BTreeCellMap;
use crate::canvas::raw::CanvasCell;
use crate::canvas::raw::CanvasIndex;
use crate::Axis;
use crate::RotationDirection;
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;

fn cell_map_insert_option(
    cells: &mut BTreeCellMap,
    index: CanvasIndex,
    cell: Option<CanvasCell>,
) -> Option<CanvasCell> {
    if let Some(cell) = cell {
        cells.insert(index, cell)
    } else {
        cells.remove(&index)
    }
}

/// Swaps indices around. The value at the first index is moved to the second, the second to third
/// and so on. Finally, the last is moved to the first.
fn swap_indices_iter(
    cells: &mut BTreeCellMap,
    character_swaps: &CharacterSwapMap,
    first: CanvasIndex,
    mut others: impl Iterator<Item = CanvasIndex>,
) {
    let mut previous_cell = cells.remove(&first);
    while let Some(index) = others.next() {
        apply_character_swap_map_to_cell_option(previous_cell.as_mut(), character_swaps);
        previous_cell = cell_map_insert_option(cells, index, previous_cell);
    }
    cell_map_insert_option(cells, first, previous_cell);
}

macro_rules! swap_indices {
    ($cells:expr, $character_swaps:expr, $first:expr, $($others:expr,)*) => {
        swap_indices_iter($cells, $character_swaps, $first, vec![$($others,)*].into_iter())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharacterSwapMap(HashMap<char, char>);

impl<'de> Deserialize<'de> for CharacterSwapMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let swaps = HashMap::<char, char>::deserialize(deserializer)?;
        let mut result = HashMap::new();
        for (a, b) in swaps {
            result.entry(a).or_insert(b);
            result.entry(b).or_insert(a);
        }
        Ok(Self(result))
    }
}

fn apply_character_swap_map_to_cell_option(cell: Option<&mut CanvasCell>, map: &CharacterSwapMap) {
    if let Some(cell) = cell {
        if let Some(swap) = map.0.get(&cell.character) {
            cell.character = *swap;
        }
    }
}

pub fn map_from_swap_pairs(swaps: impl Iterator<Item = (char, char)>) -> CharacterSwapMap {
    let mut map = HashMap::new();
    for (a, b) in swaps {
        map.insert(a, b);
        map.insert(b, a);
    }
    CharacterSwapMap(map)
}

fn mirror_index(index: i16, mirror: i16) -> i16 {
    index - 2 * (index - mirror)
}

/// Given axis is not the mirror line, but rather the axis along which the mirroring is performed.
pub fn mirror_cells(cells: &mut BTreeCellMap, axis: Axis, mirror: i16, swaps: &CharacterSwapMap) {
    let mut indices_mirrored = HashSet::new();
    for index in cells.keys().cloned().collect::<Vec<_>>() {
        let index_mirrored = if axis == Axis::X {
            (index.0, mirror_index(index.1, mirror))
        } else {
            (mirror_index(index.0, mirror), index.1)
        };
        if indices_mirrored.contains(&index) {
            continue;
        };
        if index == index_mirrored {
            apply_character_swap_map_to_cell_option(cells.get_mut(&index), swaps);
            indices_mirrored.insert(index);
            continue;
        }
        // let mut index_mirrored = index.clone();
        swap_indices!(cells, &swaps, index, index_mirrored,);
        indices_mirrored.insert(index);
        indices_mirrored.insert(index_mirrored);
    }
}

fn rotate_index(
    index: CanvasIndex,
    rotation_index: CanvasIndex,
    direction: RotationDirection,
) -> CanvasIndex {
    if direction == RotationDirection::Clockwise {
        (rotation_index.0 + index.1, rotation_index.1 - index.0)
    } else {
        (rotation_index.0 - index.1, rotation_index.1 + index.0)
    }
}

/// Given axis is not the mirror line, but rather the axis along which the mirroring is performed.
pub fn rotate_cells(
    cells: &mut BTreeCellMap,
    rotation_index: CanvasIndex,
    direction: RotationDirection,
    swaps: &CharacterSwapMap,
) {
    let mut indices_rotated = HashSet::new();
    for index in cells.keys().cloned().collect::<Vec<_>>() {
        if indices_rotated.contains(&index) {
            continue;
        };
        let index_b = rotate_index(index, rotation_index, direction);
        let index_c = rotate_index(index_b, rotation_index, direction);
        let index_d = rotate_index(index_c, rotation_index, direction);
        swap_indices!(cells, swaps, index, index_b, index_c, index_d,);
        indices_rotated.insert(index);
        indices_rotated.insert(index_b);
        indices_rotated.insert(index_c);
        indices_rotated.insert(index_d);
    }
}
