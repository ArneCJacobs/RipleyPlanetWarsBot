use crate::{
    data::{Expedition, Move, Planet, PlayerId, ME_ID},
    state::State,
};

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

pub struct Ripley;

impl Ripley {
    pub fn new() -> Self {
        Ripley {}
    }

    pub fn calculate(&mut self, state: &State) -> Vec<Move> {
        let mut moves = vec![];

        let planet_it = state
            .current_state
            .planets
            .iter()
            .map(|p| (p, simulate_expeditions(&state.current_state.expeditions, p)))
            .collect::<Vec<_>>();

        for (planet, (owner_sim, _)) in &planet_it {
            if *owner_sim != ME_ID {
                continue;
            }

            let mut best_enemy_planet = None;
            {
                let mut best_score = None;
                for (p, (o, fleet)) in &planet_it {
                    if *o == ME_ID {
                        continue; // skip our own planets
                    }
                    let distance: i64 = planet.distance(p).ceil() as i64;
                    let score: i64 = fleet + distance;
                    if best_score.is_none() || score < best_score.unwrap() {
                        best_enemy_planet = Some(p);
                        best_score = Some(score);
                    }
                }
            }

            if let Some(enemy_planet) = best_enemy_planet {
                let (o1, sc1) = planet_it[planet.index].1;
                let (o2, sc2) = planet_it[enemy_planet.index].1;
                if o1 != ME_ID || o2 == ME_ID {
                    continue;
                }
                if sc1 < sc2 + 1 {
                    continue; // skip if we can't conquer the enemy planet
                }
                moves.push(Move::new(
                    planet.name.clone(),
                    enemy_planet.name.clone(),
                    sc2 + 1,
                ));
            }
        }
        moves
    }
}
