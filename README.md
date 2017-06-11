This is a single player version of a card game called 31.

There are several slightly different versions of this game so here's the rules this version is based on:

### Setup
Shuffle the deck, deal three cards to each player, then take the top card of the deck and place it face-up next to the deck, beginning what we will refer to as "the pile".

### Play
Players take turns choosing either the top card of the pile or the top card of the deck and adding it to their hand. The player then chooses a card to place back on the pile, attempting to create the highest scoring hand. If a player creates a hand worth 31 points they instantly win. See the hand Scoring Section below.

Instead of the above players may alternatively announce that they believe they have a good enough hand to beat everyone else. This is referred to as "knocking". The other players each get one more turn to try and get a better hand, then all hands are revealed and the player(s) with the best hand(s) win.

### Hand Scoring
Aces count fo 11 points, face cards all count for 10 points and all other cards count for their value. A hand is worth a number of points equal to the count of the cards in one suit in their hand.   

If a hand consists of three cards with the same value (say three 4's), then that hand is worth 30½ points, (despite the cards not all being the same suit).

### Examples
  9♠, A♥, 8♥ scores 19.
  ♦7, 7♠, 7♥ scores 30½.
	A♣, 10♣, K♣ scores 31 and wins immeadiately

## Installing required lib on Linux

This program relies on `libBearLibTerminal.so` so that should be copied into `usr/local/lib` or another folder indicated by this command: `ldconfig -v 2>/dev/null | grep -v ^$'\t'`

then you should run `sudo ldconfig` to complete the installation.

Then the executable should run correctly.

Alternately if your OS has a package for BearLibTerminal, that may work as well.

Once that's done compiling in debug mode with `cargo build` and release mode with `cargo build --release` should work.

## Compiling release mode for Windows

You will need a copy of the precompiled `BearLibTerminal.dll` and `BearLibTerminal.lib`.

Perform the folloing steps:

copy BearLibTerminal.lib to the project root

Comment out the line containing `crate-type = ["dylib"]` in the `Cargo.toml` in the `state_manipulation` folder. (this is more or less a workaround for [this issue](https://github.com/rust-lang/rust/issues/18807), hopefully we will eventually be able to make this switch using the `cfg` attribute, but currently using the attribute doesn't appear to work correctly.)

Run `cargo build --release` then copy the exe in `./target/release` to the desired location as well as `BearLibTerminal.dll` and any necessary assets (graphics, sound, etc.).
