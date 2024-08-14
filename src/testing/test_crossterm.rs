use crossterm::{
    event::{self, Event, KeyCode},
    terminal::disable_raw_mode,
};

fn application() {
    loop {
        let event = event::read().unwrap();
        dbg!(event.clone());
        if let Event::Key(event) = event {
            if event.code == KeyCode::Char('q') {
                return;
            }
        }
    }
}

fn application_wrapper() {
    // let terminal = enable_raw_mode().unwrap();
    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |e| {
        disable_raw_mode().unwrap();
        default_panic_hook(e);
        std::process::exit(1);
    }));
    let result = application();
    disable_raw_mode().unwrap();
    return result;
}

fn main() {
    application_wrapper();
}
