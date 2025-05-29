mod data;
mod state;
mod algorithms;

use std::io::{self, BufRead};

use algorithms::simple::AlgorithmSimple;
use data::{Input, Output};
use state::State;

const MAX_TURNS: u64 = 500;
#[allow(dead_code)]
const HARD_MAX_DURATION: u64 = 1000;
#[allow(dead_code)]
const MAX_DURATION: u64 = 800;

fn main() {

    let stdin = io::stdin();
    let mut state = State::default();
    let mut algorithm = AlgorithmSimple::default();
    // let mut algorithm = SimpleAlrorithm {};


    for line in stdin.lock().lines() {
        //let now = Instant::now();
        let line = line.unwrap();
        eprintln!("=========================================================");

        let input: Input = serde_json::from_str(&line).unwrap();
        if state.turn == 0 {
            state = State::new(input);
        } else {
            state.update(input);
        }

        let output = Output {
            moves: algorithm.calculate(&state)
        };

        // while now.elapsed() < Duration::from_millis(MAX_DURATION) {
        //    sleep(Duration::from_millis(10));
        ////     //TODO: do things
        ////
        // }

        println!("{}", serde_json::to_string(&output).expect("Could not serialize output"));
        eprintln!("{}", serde_json::to_string(&output).expect("Could not serialize output"));
        state.tick();
    }
}
