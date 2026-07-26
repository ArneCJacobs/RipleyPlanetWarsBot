use std::{collections::{BTreeMap, HashSet}, time::Instant};

use crate::{
    algorithms::ripley::Ripley, data::{MAX_DURATION, Move, PlayerId}, score::get_score_state, state::{State, apply_simulated_moves}, utils::consolidate_moves
};
use rand::{RngExt, SeedableRng, distr::{Distribution, weighted::WeightedIndex}, rngs::StdRng, seq::IteratorRandom};
use std::sync::{LazyLock, Mutex};

const MAX_ITERATIONS: u64 = 1000;
// simulated annealing temperature schedule: start hot enough to cross the score barriers of a
// half-built capture, cool geometrically so T_0 reaches ~0.5 over MAX_ITERATIONS steps
const INITIAL_TEMPERATURE: f64 = 30.0;
const COOLING_RATE: f64 = 0.993;
// neighbour concentration: chance to relocate a whole move rather than a random fraction
const WHOLE_MOVE_PROB: f64 = 0.7;
// upper bound on the lookahead derived from the map diameter
const MAX_HORIZON: i64 = 100;
const RNG_SEED: u64 = 42;
static RNG: LazyLock<Mutex<StdRng>> = LazyLock::new(|| Mutex::new(StdRng::seed_from_u64(RNG_SEED)));
// static RNG: LazyLock<Mutex<StdRng>> = LazyLock::new(|| Mutex::new(StdRng::from_os_rng()));
pub struct RipleyGreedyOptimization {
    me_id: PlayerId,
    heuristic_algorithm: Ripley
}



/// Returns the mutated move set together with the number of turns the newly added move takes to
/// reach its destination (its expedition's travel time), so the scorer knows when to inject the
/// next simulated move.
/// Static lookahead horizon: the ceil of the map diameter, capped. Depends only on planet
/// positions (which never change), so the score does not depend on the in-flight expeditions.
pub fn map_horizon(state: &State) -> i64 {
    let planets = &state.current_state.planets;
    let min_x = planets.iter().min_by(|a, b| a.x.partial_cmp(&b.x).unwrap()).unwrap();
    let max_x = planets.iter().max_by(|a, b| a.x.partial_cmp(&b.x).unwrap()).unwrap();
    let min_y = planets.iter().min_by(|a, b| a.y.partial_cmp(&b.y).unwrap()).unwrap();
    let max_y = planets.iter().max_by(|a, b| a.y.partial_cmp(&b.y).unwrap()).unwrap();
    let diameter = min_x.distance(max_x).max(min_y.distance(max_y));
    (diameter.ceil() as i64).min(MAX_HORIZON)
}

