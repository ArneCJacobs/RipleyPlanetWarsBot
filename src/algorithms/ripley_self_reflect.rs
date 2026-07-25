use std::collections::HashSet;

use crate::{
    algorithms::ripley::{Ripley, simulate_expeditions, simulate_expeditions_required_ships_to_survive}, data::{Expedition, ME_ID, Move, OTHER_ID, Planet, PlayerId}, state::{State, apply_simulated_moves}
};

struct Scores {
    ship_count: i64,
    projected_ship_count: i64,
    projected_owner: PlayerId,
    ships_needed_to_survive: i64,
}
pub struct RipleySelfReflect {
    other_algorithm: Ripley
}

// score are better the lower they are
const DEFENCE_FACTOR: f32 = 1.0;
const NEUTRAL_FACTOR: f32 = 1.5;
const OFFENCE_FACTOR: f32 = 1.2;

impl RipleySelfReflect {
    pub fn new() -> Self {RipleySelfReflect { other_algorithm: Ripley::new(OTHER_ID) }}

    pub fn calculate(&mut self, starting_state: &State) -> Vec<Move> {
        let mut moves = vec![];

        let simulated_moves = self.other_algorithm.calculate(starting_state); 
        let simulated_state = apply_simulated_moves(simulated_moves, starting_state);

        let planet_it = simulated_state
            .current_state
            .planets
            .iter()
            .map(|p| {
                let (owner_sim, ship_count) = simulate_expeditions(&simulated_state.current_state.expeditions, p);
                let ships_needed_to_survive = simulate_expeditions_required_ships_to_survive(&simulated_state.current_state.expeditions, p);
                (p, Scores {
                    ship_count,
                    projected_ship_count: ship_count, // Placeholder, not used in this algorithm
                    projected_owner: owner_sim,
                    ships_needed_to_survive,
                })
            })
            .collect::<Vec<_>>();

        let mut scores = vec![];
        for planet in &simulated_state.current_state.planets {
            let planet_scores = &planet_it[planet.index].1;
            // don't send ships from our own planets if we don't have enough to survive
            if (planet.owner == Some(ME_ID) && planet.ship_count < planet_scores.ships_needed_to_survive)  
                || planet.owner != Some(ME_ID)
            {
                continue;
            }
            for other_planet in &simulated_state.current_state.planets {
                if planet.name == other_planet.name {
                    continue; // skip self
                }

                let distance = planet.distance(other_planet).ceil() as i64;
                //if 
                let mut score = None; 
                let other_planet_scores = &planet_it[other_planet.index].1;

                if other_planet.owner == Some(ME_ID) && 
                    other_planet.ship_count < other_planet_scores.ships_needed_to_survive {
                    score = Some((distance + other_planet_scores.ships_needed_to_survive) as f32 * DEFENCE_FACTOR);
                } else if other_planet_scores.projected_owner != ME_ID {
                    let factor = if other_planet.owner.is_none() {
                        NEUTRAL_FACTOR
                    } else {
                        OFFENCE_FACTOR
                    };
                    score = Some((distance + other_planet_scores.projected_ship_count) as f32 * factor);
                } 
                if let Some(score) = score {
                    scores.push((score, planet, other_planet));
                }
            }
        }

        // Sort scores by the first element (the score) in ascending order
        scores.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());


        let mut planets_seen = HashSet::new();

        for (_, planet, other_planet) in scores {
            if planets_seen.contains(&planet.index) {
                continue; // skip if we already processed this planet
            }
            planets_seen.insert(planet.index);

            let ship_count = planet.ship_count;
            let ships_needed_to_survive = planet_it[planet.index].1.ships_needed_to_survive;
            if ship_count <= ships_needed_to_survive {
                continue; // skip if we don't have enough ships to survive
            }
            let move_ship_count = ship_count - ships_needed_to_survive;
            moves.push(Move {
                origin: planet.name.clone(),
                destination: other_planet.name.clone(),
                ship_count: move_ship_count,
            });
        }
        moves
    }
}
