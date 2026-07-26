use std::{cell::{Ref, RefCell}, collections::HashMap, rc::Rc};

use crate::data::{Expedition, Input, ME_ID, Move, OTHER_ID, Planet, PlanetId, PlanetName, PlayerId};

type DistanceMatrix = Vec<Vec<(f64, PlanetId)>>;

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct State {
    pub current_state: Input,
    pub planet_map: HashMap<PlanetName, usize>,
    pub turn: i64,
    // maps planet_id to a list of planet_ids and distances, sorted by distance ascending
    //pub nearest_planets: Vec<Vec<(f32, PlanetId)>>,
    pub distance_matrix: Rc<RefCell<DistanceMatrix>>
}

impl State {
    pub fn tick(&mut self) {
        self.turn += 1;
    }

    pub fn new(mut input: Input) -> Self {
        let mut planet_map = HashMap::new();
        let mut planet_names = vec![];
        //let mut nearest_planets = Vec::new();

        for (index, planet) in input.planets.iter_mut().enumerate() {
            planet_map.insert(planet.name.clone(), index);
            planet_names.push(planet.name.clone());
            planet.index = index;
        }

        //for planet_current in &input.planets {
        //    let mut distances = vec![];
        //    let planet_location = &planet_locations[planet_current.index];
        //    for planet_other in &input.planets {
        //        if planet_other.index == planet_current.index {
        //            continue;
        //        }
        //        let other_location = &planet_locations[planet_other.index];
        //        let distance = planet_location.distance(other_location);
        //        distances.push((distance, planet_other.index));
        //    }
        //
        //    distances.sort_unstable_by(|(d1, _), (d2, _)| d1.partial_cmp(d2).unwrap());
          //nearest_planets.push(distances);
        //}
        
        // TODO: use something that inlines the vec, something like smallvec
        let distance_matrix = (0..planet_names.len()).map(|_| Vec::with_capacity(planet_names.len())).collect();

        State {
            current_state: input,
            planet_map,
            turn: 0,
            distance_matrix: Rc::new(RefCell::new(distance_matrix)),
        }
    }

    pub fn update(&mut self, mut input: Input) {
        for planet in &mut input.planets {
            planet.index = *self.planet_map.get(&planet.name).unwrap();
        }

        self.current_state = input;
    }

    /// apply expeditions and look turns_lookahead into the future. For scoring purposes it is
    /// important to have a set turns_remaining otherwise the amount of turns you look into the
    /// future depends solely on the expeditions and the scoring might differ greatly
    pub fn apply_expeditions(
        self,
        turns_lookahead: i64,
    ) -> State {
       let mut simulated_state = self;

       let expeditions = &mut simulated_state.current_state.expeditions;
       expeditions.sort_unstable_by_key(|expidition| expidition.turns_remaining);
       let mut turn = 0;

       for expedition in expeditions {
           let delta = expedition.turns_remaining - turn;
           if turn + delta > turns_lookahead {
               break;
           }
           for planet in &mut simulated_state.current_state.planets {
               // account for growth
               if planet.owner.is_some() {
                   planet.ship_count += delta;
               }

               // only apply expidition to destination planet
               if planet.name != expedition.destination {
                   continue;
               }

               if Some(expedition.owner) == planet.owner {
                   planet.ship_count += expedition.ship_count;
               } else if expedition.ship_count > planet.ship_count {
                   planet.ship_count = expedition.ship_count - planet.ship_count;
                   planet.owner = Some(expedition.owner);
               } else if expedition.ship_count == planet.ship_count {
                   planet.ship_count = 0;
                   planet.owner = None;
               } else {
                   planet.ship_count -= expedition.ship_count;
               }
           }
           turn = expedition.turns_remaining;
       }

       let delta = turns_lookahead - turn;
       if delta > 0 {
           for planet in &mut simulated_state.current_state.planets {
               if planet.owner.is_none() {
                   continue
               }
               planet.ship_count += delta;
           }
       }

       simulated_state
    }

    /// Advance the simulation by `turns`: resolve the expeditions that arrive within that window
    /// (removing them), decrement the turns_remaining of the rest, and grow owned planets over the
    /// elapsed time. Unlike `apply_expeditions` this consumes arrived expeditions, so it can be
    /// chained to simulate several stages in a row.
    pub fn advance(self, turns: i64) -> State {
        let mut simulated_state = self;
        let mut expeditions = std::mem::take(&mut simulated_state.current_state.expeditions);
        expeditions.sort_unstable_by_key(|expedition| expedition.turns_remaining);

        let mut turn = 0;
        let mut remaining = Vec::new();
        for mut expedition in expeditions {
            if expedition.turns_remaining > turns {
                // not arrived within the window: keep it, shifted back in time
                expedition.turns_remaining -= turns;
                remaining.push(expedition);
                continue;
            }

            let delta = expedition.turns_remaining - turn;
            for planet in &mut simulated_state.current_state.planets {
                if planet.owner.is_some() {
                    planet.ship_count += delta;
                }
                if planet.name != expedition.destination {
                    continue;
                }
                if Some(expedition.owner) == planet.owner {
                    planet.ship_count += expedition.ship_count;
                } else if expedition.ship_count > planet.ship_count {
                    planet.ship_count = expedition.ship_count - planet.ship_count;
                    planet.owner = Some(expedition.owner);
                } else if expedition.ship_count == planet.ship_count {
                    planet.ship_count = 0;
                    planet.owner = None;
                } else {
                    planet.ship_count -= expedition.ship_count;
                }
            }
            turn = expedition.turns_remaining;
        }

        let delta = turns - turn;
        if delta > 0 {
            for planet in &mut simulated_state.current_state.planets {
                if planet.owner.is_some() {
                    planet.ship_count += delta;
                }
            }
        }

        simulated_state.current_state.expeditions = remaining;
        simulated_state
    }

