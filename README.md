# Upaint

```
                                                                      .
                                                                     //./ . 
                                                                    ///////./ 
                                                                   ///////// 
                                            ___________           ///////// 
                                  _..---''''           ''''---.._| '-/////   
                             ..-''     .---.       .---.         |     '/  
                          .-'         /     \     /     \      .-/  .--'. 
                        .'  .---.     \     /     \     /     / /  /\    '.
                       /   /     \     '---'       '---'      \/  / /      \
                      |    \     /                            /  /-'        |
                      |     '---'                            /  /           |
                      |                                     /  /  .---.     |
                       '.                                  /  /  /     \    |
                         '-.._                  ____      /  /   \     /    |
                              ''""----------""''    )   ./=-/     '---'    /
                                                   /   //  /\\            /
                                                   |   \\ / //          .'
                                                    \   '-=-'       _.-'
                                                     /-._______...-'  
                                                    /  /
                                                   /  /
                                                  /  /
                                                 (__/
```

Upaint is a modern and VIM-like program for creating and modifiying text-art. Upaint is written in Rust and has out-of-the-box support for Unicode characters and 24-bit colors.

## Installing

Upaint can currently not be installed automatically through any package manager or automated installer.

Upaint is currently only supported on Linux and WSL2 platforms.

1\. Install rust and cargo by following the instructions at [https://www.rust-lang.org/learn/get-started](https://www.rust-lang.org/learn/get-started).

2\. Download the source code

```
git clone https://github.com/sigurdo/upaint.git
cd upaint
```

3\. Build

```
cargo build --release
```

4\. Create a symbolic link from `/usr/bin`

```
ln -s $(pwd)/target/release/main /usr/bin/upaint
```

## Usage

### Basics

To open a blank canvas:

```
upaint
```

To open an existing artwork (`art.ansi`):

```
upaint art.ansi
```

To move the cursor left, down, up or right, use `h`, `j`, `k` or `l`, respectively.

To enter input mode, press `d`, followed by the appropriate direction key (`h`, `j`, `k` or `l`) for the direction in which you want the cursor to move as you type.

To return to normal mode, press `ESCAPE`.

To start typing a command, press `:`, and press `ENTER` to execute it.

To save your artwork as `art.ansi`, use the command `:w art.ansi`.

To exit, use the command `:q`.

### Advanced

Press `u` to undo and `ctrl+r` to redo your last changes.

Press `e`, followed by a brush aspect key to change that aspect of the paintbrush. The brush aspect keys are `c` for character, `f` for foreground color, `b` for background color, `m` for modifiers and `a` for all of them.

In normal mode, press `SPACE` to paint the cell under the cursor with the chosen paintbrush. Press one of the brush aspect keys to paint with only that brush aspect.

Press `p` followed by a brush aspect key to pick that brush aspect from the cell under the cursor and apply it to the paintbrush.

## Licensing

Upaint is dual-licensed under MÅKESODD v1 and GNU GPL v3. You are required to follow the terms from either one of the two licenses (your choice) when distributing modified or unmodified versions of Upaint.
