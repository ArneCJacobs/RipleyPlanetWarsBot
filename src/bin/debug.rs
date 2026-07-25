use std::fs::File;

use RipleyPlanetWarsBot::{algorithms::ripley_greedy_optimization::RipleyGreedyOptimization, data::{Input, ME_ID}, state::State};

fn main() {

    let file = File::open("what.json").unwrap();
    let input: Input = serde_json::from_reader(file).unwrap();
    let state = State::new(input);
    eprintln!("{:?}", state);

    let mut ripley = RipleyGreedyOptimization::new(ME_ID);

    let moves = ripley.calculate(&state);

    eprintln!("{:?}", moves);
}
