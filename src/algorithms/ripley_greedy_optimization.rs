use std::time::Instant;

use crate::{
    algorithms::ripley::Ripley, data::{Move, PlayerId}, state::{State, apply_simulated_moves}
};

pub struct RipleyGreedyOptimization {
    me_id: PlayerId,
    heuristic_algorithm: Ripley
}

/// This function calculates 2 score functions:
/// 1. total scip score (sum of all enemy ships - sum of al allied ships)
/// 2. dynamic defence-ability:
///    sum over all planets p =>
///        sum over all allied planets pi of (|distance(p, pi)| - pi.ships) 
///        plus sum over all enemy planet pi of (pi.ships - |distance(p, pi)|)
pub fn get_score_state(
    me_id: PlayerId,
    state: &State,
) -> f64 {
    // lower better
    let mut score = 0.0;
    for (index, planet) in state.current_state.planets.iter().enumerate() {
        if planet.owner == Some(me_id) {
            score -= planet.ship_count as f64;
        } else {
            score += planet.ship_count as f64;
        }

        // TODO: instead iterate over the n closest
        for other_planet in &state.current_state.planets[index+1..] {
            match planet.owner {
                None => continue,
                Some(x) if x == me_id => {
                    score += planet.distance(other_planet).ceil() as f64 - other_planet.ship_count as f64;
                },
                Some(_) => {
                    score -= planet.distance(other_planet).ceil() as f64 - other_planet.ship_count as f64;
                }
            }
        }
    }

    score
}



impl RipleyGreedyOptimization {
    pub fn new(me_id: PlayerId) -> Self {
        RipleyGreedyOptimization {
            me_id,
            heuristic_algorithm: Ripley::new(me_id)
        }
    }

    pub fn calculate(&mut self, state: &State) -> Vec<Move> {
        let start = self.heuristic_algorithm.calculate(state);

        let now = Instant::now();
        let simulated_state = state.apply_expeditions(100);
        let _score = get_score_state(self.me_id, &simulated_state);
        let elapsed = now.elapsed();
        eprintln!("{:.2?}", elapsed);


        start
    }
}