pub fn neighbour(
    state: &State,
    moves: &Vec<Move>,
) -> (Vec<Move>, i64) {
    let mut neighbour_moves = moves.clone();

    if neighbour_moves.is_empty() {
        return (vec![], 0);
    }

    let mut rng = RNG.lock().unwrap();
    let chosen_index = rng.random_range(..neighbour_moves.len());
    let mut old_move = neighbour_moves[chosen_index].clone();
    let old_target_id = state.planet_map[&old_move.destination];
    let origin_id = state.planet_map[&old_move.origin];

    // usually relocate the whole blob so concentration is preserved; occasionally split to explore
    let ships = if rng.random_range(0.0..1.0) < WHOLE_MOVE_PROB {
        old_move.ship_count as u64
    } else {
        rng.random_range(1..=old_move.ship_count as u64)
    };

    // planets this origin already sends to, used to bias the choice toward consolidation
    let same_origin_targets: HashSet<usize> = neighbour_moves
        .iter()
        .filter(|mv| mv.origin == old_move.origin)
        .map(|mv| state.planet_map[&mv.destination])
        .collect();

    // choose the destination by weight over candidate planets. The weight combines:
    //  - distance to the old target (nearer scores higher), with a hold (redirect back to origin)
    //    scored as high as the nearest neighbour
    //  - a bonus for planets this origin already sends to
    let closest_planets = state.get_closest(old_target_id);
    let max_weight = closest_planets.last().unwrap().0;
    let min_weight = closest_planets[1].0;
    let weights = closest_planets.iter().map(|(distance, id)| {
        let distance_weight = if *id == origin_id {
            max_weight
        } else {
            max_weight - distance.max(min_weight) + min_weight
        };
        let reinforce_weight = if same_origin_targets.contains(id) { max_weight } else { 0.0 };
        distance_weight + reinforce_weight
    });
    let new_target_id = closest_planets[WeightedIndex::new(weights).unwrap().sample(&mut *rng)].1;

    let arrival = state.current_state.planets[origin_id]
        .distance(&state.current_state.planets[new_target_id])
        .ceil() as i64;

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

    (consolidate_moves(neighbour_moves), arrival)
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

    /// Scores a candidate by simulating our own continued play into the future: apply the moves,
    /// advance to the new move's arrival, let Ripley pick our next move, advance 5 turns, let Ripley
    /// move again, then coast to the horizon and score. `arrival` is clamped so both follow-up
    /// stages fit inside the horizon.
    /// TODO: only our side is simulated here; test simulating the opponent's replies too.
    pub fn score_candidate(&mut self, begin_state: &State, moves: &Vec<Move>, arrival: i64, horizon: i64) -> f64 {
        let arrival = arrival.clamp(0, (horizon - 5).max(0));
        let mut state = apply_simulated_moves(self.me_id, moves, begin_state.clone()).advance(arrival);

        // our-side rollout: our follow-up moves via Ripley, then coast to the horizon.
        // A full two-sided rollout (opponent also playing to the horizon) was tried and washed out
        // the candidate move's signal, causing under-expansion; see git history.
        let our_move = self.heuristic_algorithm.calculate(&state);
        state = apply_simulated_moves(self.me_id, &our_move, state).advance(5);
        let our_move = self.heuristic_algorithm.calculate(&state);
        state = apply_simulated_moves(self.me_id, &our_move, state).advance((horizon - arrival - 5).max(0));

        get_score_state(self.me_id, &state)
    }

    pub fn calculate(&mut self, begin_state: &State) -> Vec<Move> {
        // eprintln!("======================================================================");
        // eprintln!("Begin state: {:?}", begin_state);
        let now = Instant::now();
        let mut best_moves = consolidate_moves(self.heuristic_algorithm.calculate(begin_state));

        add_loopback_moves(self.me_id, begin_state, &mut best_moves);

        let horizon = map_horizon(begin_state);

        // eprintln!("Initial moves: {:?}", best_moves);
        let initial_score = self.score_candidate(begin_state, &best_moves, 0, horizon);
        let seed_owned = apply_simulated_moves(self.me_id, &best_moves, begin_state.clone()).advance(horizon)
            .current_state.planets.iter().filter(|p| p.owner == Some(self.me_id)).count();

        // simulated annealing: `current` wanders (accepting worse moves so captures can be built up
        // across a valley of worse intermediate states), while `best` records the best seen and is
        // what we return.
        let mut current_moves = best_moves.clone();
        let mut current_score = initial_score;
        let mut best_score = initial_score;
        let mut temperature = INITIAL_TEMPERATURE;
        let mut iterations = 0;
        let mut accepts = 0;

        while now.elapsed().as_millis() < MAX_DURATION.into() && iterations < MAX_ITERATIONS {
            let (new_moves, arrival) = neighbour(begin_state, &current_moves);

            let new_score = self.score_candidate(begin_state, &new_moves, arrival, horizon);
            let delta = new_score - current_score;

            let accept = delta < 0.0 || {
                let mut rng = RNG.lock().unwrap();
                rng.random_range(0.0..1.0) < (-delta / temperature).exp()
            };
            if accept {
                accepts += 1;
                if new_score < best_score {
                    best_score = new_score;
                    best_moves = new_moves.clone();
                }
                current_moves = new_moves;
                current_score = new_score;
            }

            temperature *= COOLING_RATE;
            iterations += 1;
        }

        let best_sim = apply_simulated_moves(self.me_id, &best_moves, begin_state.clone()).advance(horizon);
        let best_owned = best_sim.current_state.planets.iter().filter(|p| p.owner == Some(self.me_id)).count();
        eprintln!(
            "turn={} iters={} accepts={} seed_score={:.1} best_score={:.1} seed_owned={} best_owned={} horizon={}",
            begin_state.turn, iterations, accepts, initial_score, best_score, seed_owned, best_owned, horizon
        );

        // eprintln!("moves: {:?}", best_moves);
        let elapsed = now.elapsed();
        // eprintln!("{:.2?}", elapsed);

        best_moves
    }

}
