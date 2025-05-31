use std::collections::HashSet;

use crate::{
    data::{Expedition, Move, Planet, PlayerId, ME_ID},
    state::State,
};

pub fn simulate_expeditions_required_ships_to_survive(expeditions: &[Expedition], planet: &Planet) -> i64 {
    let mut relevant_expiditions: Vec<_> = expeditions
        .iter()
        .filter(|exp| exp.destination == planet.name)
        .collect();

    relevant_expiditions.sort_by_key(|exp| exp.turns_remaining);

    let mut ship_count_required_to_survive = 0;
    let owner = planet.owner.unwrap_or(0);
    let mut ship_count = 0;
    let mut last_simulated_turn = 0;
    //eprintln!("{}", planet.name);
    //eprintln!("T\tS\tSn");
    //eprintln!("{}\t{}\t{}", last_simulated_turn, ship_count, ship_count_required_to_survive);

    for expedition in relevant_expiditions {
        // account for growth
        if owner != 0 {
            ship_count += expedition.turns_remaining - last_simulated_turn;
        }

        if expedition.owner == owner {
            ship_count += expedition.ship_count;
        } else if expedition.ship_count >= ship_count {
            ship_count_required_to_survive += expedition.ship_count - ship_count + 1;
            ship_count = 1;
        } else {
            ship_count -= expedition.ship_count;
        }
        last_simulated_turn = expedition.turns_remaining;
        //eprintln!("{}\t{}\t{}", last_simulated_turn, ship_count, ship_count_required_to_survive);
    }

    ship_count_required_to_survive
}

pub fn simulate_expeditions(expeditions: &[Expedition], planet: &Planet) -> (PlayerId, i64) {
    let mut relevant_expiditions: Vec<_> = expeditions
        .iter()
        .filter(|exp| exp.destination == planet.name)
        .collect();

    relevant_expiditions.sort_by_key(|exp| exp.turns_remaining);

    let mut owner = planet.owner.unwrap_or(0);
    let mut ship_count = planet.ship_count;
    let mut last_simulated_turn = 0;

    for expedition in relevant_expiditions {
        // account for growth
        if owner != 0 {
            ship_count += expedition.turns_remaining - last_simulated_turn;
        }

        if expedition.owner == owner {
            ship_count += expedition.ship_count;
        } else if expedition.ship_count > ship_count {
            ship_count = expedition.ship_count - ship_count;
            owner = expedition.owner;
        } else if expedition.ship_count == ship_count {
            ship_count = 0;
            owner = 0;
        } else {
            ship_count -= expedition.ship_count;
        }
        last_simulated_turn = expedition.turns_remaining;
    }

    (owner, ship_count)
}

struct Scores {
    ship_count: i64,
    projected_ship_count: i64,
    projected_owner: PlayerId,
    ships_needed_to_survive: i64,
}
pub struct Ripley {
}

// score are better the lower they are
const DEFENCE_FACTOR: f32 = 2.0;
const NEUTRAL_FACTOR: f32 = 4.0;
const OFFENCE_FACTOR: f32 = 1.0;

impl Ripley {
    pub fn new() -> Self {Ripley { }}

    pub fn calculate(&mut self, state: &State) -> Vec<Move> {
        let mut moves = vec![];

        let planet_it = state
            .current_state
            .planets
            .iter()
            .map(|p| {
                let (owner_sim, ship_count) = simulate_expeditions(&state.current_state.expeditions, p);
                let ships_needed_to_survive = simulate_expeditions_required_ships_to_survive(&state.current_state.expeditions, p);
                (p, Scores {
                    ship_count,
                    projected_ship_count: ship_count, // Placeholder, not used in this algorithm
                    projected_owner: owner_sim,
                    ships_needed_to_survive,
                })
            })
            .collect::<Vec<_>>();

        let mut scores = vec![];
        for planet in &state.current_state.planets {
            let planet_scores = &planet_it[planet.index].1;
            // don't send ships from our own planets if we don't have enough to survive
            if (planet.owner == Some(ME_ID) && planet.ship_count < planet_scores.ships_needed_to_survive)  
                || planet.owner != Some(ME_ID)
            {
                continue;
            }
            for other_planet in &state.current_state.planets {
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
