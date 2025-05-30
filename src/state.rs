use std::collections::HashMap;

use bit_set::BitSet;

use crate::data::{Input,PlanetName};

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct State {
    pub current_state: Input,
    pub saved_expeditions: BitSet,
    pub planet_map: HashMap<PlanetName, usize>,
    pub planet_names: Vec<PlanetName>,
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
            //nearest_planets,
            planet_names,
            current_state: input,
            planet_map,
            saved_expeditions: BitSet::new(),
            turn: 0,
        }
    }

    pub fn update(&mut self, mut input: Input) {
        for planet in &mut input.planets {
            planet.index = *self.planet_map.get(&planet.name).unwrap();
        }

        self.current_state = input;
    }
}
