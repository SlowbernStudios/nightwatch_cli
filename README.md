# Nightwatch

You have one job: restore power to the generator. Unfortunately, the building
has other ideas, the equipment wants paperwork, and your supervisor is a cat.

Nightwatch is a cozy, turn-based terminal adventure that teaches you Vim keys
while you bumble through your shift. Every shift brings a new layout and a fresh
collection of maintenance problems. No Vim experience needed. No timer, either.
The cat can wait. Probably.

## How to play

With Git and [Rust](https://rustup.rs/) installed, open your terminal and run:

```sh
git clone https://github.com/SlowbernStudios/nightwatch_cli.git
cd nightwatch_cli
cargo run --locked --release
```

Follow the task card, explore the rooms, and repair the equipment standing
between you and a working generator. The opening lessons show you the keys as
you need them. If a console has you stumped, open the menu with `Tab` for a
worked example or a reset. Petting the supervisor is also encouraged.

Save through the menu before quitting; there's no autosave. To resume:

```sh
cargo run --locked --release -- --load .nightwatch-save.json
```

## Controls

| Key | Action |
| --- | --- |
| `h` `j` `k` `l` | Walk left, down, up, right. Arrow keys work too. |
| Number + direction | Walk several steps: `3l` moves up to three spaces right. |
| `Enter` | Interact with nearby equipment or submit a console repair. |
| `Tab` | Open the menu: help, supplies, cat time, save, and quit. |
| `j` / `k` or arrows, then `Enter` | Choose a menu option. |
| `Esc` | Close a menu or stop typing in a console. |

At repair consoles, you'll learn these Vim keys along the way:

| Key | Action |
| --- | --- |
| `w` / `b` / `e` | Next word, previous word, end of word. |
| `0` / `$` | Start / end of line. |
| `i` | Start typing before the cursor; press `Esc` when finished. |
| `x` | Delete a character. |
| `dd` | Delete a line. |
| `u` | Undo. We've all been there. |
