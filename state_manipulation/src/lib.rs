extern crate rand;
extern crate common;

use common::*;
use common::HandEnum::*;

use rand::{StdRng, SeedableRng, Rng};

//NOTE(Ryan1729): debug_assertions only appears to work correctly when the
//crate is not a dylib. Assuming you make this crate *not* a dylib on release,
//these configs should work
#[cfg(debug_assertions)]
#[no_mangle]
pub fn new_state(size: Size) -> State {
    //skip the title screen
    println!("debug on");

    let seed: &[_] = &[42];
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    make_state(size, false, rng)
}
#[cfg(not(debug_assertions))]
#[no_mangle]
pub fn new_state(size: Size) -> State {
    //show the title screen
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|dur| dur.as_secs())
        .unwrap_or(42);

    println!("{}", timestamp);
    let seed: &[_] = &[timestamp as usize];
    let rng: StdRng = SeedableRng::from_seed(seed);

    make_state(size, true, rng)
}

fn deal(state: &mut State) -> Card {
    deal_parts(&mut state.deck, &mut state.pile, &mut state.rng)
}

fn deal_parts(deck: &mut Vec<Card>, pile: &mut Vec<Card>, rng: &mut StdRng) -> Card {
    //reshuffle if we run out of cards.
    if deck.len() == 0 {
        debug_assert!(pile.len() != 0, "deck was empty and so was the pile!");

        for card in pile.drain(..) {
            deck.push(card);
        }

        rng.shuffle(deck.as_mut_slice());
    };

    deck.pop()
        .unwrap_or(Card {
                       suit: Suit::Spades,
                       value: Value::Ace,
                   })
}


fn make_state(size: Size, title_screen: bool, mut rng: StdRng) -> State {
    let mut deck = Card::all_values();

    rng.shuffle(deck.as_mut_slice());

    let mut pile = Vec::new();
    let player;
    let mut cpu_players;
    let pile_card = {
        let deck_ref = &mut deck;
        let pile_ref = &mut pile;
        let rng_ref = &mut rng;

        player = Hand(deal_parts(deck_ref, pile_ref, rng_ref),
                      deal_parts(deck_ref, pile_ref, rng_ref),
                      deal_parts(deck_ref, pile_ref, rng_ref));

        let cpu_players_count = rng_ref.gen_range(1, 5);
        cpu_players = Vec::new();

        for _ in 0..cpu_players_count {
            cpu_players.push(Hand(deal_parts(deck_ref, pile_ref, rng_ref),
                                  deal_parts(deck_ref, pile_ref, rng_ref),
                                  deal_parts(deck_ref, pile_ref, rng_ref)));
        }

        deal_parts(deck_ref, pile_ref, rng_ref)
    };

    pile.push(pile_card);

    State {
        rng,
        title_screen,
        deck,
        pile,
        player,
        cpu_players,
        ui_context: UIContext::new(),
    }
}

#[no_mangle]
//returns true if quit requested
pub fn update_and_render(platform: &Platform, state: &mut State, events: &mut Vec<Event>) -> bool {
    if state.title_screen {

        for event in events {
            cross_mode_event_handling(platform, state, event);
            match *event {
                Event::Close |
                Event::KeyPressed {
                    key: KeyCode::Escape,
                    ctrl: _,
                    shift: _,
                } => return true,
                Event::KeyPressed {
                    key: _,
                    ctrl: _,
                    shift: _,
                } => state.title_screen = false,
                _ => (),
            }
        }

        (platform.print_xy)(5, 10, "Click to start.");

        false
    } else {
        game_update_and_render(platform, state, events)
    }
}

pub fn game_update_and_render(platform: &Platform,
                              state: &mut State,
                              events: &mut Vec<Event>)
                              -> bool {
    let mut left_mouse_pressed = false;
    let mut left_mouse_released = false;

    for event in events {
        cross_mode_event_handling(platform, state, event);

        match *event {
            Event::KeyPressed {
                key: KeyCode::MouseLeft,
                ctrl: _,
                shift: _,
            } => {
                left_mouse_pressed = true;
            }
            Event::KeyReleased {
                key: KeyCode::MouseLeft,
                ctrl: _,
                shift: _,
            } => {
                left_mouse_released = true;
            }
            Event::Close |
            Event::KeyPressed {
                key: KeyCode::Escape,
                ctrl: _,
                shift: _,
            } => return true,
            _ => (),
        }
    }

    state.ui_context.frame_init();

    let redeal_spec = ButtonSpec {
        x: 0,
        y: 0,
        w: 11,
        h: 3,
        text: "Redeal".to_string(),
        id: 1,
    };

    if do_button(platform,
                 &mut state.ui_context,
                 &redeal_spec,
                 left_mouse_pressed,
                 left_mouse_released) {
        *state = make_state((platform.size)(), false, state.rng);
    }

    match state.player {
        Hand(ref c1, ref c2, ref c3) => {
            let size = (platform.size)();

            let mut x = CARD_OFFSET;
            let y = size.height - HAND_HEIGHT_OFFSET;

            draw_card(platform, x, y, c1);
            x += CARD_OFFSET_DELTA;

            draw_card(platform, x, y, c2);
            x += CARD_OFFSET_DELTA;

            draw_card(platform, x, y, c3);
        }
    }

    if state.deck.len() > 0 {
        if do_card_back_button(platform,
                               &mut state.ui_context,
                               60,
                               10,
                               left_mouse_pressed,
                               left_mouse_released,
                               88) {
            let card = deal(state);

            state.pile.push(card);
        }
    }

    if let Some(top_card) = state.pile.last() {
        draw_card(platform, 40, 10, top_card)
    }

    false
}

