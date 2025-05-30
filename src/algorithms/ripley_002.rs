
use crate::{data::{Move, ME_ID}, state::State};

use super::util::simulate_expeditions;

pub struct Ripley002;

impl Ripley002 {
    pub fn new() -> Self {
        Ripley002 {}
    }

    pub fn calculate(&mut self, state: &State) -> Vec<Move> {
        let mut moves = vec![];

        let planet_it = state.current_state.planets.iter()
            .map(|p| (p, simulate_expeditions(&state.current_state.expeditions, p)))
            .collect::<Vec<_>>();

        for (planet, (owner_sim, _)) in &planet_it {
            if *owner_sim != ME_ID {
                continue
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
