#[test]
fn test_canvas_index_iterator_infinite() {
    use super::CanvasIndexIteratorInfinite;
    use super::CanvasIterationJump;
    use crate::DirectionFree;
    #[rustfmt::skip]
    let tests = vec![
        ((0, 0), (0, 1), CanvasIterationJump::NoJump, vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)]),
        ((0, 0), (1, 1), CanvasIterationJump::NoJump, vec![(0, 0), (0, 1), (1, 1), (1, 2), (2, 2)]),
        ((0, 0), (1, 2), CanvasIterationJump::NoJump, vec![(0, 0), (0, 1), (1, 1), (1, 2), (1, 3), (2, 3), (2, 4), (2, 5), (3, 5), (3, 6)]),
        ((0, 0), (-1, 2), CanvasIterationJump::NoJump, vec![(0, 0), (0, 1), (-1, 1), (-1, 2), (-1, 3), (-2, 3), (-2, 4), (-2, 5), (-3, 5), (-3, 6)]),
        ((0, 0), (-1, -2), CanvasIterationJump::NoJump, vec![(0, 0), (0, -1), (-1, -1), (-1, -2), (-1, -3), (-2, -3), (-2, -4), (-2, -5), (-3, -5), (-3, -6)]),
        ((0, 0), (1, -2), CanvasIterationJump::NoJump, vec![(0, 0), (0, -1), (1, -1), (1, -2), (1, -3), (2, -3), (2, -4), (2, -5), (3, -5), (3, -6)]),
        ((0, 0), (2, -1), CanvasIterationJump::NoJump, vec![(0, 0), (1, 0), (1, -1), (2, -1), (3, -1), (3, -2), (4, -2), (5, -2), (5, -3), (6, -3)]),
        ((10, 100), (1, 1), CanvasIterationJump::NoJump, vec![(10, 100), (10, 101), (11, 101), (11, 102), (12, 102)]),
        ((0, 0), (0, 1), CanvasIterationJump::Diagonals, vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)]),
        ((0, 0), (1, 1), CanvasIterationJump::Diagonals, vec![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)]),
        ((0, 0), (-1, 1), CanvasIterationJump::Diagonals, vec![(0, 0), (-1, 1), (-2, 2), (-3, 3), (-4, 4)]),
        ((0, 0), (1, -2), CanvasIterationJump::Diagonals, vec![(0, 0), (0, -1), (1, -2), (1, -3), (2, -4)]),
        ((0, 0), (-1, -2), CanvasIterationJump::Diagonals, vec![(0, 0), (0, -1), (-1, -2), (-1, -3), (-2, -4)]),
        ((0, 0), (3, 2), CanvasIterationJump::Diagonals, vec![(0, 0), (1, 1), (2, 1), (3, 2), (4, 3), (5, 3)]),
    ];
    for (start, (direction_rows, direction_columns), jump, indices) in tests {
        let mut it = CanvasIndexIteratorInfinite::new(
            start,
            DirectionFree::new(direction_rows, direction_columns).unwrap(),
            jump,
        );
        it.go_backward();
        for expected in indices.iter() {
            let actual = it.go_forward();
            assert_eq!(actual, *expected);
        }
        it.go_forward();
        // Backwards iteration should produce same path
        for expected in indices.iter().rev() {
            let actual = it.go_backward();
            assert_eq!(actual, *expected);
        }
    }
}

