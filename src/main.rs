mod algorithms;
mod data;
mod state;

use std::io::{self, BufRead, Write};

use data::{Input, Output};
use state::State;

#[allow(dead_code)]
const MAX_TURNS: u64 = 500;
#[allow(dead_code)]
const HARD_MAX_DURATION: u64 = 1000;
#[allow(dead_code)]
const MAX_DURATION: u64 = 800;

fn main() {
    let stdin = io::stdin();
    let mut state = State::default();
    //let mut algorithm = AlgorithmSimple::default();
    let mut algorithm = algorithms::ripley::Ripley::new();
    //let mut algorithm = algorithms::simple::AlgorithmSimple::default();

    for line in stdin.lock().lines() {
        //let now = Instant::now();

        let line = line.unwrap();
        //eprintln!("{}", line);
        //eprintln!("=========================================================");
        //eprintln!("Turn: {}", state.turn + 1);
        //eprintln!("Input: {}", line);

        let input: Input = serde_json::from_str(&line).unwrap();
        //eprintln!("Input: {:?}", input);
        if state.turn == 0 {
            state = State::new(input);
        } else {
            state.update(input);
        }

        let output = Output {
            moves: algorithm.calculate(&state),
            //moves: vec![],
        };

        // while now.elapsed() < Duration::from_millis(MAX_DURATION) {
        //    sleep(Duration::from_millis(10));
        ////     //TODO: do things
        ////
        // }

        println!("{}", serde_json::to_string(&output).expect("Could not serialize output"));
        io::stdout().flush().unwrap();
        state.tick();
    }
}
