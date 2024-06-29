#[test]
fn test_find_cell_exit() {
    use nalgebra as na;
    use crate::DirectionFree;
    let tests = vec![
        ((0.0, 0.0), (0, 1), (0.0, 0.5)),
        ((0.0, 0.0), (1, 1), (0.5, 0.5)),
        ((0.0, 0.0), (2, 2), (0.5, 0.5)),
        ((0.0, 0.0), (1, -1), (0.5, -0.5)),
        ((0.0, 0.0), (2, 1), (0.5, 0.25)),
        ((0.0, 0.0), (-4, -1), (-0.5, -0.125)),
        ((-0.5, 0.0), (1, 1), (0.0, 0.5)),
        ((-0.5, 0.5), (1, 1), (-0.5, 0.5)),
        ((0.3, 0.5), (2, 1), (0.3, 0.5)),
        ((-0.5, 0.5), (2, -1), (0.5, 0.0)),
        ((-0.5, 0.0), (2, -1), (0.5, -0.5)),
    ];
    for ((x0, y0), (dx, dy), (x1, y1)) in tests {
        let start = na::Vector2::new(x0, y0);
        let direction = DirectionFree::new(dy, dx).unwrap();
        let exit = super::find_cell_exit(start, direction);
        let expected = na::Vector2::new(x1, y1);
        assert_eq!(exit, expected);
    }
}

#[test]
fn test_canvas_index_iterator_infinite() {
    use crate::DirectionFree;
    use super::CanvasIndexIteratorInfinite;
    let tests = vec![
        ((0, 0), (0, 1), vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)]),
        ((0, 0), (1, 1), vec![(0, 0), (0, 1), (1, 1), (1, 2), (2, 2)]),
        ((0, 0), (1, 2), vec![(0, 0), (0, 1), (1, 1), (1, 2), (1, 3), (2, 3), (2, 4), (2, 5), (3, 5), (3, 6)]),
        ((0, 0), (-1, 2), vec![(0, 0), (0, 1), (-1, 1), (-1, 2), (-1, 3), (-2, 3), (-2, 4), (-2, 5), (-3, 5), (-3, 6)]),
        ((0, 0), (-1, -2), vec![(0, 0), (0, -1), (-1, -1), (-1, -2), (-1, -3), (-2, -3), (-2, -4), (-2, -5), (-3, -5), (-3, -6)]),
        ((0, 0), (1, -2), vec![(0, 0), (0, -1), (1, -1), (1, -2), (1, -3), (2, -3), (2, -4), (2, -5), (3, -5), (3, -6)]),
        ((0, 0), (2, -1), vec![(0, 0), (1, 0), (1, -1), (2, -1), (3, -1), (3, -2), (4, -2), (5, -2), (5, -3), (6, -3)]),
        ((10, 100), (1, 1), vec![(10, 100), (10, 101), (11, 101), (11, 102), (12, 102)]),
    ];
    for (start, (direction_rows, direction_columns), indices) in tests {
        let mut it = CanvasIndexIteratorInfinite::new(start, DirectionFree::new(direction_rows, direction_columns).unwrap());
        for expected in indices {
            let actual = it.next().unwrap();
            assert_eq!(actual, expected);
        }
    }
}

#[test]
fn test_canvas_index_iterator() {
    use crate::{DirectionFree, Direction};
    use super::{CanvasIndexIterator, StopCondition, RawCanvas, WordBoundaryType};
    let tests = vec![
        ("    4567    abcd", (0, 0), DirectionFree::from(Direction::Right), StopCondition::SecondCell, (0, 1)),
        ("    4567    abcd", (0, 0), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::ANY), (0, 4)),
        ("    4567    abcd", (0, 4), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::ANY), (0, 7)),
        ("    4567    abcd", (0, 6), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::ANY), (0, 7)),
        ("    4567    abcd", (0, 7), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::ANY), (0, 12)),
        ("    4567    abcd", (0, 9), DirectionFree::from(Direction::Left), StopCondition::WordBoundary(WordBoundaryType::ANY), (0, 7)),
        ("    4567    abcd", (0, 0), DirectionFree::from(Direction::Left), StopCondition::WordBoundary(WordBoundaryType::ANY), (0, 0)),
        ("    4567    abcd", (0, 6), DirectionFree::from(Direction::Up), StopCondition::WordBoundary(WordBoundaryType::ANY), (0, 6)),
        ("    4567    abcd", (-3, 6), DirectionFree::from(Direction::Up), StopCondition::WordBoundary(WordBoundaryType::ANY), (-3, 6)),
        ("    4567    abcd", (-3, 6), DirectionFree::from(Direction::Down), StopCondition::WordBoundary(WordBoundaryType::ANY), (0, 6)),
        ("    4567    abcd", (-3, 3), DirectionFree::from(Direction::Down), StopCondition::WordBoundary(WordBoundaryType::ANY), (-3, 3)),
        ("    4567    abcd", (0, 0), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::START), (0, 4)),
        ("    4567    abcd", (0, 4), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::START), (0, 12)),
        ("    4567    abcd", (0, 12), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::START), (0, 12)),
        ("    4567    abcd", (0, 0), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::END), (0, 7)),
        ("    4567    abcd", (0, 4), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::END), (0, 7)),
        ("    4567    abcd", (0, 12), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::END), (0, 15)),
        ("    4567    abcd", (0, 4), DirectionFree::from(Direction::Right), StopCondition::CharacterChange, (0, 5)),
        ("    4567    abcd", (0, 7), DirectionFree::from(Direction::Right), StopCondition::CharacterChange, (0, 8)),
        ("    4567    abcd", (0, 8), DirectionFree::from(Direction::Right), StopCondition::CharacterChange, (0, 12)),
        ("    4567    abcd", (0, 6), DirectionFree::from(Direction::Up), StopCondition::CharacterChange, (-1, 6)),
        ("    4567    abcd", (-3, 6), DirectionFree::from(Direction::Up), StopCondition::CharacterChange, (-3, 6)),
        ("    4567    abcd", (-3, 6), DirectionFree::from(Direction::Down), StopCondition::CharacterChange, (0, 6)),
        ("    4567    abcd", (-3, 3), DirectionFree::from(Direction::Down), StopCondition::CharacterChange, (-3, 3)),
    ];
    for (txt, start, direction, stop, end) in tests {
        let mut canvas = RawCanvas::from_txt(txt.to_string()).unwrap();
        let it = CanvasIndexIterator::new(&mut canvas, start, direction, stop);
        assert_eq!(it.last(), Some(end));
    }
}