#[test]
fn test_canvas_index_iterator() {
    use super::{
        Canvas, CanvasIndexIterator, CanvasIterationJump, StopCondition, WordBoundaryType,
    };
    use crate::{Direction, DirectionFree};
    #[rustfmt::skip]
    let tests = vec![
        ("    4567    abcd", (0, 0), DirectionFree::from(Direction::Right), StopCondition::Always, 1, (0, 1)),
        ("    4567    abcd", (1, 1), DirectionFree::from(Direction::Right), StopCondition::Always, 3, (1, 4)),
        ("    4567    abcd", (0, 0), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::ANY), 1, (0, 4)),
        ("    4567    abcd", (0, 4), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::ANY), 1, (0, 7)),
        ("    4567    abcd", (0, 6), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::ANY), 1, (0, 7)),
        ("    4567    abcd", (0, 7), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::ANY), 1, (0, 12)),
        ("    4567    abcd", (0, 9), DirectionFree::from(Direction::Left), StopCondition::WordBoundary(WordBoundaryType::ANY), 1, (0, 7)),
        ("    4567    abcd", (0, 0), DirectionFree::from(Direction::Left), StopCondition::WordBoundary(WordBoundaryType::ANY), 1, (0, 0)),
        ("    4567    abcd", (0, 6), DirectionFree::from(Direction::Up), StopCondition::WordBoundary(WordBoundaryType::ANY), 1, (0, 6)),
        ("    4567    abcd", (-3, 6), DirectionFree::from(Direction::Up), StopCondition::WordBoundary(WordBoundaryType::ANY), 1, (-3, 6)),
        ("    4567    abcd", (-3, 6), DirectionFree::from(Direction::Down), StopCondition::WordBoundary(WordBoundaryType::ANY), 1, (0, 6)),
        ("    4567    abcd", (-3, 3), DirectionFree::from(Direction::Down), StopCondition::WordBoundary(WordBoundaryType::ANY), 1, (-3, 3)),
        ("    4567    abcd", (0, 0), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::START), 1, (0, 4)),
        ("    4567    abcd", (0, 4), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::START), 1, (0, 12)),
        ("    4567    abcd", (0, 12), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::START), 1, (0, 12)),
        ("    4567    abcd", (0, 0), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::END), 1, (0, 7)),
        ("    4567    abcd", (0, 4), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::END), 1, (0, 7)),
        ("    4567    abcd", (0, 12), DirectionFree::from(Direction::Right), StopCondition::WordBoundary(WordBoundaryType::END), 1, (0, 15)),

        // TODO: Have to ponder a bit more on how to implement this feature. Test disabled for now.
        // ("0123456789\n\n0123456789", (0, 1), DirectionFree::new(1, 2).unwrap(), StopCondition::WordBoundary(WordBoundaryType::END), 1, (2, 6)),
        ("    4567    abcd", (0, 4), DirectionFree::from(Direction::Right), StopCondition::CharacterChange, 1, (0, 5)),
        ("    4567    abcd", (0, 7), DirectionFree::from(Direction::Right), StopCondition::CharacterChange, 1, (0, 8)),
        ("    4567    abcd", (0, 8), DirectionFree::from(Direction::Right), StopCondition::CharacterChange, 1, (0, 12)),
        ("    4567    abcd", (0, 6), DirectionFree::from(Direction::Up), StopCondition::CharacterChange, 1, (-1, 6)),
        ("    4567    abcd", (-3, 6), DirectionFree::from(Direction::Up), StopCondition::CharacterChange, 1, (-3, 6)),
        ("    4567    abcd", (-3, 6), DirectionFree::from(Direction::Down), StopCondition::CharacterChange, 1, (0, 6)),
        ("    4567    abcd", (-3, 3), DirectionFree::from(Direction::Down), StopCondition::CharacterChange, 1, (-3, 3)),
        ("    4567    abcd", (0, 1), DirectionFree::from(Direction::Right), StopCondition::CharacterMatch('6'), 1, (0, 6)),
        ("0  34 678   abcd", (0, 9), DirectionFree::from(Direction::Left), StopCondition::WordBoundary(WordBoundaryType::START), 2, (0, 4)),
    ];
    for (txt, start, direction, stop, stop_count, end) in tests {
        let mut canvas = Canvas::from_txt(txt.to_string()).unwrap();
        let it = CanvasIndexIterator::new(
            &mut canvas,
            start,
            direction,
            CanvasIterationJump::NoJump,
            stop,
            stop_count,
        );
        assert_eq!(it.last(), Some(end));
    }
}
