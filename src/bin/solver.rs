use clap::{arg, command, value_parser, ArgAction, Command};
use one_forty_nine_solver::{BoardState, PIECE_MAP};

// To log after this many states are computed
const LOG_FREQUENCY: u64 = 100000;

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(
            arg!(
                -t --target <NUMBER> "Set the target number"
            )
            // We don't have syntax yet for optional options, so manually calling `required`
            .required(true)
            .value_parser(value_parser!(u8)),
        )
        .get_matches();

    let target: &u8 = matches.get_one::<u8>("target").unwrap();
    if *target > 49 {
        println!()
    }

    let mut states = BoardState::new(*target);

    let mut counter: u64 = 0;

    while !states.is_empty() {
        let state: &BoardState = &(states.pop().unwrap());
        if state.pieces_to_place.is_empty() {
            println!("Solution found:");
            println!("{}", state);
            return;
        }

        for piece_index in state.pieces_to_place.iter() {
            let piece = PIECE_MAP[piece_index];
            for i in 0..piece.variants.len() {
                if let Ok(new_state) = state.place_piece(piece, i, false) {
                    states.push(new_state);
                    counter += 1;

                    if counter % LOG_FREQUENCY == 0 {
                        println!("# states searched: {}, stack size: {}", counter, states.len());
                    }
                }
            }
        }
    }

    println!("No solution found");
}
