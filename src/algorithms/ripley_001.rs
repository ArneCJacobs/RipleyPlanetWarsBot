use crate::{data::{Expedition, Move, Planet, PlayerId, ME_ID}, state::State};

pub struct Ripley001;


fn simulate_expeditions(
    expeditions: &[Expedition],
    planet: &Planet,
) -> (PlayerId, i64) {
    let mut relevant_expiditions: Vec<_> = expeditions.iter()
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

impl Ripley001 {
    pub fn new() -> Self {
        Ripley001 {}
    }

    pub fn calculate(&mut self, state: &State) -> Vec<Move> {
        let mut moves = vec![];
        for planet in &state.current_state.planets {
            if planet.owner != Some(ME_ID) {
                continue
            }

            let mut nearest_enemy_planet = None;
            {
                for (_, planet_id) in &state.nearest_planets[planet.index] {
                    let planet = &state.current_state.planets[*planet_id];
                    if planet.owner != Some(ME_ID) {
                        nearest_enemy_planet = Some(planet);
                        break;
                    }
                }
            }

            if let Some(enemy_planet) = nearest_enemy_planet {
                let (o1, sc1) = simulate_expeditions(&state.current_state.expeditions, planet);
                let (o2, sc2) = simulate_expeditions(&state.current_state.expeditions, enemy_planet);
                if o1 != ME_ID || o2 == ME_ID {
                    continue; // skip if we can't conquer the enemy planet
                }
                moves.push(Move::new(
                    planet.name.clone(),
                    enemy_planet.name.clone(),
                    i64::min(sc1, sc2 + 1),
                ));
            }
        }
        moves
    }
}
