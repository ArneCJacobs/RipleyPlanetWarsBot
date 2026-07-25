use std::collections::HashMap;

use crate::data::{Expedition, Input, Move, OTHER_ID, Planet, PlanetName, PlayerId};

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct State {
    pub current_state: Input,
    pub planet_map: HashMap<PlanetName, usize>,
    pub turn: i64,
    // maps planet_id to a list of planet_ids and distances, sorted by distance ascending
    //pub nearest_planets: Vec<Vec<(f32, PlanetId)>>,
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

        State {
            current_state: input,
            planet_map,
            turn: 0,
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
        &self,
        turns_lookahead: i64,
    ) -> State {
       let mut simulated_state = self.clone();

       let expeditions = &mut simulated_state.current_state.expeditions;
       expeditions.sort_unstable_by_key(|expidition| expidition.turns_remaining);
       let mut turn = 0;

       for expedition in expeditions {
           let delta = expedition.turns_remaining - turn;
           if turn + delta > turns_lookahead {
               break;
           }
           for planet in &mut simulated_state.current_state.planets {
               if planet.owner.is_none() {
                   continue;
               }

               // account for growth
               planet.ship_count += delta;

               if expedition.owner == planet.owner.unwrap() {
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
               turn = expedition.turns_remaining;
           }
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
}

pub fn apply_simulated_moves(
    simulated_moves: Vec<Move>,
    state: &State,
) -> State {
    let mut simulated_state = state.clone(); 

    for player_move in &simulated_moves {
        let planet_origin = &state.current_state.planets[state.planet_map[&player_move.origin]];
        let planet_destination = &state.current_state.planets[state.planet_map[&player_move.destination]];
        let distance = planet_origin.distance(planet_destination).ceil() as i64;
        simulated_state.current_state.expeditions.push(Expedition{
            id: 1235,
            ship_count: player_move.ship_count,
            origin: player_move.origin.clone(),
            destination: player_move.destination.clone(),
            owner: OTHER_ID,
            turns_remaining: distance,
        })
    }


    simulated_state
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
