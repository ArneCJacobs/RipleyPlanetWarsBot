mod algorithms;
mod data;
mod state;

use std::{io::{self, BufRead, Write}};

use data::{Input, Output};
use state::State;

use crate::data::ME_ID;

fn main() {
    let stdin = io::stdin();
    let mut state = State::default();
    //let mut algorithm = AlgorithmSimple::default();
    // let mut algorithm = algorithms::ripley_self_reflect::RipleySelfReflect::new();
    // let mut algorithm = algorithms::ripley::Ripley::new(ME_ID);
    //let mut algorithm = algorithms::simple::AlgorithmSimple::default();
    // let mut file = File::open("debug.jsonl").unwrap();
    let mut algorithm = algorithms::ripley_greedy_optimization::RipleyGreedyOptimization::new(ME_ID);

    for line in stdin.lock().lines() {
        //let now = Instant::now();

        let line = line.unwrap();
        //eprintln!("{}", line);
        //eprintln!("=========================================================");
        //eprintln!("Turn: {}", state.turn + 1);
        //eprintln!("Input: {}", line);
        // file.write_all(&line.clone().into_bytes()).unwrap();

        let input: Input = serde_json::from_str(&line).unwrap();
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
