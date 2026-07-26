use std::{collections::BTreeMap, time::Instant};

use crate::{
    algorithms::ripley::Ripley, data::{MAX_DURATION, Move, PlayerId}, state::{State, apply_simulated_moves}, utils::consolidate_moves
};
use rand::{RngExt, SeedableRng, distr::{Distribution, weighted::WeightedIndex}, rngs::StdRng, seq::IteratorRandom};
use std::sync::{LazyLock, Mutex};

const MAX_ITERATIONS: u64 = 600;
// long-term worth of owning a planet, replacing the growth of a long unopposed lookahead
const PLANET_VALUE: f64 = 50.0;
// upper bound on the lookahead derived from the map diameter
const MAX_HORIZON: i64 = 100;
const RNG_SEED: u64 = 42;
static RNG: LazyLock<Mutex<StdRng>> = LazyLock::new(|| Mutex::new(StdRng::seed_from_u64(RNG_SEED)));
// static RNG: LazyLock<Mutex<StdRng>> = LazyLock::new(|| Mutex::new(StdRng::from_os_rng()));
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
    for planet in &state.current_state.planets {
        match planet.owner {
            Some(owner) if owner == me_id => score -= PLANET_VALUE + planet.ship_count as f64,
            Some(_) => score += PLANET_VALUE,
            None => {} // neutral planets are not the enemy, so they do not count
        }

        // for other_planet in &state.current_state.planets {
        //     match other_planet.owner {
        //         None => continue,
        //         Some(x) if x == me_id => {
        //             score += planet.distance(other_planet).ceil() as f64 - other_planet.ship_count as f64;
        //         },
        //         Some(_) => {
        //             score -= planet.distance(other_planet).ceil() as f64 - other_planet.ship_count as f64;
        //         }
        //     }
        // }
    }

    score
}


pub fn neighbour(
    state: &State,
    moves: &Vec<Move>,
) -> Vec<Move> {
    let mut neighbour_moves = moves.clone();

    if neighbour_moves.is_empty() {
        return vec![];
    }

    let mut rng = RNG.lock().unwrap();
    let chosen_index = rng.random_range(..neighbour_moves.len());
    let mut old_move = neighbour_moves[chosen_index].clone();
    let old_target_id = state.planet_map[&old_move.destination];
    // TODO: only ever take one ship? or at least a max amount
    let ships = rng.random_range(1..=old_move.ship_count as u64);
    // pick planet close to current destination
    let closest_planets = state.get_closest(old_target_id);
    // TODO: we could store weighed index so its not constructed each time
    // TODO: instead of only taking distance into account, we could also use static defence-ability
    let max_weight = closest_planets.last().unwrap().0;
    let min_weight = closest_planets[1].0;
    let dist = WeightedIndex::new(closest_planets.iter().map(|(d, _)| max_weight-d.max(min_weight)+min_weight)).unwrap();
    let (_, new_target_id) = closest_planets[dist.sample(&mut *rng)];

    let destination = state.current_state.planets[new_target_id].name.clone();
    neighbour_moves.push(Move::new(
            old_move.origin.clone(),
            destination,
            ships.try_into().unwrap(),
    ));

    old_move.ship_count -= ships as i64;
    if old_move.ship_count == 0 {
        neighbour_moves.remove(chosen_index);
    } else {
        neighbour_moves[chosen_index] = old_move;
    }

    consolidate_moves(neighbour_moves)
}

pub fn add_loopback_moves(player_id: PlayerId, begin_state: &State, best_moves: &mut Vec<Move>) {
    let mut temp_map: BTreeMap<String, i64> = BTreeMap::new();
    for planet in &begin_state.current_state.planets {
        temp_map.insert(planet.name.clone(), planet.ship_count);
    }

    for mv in &*best_moves {
        *temp_map.get_mut(&mv.origin).unwrap() -= mv.ship_count;
    }
    // simulate reserves as moves from and to the same planet
    for planet in &begin_state.current_state.planets {
        let value = *temp_map.get(&planet.name).unwrap();
        if planet.owner != Some(player_id) || value <= 0 {
            continue
        }
        best_moves.push(Move{
            origin: planet.name.clone(),
            destination: planet.name.clone(),
            ship_count: value,
        });
    }
}

impl RipleyGreedyOptimization {
    pub fn new(me_id: PlayerId) -> Self {
        RipleyGreedyOptimization {
            me_id,
            heuristic_algorithm: Ripley::new(me_id)
        }
    }

    pub fn calculate(&mut self, begin_state: &State) -> Vec<Move> {
        // eprintln!("======================================================================");
        // eprintln!("Begin state: {:?}", begin_state);
        let now = Instant::now();
        let mut best_moves = consolidate_moves(self.heuristic_algorithm.calculate(begin_state));

        add_loopback_moves(self.me_id, begin_state, &mut best_moves);

        // static horizon: the ceil of the map diameter, capped, so the score does not depend on
        // the in-flight expeditions. The diameter is approximated by the planets furthest apart in
        // x and in y (whichever pair spans more), since planet positions never change.
        let planets = &begin_state.current_state.planets;
        let min_x = planets.iter().min_by(|a, b| a.x.partial_cmp(&b.x).unwrap()).unwrap();
        let max_x = planets.iter().max_by(|a, b| a.x.partial_cmp(&b.x).unwrap()).unwrap();
        let min_y = planets.iter().min_by(|a, b| a.y.partial_cmp(&b.y).unwrap()).unwrap();
        let max_y = planets.iter().max_by(|a, b| a.y.partial_cmp(&b.y).unwrap()).unwrap();
        let diameter = min_x.distance(max_x).max(min_y.distance(max_y));
        let horizon = (diameter.ceil() as i64).min(MAX_HORIZON);

        // eprintln!("Initial moves: {:?}", best_moves);
        let simulated_state = apply_simulated_moves(self.me_id, &best_moves, begin_state.clone()).apply_expeditions(horizon);
        let mut best_score = get_score_state(self.me_id, &simulated_state);
        let mut iterations = 0;

        while now.elapsed().as_millis() < MAX_DURATION.into() && iterations < MAX_ITERATIONS {
            // eprintln!("{:.2?}, {}", now.elapsed().as_millis(), iterations);
            // eprintln!("new moves before: {:?}", temp);
            let new_moves = neighbour(begin_state, &best_moves);
            // eprintln!("new moves after: {:?}", new_moves);

            let simulated_state = apply_simulated_moves(self.me_id, &new_moves, begin_state.clone()).apply_expeditions(horizon);
            let new_score = get_score_state(self.me_id, &simulated_state);
            if new_score < best_score {
                best_score = new_score;
                best_moves = new_moves;
            }
            iterations += 1;
        }

        // eprintln!("moves: {:?}", best_moves);
        let elapsed = now.elapsed();
        // eprintln!("{:.2?}", elapsed);

        best_moves
    }

}