const CARD_OFFSET: i32 = 5;
const CARD_OFFSET_DELTA: i32 = 6;

const HAND_HEIGHT_OFFSET: i32 = 8;

const CARD_WIDTH: i32 = 16;
const CARD_HEIGHT: i32 = 12;

fn draw_card(platform: &Platform, x: i32, y: i32, card: &Card) {
    draw_rect(platform, x, y, CARD_WIDTH, CARD_HEIGHT);

    (platform.print_xy)(x + 1, y + 1, &card.value.to_string());
    (platform.print_xy)(x + 1, y + 2, &card.suit.to_string());
}
fn draw_card_back(platform: &Platform, x: i32, y: i32) {
    draw_rect(platform, x, y, CARD_WIDTH, CARD_HEIGHT);
    draw_rect(platform, x + 2, y + 1, CARD_WIDTH - 4, CARD_HEIGHT - 2);
}

fn do_card_back_button(platform: &Platform,
                       context: &mut UIContext,
                       x: i32,
                       y: i32,
                       left_mouse_pressed: bool,
                       left_mouse_released: bool,
                       id: i32)
                       -> bool {
    let spec = ButtonSpec {
        x,
        y,
        w: CARD_WIDTH,
        h: CARD_HEIGHT,
        text: String::new(),
        id,
    };

    let result = do_button(platform,
                           context,
                           &spec,
                           left_mouse_pressed,
                           left_mouse_released);

    draw_rect(platform, x + 2, y + 1, CARD_WIDTH - 4, CARD_HEIGHT - 2);

    result
}

fn cross_mode_event_handling(platform: &Platform, state: &mut State, event: &Event) {
    match *event {
        Event::KeyPressed {
            key: KeyCode::R,
            ctrl: true,
            shift: _,
        } => {
            println!("reset");
            *state = new_state((platform.size)());
        }
        _ => (),
    }
}

pub struct ButtonSpec {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub text: String,
    pub id: i32,
}

//calling this once will swallow multiple clicks on the button. We could either
//pass in and return the number of clicks to fix that, or this could simply be
//called multiple times per frame (once for each click).
fn do_button(platform: &Platform,
             context: &mut UIContext,
             spec: &ButtonSpec,
             left_mouse_pressed: bool,
             left_mouse_released: bool)
             -> bool {
    let mut result = false;

    let mouse_pos = (platform.mouse_position)();
    let inside = inside_rect(mouse_pos, spec.x, spec.y, spec.w, spec.h);
    let id = spec.id;

    if context.active == id {
        if left_mouse_released {
            result = context.hot == id && inside;

            context.set_not_active();
        }
    } else if context.hot == id {
        if left_mouse_pressed {
            context.set_active(id);
        }
    }

    if inside {
        context.set_next_hot(id);
    }

    if context.active == id && (platform.key_pressed)(KeyCode::MouseLeft) {
        draw_rect_with(platform,
                       spec.x,
                       spec.y,
                       spec.w,
                       spec.h,
                       ["╔", "═", "╕", "║", "│", "╙", "─", "┘"]);
    } else if context.hot == id {
        draw_rect_with(platform,
                       spec.x,
                       spec.y,
                       spec.w,
                       spec.h,
                       ["┌", "─", "╖", "│", "║", "╘", "═", "╝"]);
    } else {
        draw_rect(platform, spec.x, spec.y, spec.w, spec.h);
    }

    print_centered_line(platform, spec.x, spec.y, spec.w, spec.h, &spec.text);

    return result;
}

pub fn inside_rect(point: Point, x: i32, y: i32, w: i32, h: i32) -> bool {
    x <= point.x && y <= point.y && point.x < x + w && point.y < y + h
}

fn print_centered_line(platform: &Platform, x: i32, y: i32, w: i32, h: i32, text: &str) {
    let char_count = text.chars().count();

    if char_count == 0 {
        return;
    }

    let x_ = {
        let rect_middle = x + (w / 2);

        rect_middle - (char_count as f32 / 2.0) as i32
    };

    let y_ = y + (h / 2);

    (platform.print_xy)(x_, y_, &text);
}


fn draw_rect(platform: &Platform, x: i32, y: i32, w: i32, h: i32) {
    draw_rect_with(platform,
                   x,
                   y,
                   w,
                   h,
                   ["┌", "─", "┐", "│", "│", "└", "─", "┘"]);
}

fn draw_rect_with(platform: &Platform, x: i32, y: i32, w: i32, h: i32, edges: [&str; 8]) {
    (platform.clear)(Some(Rect::from_values(x, y, w, h)));

    let right = x + w - 1;
    let bottom = y + h - 1;
    // top
    (platform.print_xy)(x, y, edges[0]);
    for i in (x + 1)..right {
        (platform.print_xy)(i, y, edges[1]);
    }
    (platform.print_xy)(right, y, edges[2]);

    // sides
    for i in (y + 1)..bottom {
        (platform.print_xy)(x, i, edges[3]);
        (platform.print_xy)(right, i, edges[4]);
    }

    //bottom
    (platform.print_xy)(x, bottom, edges[5]);
    for i in (x + 1)..right {
        (platform.print_xy)(i, bottom, edges[6]);
    }
    (platform.print_xy)(right, bottom, edges[7]);
}
