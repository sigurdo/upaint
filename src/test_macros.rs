fn main() {
    macro_rules! action_arm {
        ($($variant:expr => $action:expr),*) => {
            match 64 {
                $(
                    $variant => $action,
                )*
                _ => 10,
            }
        };
    }

    fn a(b: i64) {
        println!("a: {b}");
    }

    let c = 64;

    // let result = match c {
    //     action_arm!(63, 3),
    //     action_arm!(64, 4),
    //     action_arm!(65, 5),
    //     _ => 10,
    // };
    let result = action_arm!(63 => 3, 65 => 5);

    dbg!(result);
}
