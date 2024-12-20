use ratatui::style::Color;
use upaint::canvas::raw::Canvas;

fn main() {
    // Just some unscientific benchmarking of btree_map::CellMap vs array::CellMap.
    // Conclusion: In this test, which heavily benefits array due to accessing many different cells
    // in a compact region, array was about 4x faster, if I interpret flamegraph correctly.
    // This means, for artworks with e.g. background color or thight patterns, array map will be
    // significantly faster, but it's not like a 100x speedup, and there are many other factors
    // contributing to performance issues.
    // So for now, it's not worth the effort and risk involved with the array map.
    let mut canvas = Canvas::default();
    for red in 0..255 {
        for row in -10..10 {
            for column in -100..100 {
                canvas.set_fg((row, column), Color::Rgb(red, 0, 0));
            }
        }
    }
}
