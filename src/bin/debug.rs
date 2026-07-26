use std::fs::File;

use RipleyPlanetWarsBot::{
    data::{Input, Move, ME_ID},
    state::{State, apply_simulated_moves, simulate_expeditions_planet},
};

fn duteros_fate(label: &str, state: &State, moves: &Vec<Move>) {
    // apply the moves to produce the resulting expedition set, then run the single-planet oracle
    let after_moves = apply_simulated_moves(ME_ID, moves, state.clone());
    let duteros = after_moves.current_state.planets.iter().find(|p| p.name == "duteros").unwrap();
    let (owner, ships) = simulate_expeditions_planet(&after_moves.current_state.expeditions, duteros);
    eprintln!("{label}: duteros ends owner={owner} ships={ships}");
}

fn main() {
    let file = File::open("what.json").unwrap();
    let input: Input = serde_json::from_reader(file).unwrap();
    let state = State::new(input);

    // baseline: duteros defends alone (keeps its 2 ships)
    duteros_fate("duteros alone", &state, &vec![]);

    // protos sends its 1 ship to help duteros
    let protos_helps = vec![Move::new("protos".to_string(), "duteros".to_string(), 1)];
    duteros_fate("protos helps", &state, &protos_helps);

    // both protos and extos send everything to duteros
    let all_help = vec![
        Move::new("protos".to_string(), "duteros".to_string(), 1),
        Move::new("extos".to_string(), "duteros".to_string(), 1),
    ];
    duteros_fate("protos + extos help", &state, &all_help);
}
