extern crate rand;
extern crate common;

use common::*;
use common::HandEnum::*;
use common::HandCard::*;

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
        selected_card: None,
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

    let size = (platform.size)();
    let hand_height = size.height - HAND_HEIGHT_OFFSET;

    match state.player {
        Hand(ref c1, ref c2, ref c3) => {

            let mut x = CARD_OFFSET;

            if state.selected_card.is_none() {
                draw_card(platform, x, hand_height, c1);
                x += CARD_OFFSET_DELTA;

                draw_card(platform, x, hand_height, c2);
                x += CARD_OFFSET_DELTA;

                draw_card(platform, x, hand_height, c3);
            }
        }
    }


    if state.selected_card.is_some() {
        let selection = select_returned_card(platform,
                                             state,
                                             hand_height,
                                             left_mouse_pressed,
                                             left_mouse_released);

        match selection {
            FromHand(index) => {
                state
                    .pile
                    .push(state
                              .player
                              .swap(index, state.selected_card.take().unwrap()))
            }
            SelectedCard => {
                state.pile.push(state.selected_card.take().unwrap());
            }
            NoSelection => {}
        }

        if let Some(top_card) = state.pile.last() {
            draw_card(platform, PILE_X, PILE_Y, top_card);
        }

        if state.deck.len() > 0 {
            draw_card_back(platform, DECK_X, DECK_Y);
        }
    } else {
        if state.deck.len() > 0 {
            if do_card_back_button(platform,
                                   &mut state.ui_context,
                                   DECK_X,
                                   DECK_Y,
                                   left_mouse_pressed,
                                   left_mouse_released,
                                   88) {
                let card = deal(state);

                state.selected_card = Some(card);
            }
        }

        let selected_pile_top = if let Some(top_card) = state.pile.last() {
            do_card_button(platform,
                           &mut state.ui_context,
                           PILE_X,
                           PILE_Y,
                           top_card,
                           left_mouse_pressed,
                           left_mouse_released,
                           100)
        } else {
            false
        };

        if selected_pile_top {
            state.selected_card = state.pile.pop();
        }
    }

    false
}

enum ReturnSelection {
    FromHand(HandCard),
    SelectedCard,
    NoSelection,
}
use ReturnSelection::*;

fn select_returned_card(platform: &Platform,
                        state: &mut State,
                        hand_height: i32,
                        left_mouse_pressed: bool,
                        left_mouse_released: bool)
                        -> ReturnSelection {

    let mut id = 110;
    match state.player {
        Hand(ref c1, ref c2, ref c3) => {

            let mut x = CARD_OFFSET;

            if do_raised_card_button(platform,
                                     &mut state.ui_context,
                                     x,
                                     hand_height,
                                     c1,
                                     left_mouse_pressed,
                                     left_mouse_released,
                                     id) {
                return FromHand(FirstCard);
            }

            x += CARD_OFFSET_DELTA;
            id += 1;

            if do_raised_card_button(platform,
                                     &mut state.ui_context,
                                     x,
                                     hand_height,
                                     c2,
                                     left_mouse_pressed,
                                     left_mouse_released,
                                     id) {
                return FromHand(SecondCard);
            }


            x += CARD_OFFSET_DELTA;
            id += 1;

            if do_raised_card_button(platform,
                                     &mut state.ui_context,
                                     x,
                                     hand_height,
                                     c3,
                                     left_mouse_pressed,
                                     left_mouse_released,
                                     id) {
                return FromHand(ThirdCard);
            }
        }
    }

    id += 1;

    if let Some(ref selected_card) = state.selected_card {
        if do_card_button(platform,
                          &mut state.ui_context,
                          50,
                          hand_height,
                          selected_card,
                          left_mouse_pressed,
                          left_mouse_released,
                          id) {
            return SelectedCard;
        }

    }

    NoSelection
}


const PILE_X: i32 = 40;
const DECK_X: i32 = PILE_X + CARD_WIDTH + 4;

const PILE_Y: i32 = 10;
const DECK_Y: i32 = PILE_Y;

const CARD_OFFSET: i32 = 5;
const CARD_OFFSET_DELTA: i32 = 6;

const HAND_HEIGHT_OFFSET: i32 = 8;

const CARD_WIDTH: i32 = 16;
const CARD_HEIGHT: i32 = 12;

fn draw_card(platform: &Platform, x: i32, y: i32, card: &Card) {
    draw_rect(platform, x, y, CARD_WIDTH, CARD_HEIGHT);

    draw_card_value(platform, x + 1, y + 1, card);
}
fn draw_card_back(platform: &Platform, x: i32, y: i32) {
    draw_rect(platform, x, y, CARD_WIDTH, CARD_HEIGHT);

    draw_card_back_design(platform, x, y);
}

fn draw_card_value(platform: &Platform, x: i32, y: i32, card: &Card) {
    (platform.print_xy)(x, y, &card.value.to_string());
    (platform.print_xy)(x, y + 1, &card.suit.to_string());
}
fn draw_card_back_design(platform: &Platform, x: i32, y: i32) {
    draw_rect(platform, x + 2, y + 1, CARD_WIDTH - 4, CARD_HEIGHT - 2);
}

const CARD_RAISE_OFFSET: i32 = CARD_HEIGHT / 2;

//TODO animation. Maybe put a counter on the UIContext?
fn do_raised_card_button(platform: &Platform,
                         context: &mut UIContext,
                         x: i32,
                         y: i32,
                         card: &Card,
                         left_mouse_pressed: bool,
                         left_mouse_released: bool,
                         id: UiId)
                         -> bool {
    let mut result = false;

    let mouse_pos = (platform.mouse_position)();
    let active = context.active == id;
    let hot = context.hot == id;
    let raised_y = y - CARD_RAISE_OFFSET;
    let inside = inside_rect(mouse_pos, x, y, CARD_WIDTH, CARD_HEIGHT) ||
                 (hot && inside_rect(mouse_pos, x, raised_y, CARD_WIDTH, CARD_HEIGHT));

    if active {
        if left_mouse_released {
            result = hot && inside;

            context.set_not_active();
        }
    } else if hot {
        if left_mouse_pressed {
            context.set_active(id);
        }
    }

    if inside {
        context.set_next_hot(id);
    }

    if active && (platform.key_pressed)(KeyCode::MouseLeft) {
        draw_rect_with(platform,
                       x,
                       raised_y,
                       CARD_WIDTH,
                       CARD_HEIGHT,
                       ["╔", "═", "╕", "║", "│", "╙", "─", "┘"]);
        draw_card_value(platform, x + 1, raised_y + 1, card);
    } else if context.hot == id {
        draw_rect_with(platform,
                       x,
                       raised_y,
                       CARD_WIDTH,
                       CARD_HEIGHT,
                       ["┌", "─", "╖", "│", "║", "╘", "═", "╝"]);
        draw_card_value(platform, x + 1, raised_y + 1, card);
    } else {
        draw_card(platform, x, y, card);
    }

    return result;
}

fn do_card_button(platform: &Platform,
                  context: &mut UIContext,
                  x: i32,
                  y: i32,
                  card: &Card,
                  left_mouse_pressed: bool,
                  left_mouse_released: bool,
                  id: UiId)
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

    draw_card_value(platform, x + 1, y + 1, card);

    result
}

fn do_card_back_button(platform: &Platform,
                       context: &mut UIContext,
                       x: i32,
                       y: i32,
                       left_mouse_pressed: bool,
                       left_mouse_released: bool,
                       id: UiId)
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

    draw_card_back_design(platform, x, y);

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
    pub id: UiId,
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