    pub fn get_closest(
        &self,
        planet_id: PlanetId,
    ) -> Ref<'_, Vec<(f64, PlanetId)>> {

        {
            let mut distance_matrix = self.distance_matrix.borrow_mut();
            let elem = &mut distance_matrix[planet_id];
            if elem.is_empty() {
                let planet_current = &self.current_state.planets[planet_id];
                for (other_planet_id, planet_other) in self.current_state.planets.iter().enumerate() {
                    // if other_planet_id == planet_id {
                    //     continue;
                    // }

                    let distance = planet_current.distance(planet_other);
                    elem.push((distance.into(), other_planet_id));
                }
                elem.sort_unstable_by(|(d1, _), (d2, _)| d1.partial_cmp(d2).unwrap());
            }
        }

        Ref::map(self.distance_matrix.borrow(), |value| &value[planet_id])

    }
}

// converts moves given to the server to actual expiditions
pub fn apply_simulated_moves(
    owner_id: PlayerId,
    simulated_moves: &Vec<Move>,
    mut state: State,
) -> State {
    for player_move in simulated_moves {
        {
            let planet_origin = &mut state.current_state.planets[state.planet_map[&player_move.origin]];
            if planet_origin.owner != Some(owner_id) {
                panic!("Trying to send an expeidition from a planet you do not own: owner: {owner_id}, planet: {planet_origin:?}")
            }
            // move from and to the same planet finish instantly and are not counted
            if player_move.origin == player_move.destination {
                // planet_origin.ship_count += player_move.ship_count;
                continue;
            }
            if player_move.ship_count > planet_origin.ship_count {
                panic!("Player moves more ships then available, ships available: {0}, move: {player_move:?}", planet_origin.ship_count)
            }
            planet_origin.ship_count -= player_move.ship_count;
        }
        let planet_origin = &state.current_state.planets[state.planet_map[&player_move.origin]];
        let planet_destination = &state.current_state.planets[state.planet_map[&player_move.destination]];
        let distance = planet_origin.distance(planet_destination).ceil() as i64;
        state.current_state.expeditions.push(Expedition{
            id: 1235,
            ship_count: player_move.ship_count,
            origin: player_move.origin.clone(),
            destination: player_move.destination.clone(),
            owner: owner_id,
            turns_remaining: distance,
        })
    }


    state
}

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

#[cfg(test)]
mod test {
    use crate::data::{Expedition, Input, Planet};
    use crate::state::State;

    #[test]
    fn apply_expeditions_only_affects_the_destination_planet() {
        let input = Input {
            planets: vec![
                Planet { ship_count: 5, x: 0.0, y: 0.0, owner: Some(1), name: "a".to_string(), index: 0 },
                Planet { ship_count: 1, x: 0.0, y: 0.0, owner: Some(1), name: "b".to_string(), index: 0 },
            ],
            expeditions: vec![
                Expedition { id: 1, ship_count: 10, origin: "b".to_string(), destination: "b".to_string(), owner: 2, turns_remaining: 1 },
            ],
        };
        let state = State::new(input);

        let result = state.apply_expeditions(1);

        // The enemy expedition only targets "b", so "a" must stay mine (plus one turn of growth).
        let planet_a = result.current_state.planets.iter().find(|p| p.name == "a").unwrap();
        assert_eq!(planet_a.owner, Some(1), "planet a was wrongly affected by an expedition aimed at b");
        assert_eq!(planet_a.ship_count, 6, "planet a should have grown by one turn only");
    }

    #[test]
    fn apply_expeditions_captures_neutral_planet() {
        let input = Input {
            planets: vec![
                Planet { ship_count: 3, x: 0.0, y: 0.0, owner: None, name: "n".to_string(), index: 0 },
            ],
            expeditions: vec![
                Expedition { id: 1, ship_count: 5, origin: "n".to_string(), destination: "n".to_string(), owner: 2, turns_remaining: 1 },
            ],
        };
        let state = State::new(input);

        let result = state.apply_expeditions(1);

        // 5 attacking ships beat 3 neutral defenders, leaving player 2 owning it with 2 ships.
        let planet = &result.current_state.planets[0];
        assert_eq!(planet.owner, Some(2), "neutral planet should be captured by the attacker");
        assert_eq!(planet.ship_count, 2, "captured planet keeps the surplus attacking ships");
    }
}

pub fn simulate_expeditions_planet(expeditions: &[Expedition], planet: &Planet) -> (PlayerId, i64) {
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
