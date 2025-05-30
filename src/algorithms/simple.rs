use crate::{
    data::{Move, ME_ID},
    state::State,
};

#[derive(Default)]
pub struct AlgorithmSimple {}

#[allow(dead_code)]
impl AlgorithmSimple {
    pub fn calculate(&mut self, state: &State) -> Vec<Move> {
        let mut moves = vec![];
        for planet in &state.current_state.planets {
            if planet.owner != Some(ME_ID) {
                continue;
            }

            let mut nearest_enemy_planet = None;
            for (_, planet_id) in &state.nearest_planets[planet.index] {
                let planet = &state.current_state.planets[*planet_id];
                if planet.owner != Some(ME_ID) {
                    nearest_enemy_planet = Some(planet);
                    break;
                }
            }

            if let Some(enemy_planet) = nearest_enemy_planet {
                if planet.ship_count > 7 {
                    moves.push(Move::new(planet.name.clone(), enemy_planet.name.clone(), 5));
                }
            }
        }
        moves
    }
}
