use std::time::Instant;

use crate::{
    algorithms::ripley::Ripley, data::{MAX_DURATION, Move, PlayerId}, state::{State, apply_simulated_moves}
};
use rand::{distr::{Distribution, weighted::WeightedIndex}, seq::IteratorRandom};

const MAX_ITERATIONS: u64 = 600;
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

        for other_planet in &state.current_state.planets {
            match other_planet.owner {
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


pub fn neighbour(
    state: &State,
    moves: &Vec<Move>,
) -> Vec<Move> {
    // TODO: just mutate moves
    let mut neighbour_moves = moves.clone();
    if neighbour_moves.is_empty() {
        return vec![];
    }

    let chosen_index = rand::random_range(..neighbour_moves.len());
    let mut chosen_move = neighbour_moves[chosen_index].clone();
    // TODO: only ever take one ship? or at least a max amount
    let ships = rand::random_range(..chosen_move.ship_count as u64);
    let old_target_id = state.planet_map[&chosen_move.destination];
    chosen_move.ship_count -= ships as i64;
    // pick planet close to current destination
    let closest_planets = state.get_closest(old_target_id);
    // TODO: we could store weighed index so its not constructed each time
    // TODO: instead of only taking distance into account, we could also use static defence-ability
    let max_weight = closest_planets.last().unwrap().0;
    let dist = WeightedIndex::new(closest_planets.iter().map(|(d, _)| max_weight-d)).unwrap();
    let (_, mut new_target_id) = closest_planets[dist.sample(&mut rand::rng())];
    // only retry once, good enough
    if new_target_id == old_target_id {
        (_, new_target_id) = closest_planets[dist.sample(&mut rand::rng())];
    }

    neighbour_moves.push(Move::new(
        chosen_move.origin.clone(),
        state.current_state.planets[new_target_id].name.clone(),
        ships.try_into().unwrap(),
    ));
    
    if chosen_move.ship_count == 0 {
        neighbour_moves.remove(chosen_index);
    } else {
        neighbour_moves[chosen_index] = chosen_move;
    }

    neighbour_moves 
}

fn consolidate_moves(mut moves: Vec<Move>) -> Vec<Move> {
    moves.sort_unstable_by_key(|mv| (mv.origin.clone(), mv.destination.clone(), mv.ship_count));

    let mut new_moves = Vec::new();
    for (index, mv) in moves.iter().enumerate() {
        let mut run_last_index = index + 1; 
        for (other_index, other_mv) in moves.iter().enumerate().skip(index+1) {
            if other_mv.origin != mv.origin || other_mv.destination != mv.destination {
                run_last_index = other_index;
                break;
            }
        };
        if run_last_index == index + 1 {
            new_moves.push(mv.clone());
            continue;
        }
        let mv_agg = moves[index..run_last_index].iter().fold(
            Move {
                origin: mv.origin.clone(),
                destination: mv.destination.clone(),
                ship_count: 0,
            }, 
            |mut acc, mv| {
            acc.ship_count += mv.ship_count;
            acc
        });
        new_moves.push(mv_agg);
    }
    new_moves.into_iter().filter(|mv| mv.ship_count != 0).collect()
}


impl RipleyGreedyOptimization {
    pub fn new(me_id: PlayerId) -> Self {
        RipleyGreedyOptimization {
            me_id,
            heuristic_algorithm: Ripley::new(me_id)
        }
    }

    pub fn calculate(&mut self, begin_state: &State) -> Vec<Move> {
        let now = Instant::now();
        let mut best_moves = consolidate_moves(self.heuristic_algorithm.calculate(begin_state));
        let simulated_state = apply_simulated_moves(self.me_id, &best_moves, begin_state).apply_expeditions(100);
        let mut best_score = get_score_state(self.me_id, &simulated_state);
        let mut iterations = 0;

        while now.elapsed().as_millis() < MAX_DURATION.into() && iterations < MAX_ITERATIONS {
            // eprintln!("{:.2?}, {}", now.elapsed().as_millis(), iterations);
            let new_moves = consolidate_moves(neighbour(begin_state, &best_moves));

            let simulated_state = apply_simulated_moves(self.me_id, &new_moves, begin_state).apply_expeditions(100);
            let new_score = get_score_state(self.me_id, &simulated_state);
            if new_score < best_score {
                best_score = new_score;
                best_moves = new_moves;
            }
            iterations += 1;
        }
        let elapsed = now.elapsed();
        eprintln!("{:.2?}", elapsed);

        best_moves
    }
}
