// https://github.com/Robbe7730/anna/blob/master/src/main.rs
use std::io::{self, BufRead};
use serde::{Serialize, Deserialize};
use itertools::iproduct;
use std::collections::HashSet;

// ----- INPUT -----

#[derive(Clone, Serialize, Deserialize)]
struct Planet {
    name: String,
    x: f64,
    y: f64,
    owner: Option<usize>,
    ship_count: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct Expedition {
    id: usize,
    origin: String,
    destination: String,
    turns_remaining: usize,
    owner: usize,
    ship_count: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct GameState {
    planets: Vec<Planet>,
    expeditions: Vec<Expedition>,
}

// ----- OUTPUT -----

#[derive(Clone, Serialize, Deserialize)]
struct Move {
    origin: String,
    destination: String,
    ship_count: usize
}

#[derive(Clone, Serialize, Deserialize)]
struct Turn {
    moves: Vec<Move>,
}

// ----- HELPER FUNCTIONS -----

fn distance_between_planets(planet1: &Planet, planet2: &Planet) -> f64 {
    let dx = planet1.x - planet2.x;
    let dy = planet1.y - planet2.y;
    (dx * dx + dy * dy).sqrt()
}

fn simulate_arrivals(planet: &Planet, gamestate: &GameState) -> (usize, usize) {
    let mut relevant_expeditions = gamestate.expeditions
                                        .iter()
                                        .filter(|x| x.destination == planet.name)
                                        .collect::<Vec<&Expedition>>();
    relevant_expeditions.sort_by_key(|x| x.turns_remaining);
    let mut owner = planet.owner.unwrap_or(0);
    let mut ship_count = planet.ship_count;
    let mut last_simulated_turn = 0;
    for expedition in relevant_expeditions {
        // Account for growth
        if owner != 0 {
            ship_count += expedition.turns_remaining - last_simulated_turn;
        }
        last_simulated_turn = expedition.turns_remaining;

        if expedition.owner == owner {
            ship_count += expedition.ship_count;
        } else if ship_count < expedition.ship_count {
            owner = expedition.owner;
            ship_count = expedition.ship_count - ship_count;
        } else if ship_count == expedition.ship_count {
            owner = 0;
            ship_count = 0;
        } else {
            ship_count -= expedition.ship_count;
        }
    }
    (owner, ship_count)
}

fn score(source: &Planet, dest: &Planet, gamestate: &GameState) -> (usize, usize) {
    let (owner, ship_count) = simulate_arrivals(dest, gamestate);
    if (ship_count+1) >= source.ship_count || owner == 1 {
        (0, 0)
    } else {
        (
            ship_count+1,
            distance_between_planets(source, dest).ceil() as usize * (ship_count+1)
        )
    }
}

// ----- NEXT MOVE -----

fn next_move(state: &GameState) -> Turn {
    let my_planets: Vec<&Planet> = state.planets
        .iter()
        .filter(|x| x.owner.unwrap_or(0) == 1)
        .collect();
    let other_planets: Vec<&Planet> = state.planets
        .iter()
        .filter(|x| x.owner.unwrap_or(0) != 1)
        .collect();

    if my_planets.len() == 0 || other_planets.len() == 0 {
        Turn { moves: vec![] }
    } else {
        let mut moves = vec![];

        let mut best_move = iproduct!(my_planets.iter(), other_planets.iter())
            .map(|(s,d)| (s, d, score(s, d, state)))
            .filter(|(_,_,(_,sc))| *sc != 0)
            .min_by_key(|x| (*x).2.1);

        let mut used_planets: HashSet<String> = HashSet::new();

        while let Some((source, dest, (ship_count, _score))) = best_move {
            moves.push(Move {
                origin: source.name.to_string(),
                destination: dest.name.to_string(),
                ship_count: ship_count,
            });

            used_planets.insert(source.name.to_string());

            best_move = iproduct!(
                my_planets.iter().filter(|x| !used_planets.contains(&x.name)),
                other_planets.iter()
            ).map(|(s,d)| (s, d, score(s, d, state)))
             .filter(|(_,_,(_, sc))| *sc != 0)
             .min_by_key(|x| (*x).2);
        }
        Turn { moves: moves }
    }
}

// ----- MAIN -----

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let state: GameState = serde_json::from_str(line.expect("Could not deserialize").as_str()).unwrap();
        let turn: Turn = next_move(&state);
        eprintln!("{}", serde_json::to_string(&turn).expect("Could not serialize"));
        println!("{}", serde_json::to_string(&turn).expect("Could not serialize"));
    }
}

// ----- TESTS -----

#[test]
fn test_simulate_arrivals() {
    let leegistan = Planet {
                name: "Planeet Leegistan".to_string(),
                owner: Some(1),
                ship_count: 0,
                x: 1.0,
                y: 1.0
            };
    let neemmijover = Planet {
                name: "Planeet Neemmijover".to_string(),
                owner: None,
                ship_count: 1,
                x: 1.0,
                y: -1.0
            };
    let swaparoo = Planet {
                name: "Planeet Swaparoo".to_string(),
                owner: None,
                ship_count: 1,
                x: -1.0,
                y: 1.0
            };
    let reinforcement = Planet {
                name: "Planeet Reinforcement".to_string(),
                owner: Some(1),
                ship_count: 1,
                x: -1.0,
                y: -1.0
            };
    let dummy_gamestate: GameState = GameState {
        planets: vec![
            leegistan.clone(),
            neemmijover.clone(),
            swaparoo.clone(),
            reinforcement.clone(),
        ],
        expeditions: vec![
            Expedition {
                destination: "Planeet Neemmijover".to_string(),
                id: 1,
                origin: "Planeet Leegistan".to_string(),
                owner: 2,
                ship_count: 2,
                turns_remaining: 2,
            },
            Expedition {
                destination: "Planeet Swaparoo".to_string(),
                id: 2,
                origin: "Planeet Leegistan".to_string(),
                owner: 2,
                ship_count: 2,
                turns_remaining: 1,
            },
            Expedition {
                destination: "Planeet Swaparoo".to_string(),
                id: 3,
                origin: "Planeet Leegistan".to_string(),
                owner: 1,
                ship_count: 20,
                turns_remaining: 10,
            },
            Expedition {
                destination: "Planeet Reinforcement".to_string(),
                id: 4,
                origin: "Planeet Leegistan".to_string(),
                owner: 1,
                ship_count: 2,
                turns_remaining: 2,
            },
        ],
    };

    assert_eq!((1, 0),  simulate_arrivals(&leegistan,     &dummy_gamestate));
    assert_eq!((2, 1),  simulate_arrivals(&neemmijover,   &dummy_gamestate));
    assert_eq!((1, 10), simulate_arrivals(&swaparoo,      &dummy_gamestate));
    assert_eq!((1, 5),  simulate_arrivals(&reinforcement, &dummy_gamestate));
}

#[test]
fn test_score_fewer_ships() {
    // Same distance, fewer ships should pick fewest ships
    let homebase = Planet {
                name: "Homebase".to_string(),
                owner: Some(1),
                ship_count: 10,
                x: 0.0,
                y: 0.0
            };
    let fort = Planet {
                name: "Fort".to_string(),
                owner: Some(2),
                ship_count: 8,
                x: 1.0,
                y: 0.0
            };
    let mudhut = Planet {
                name: "Mud Hut".to_string(),
                owner: Some(2),
                ship_count: 1,
                x: -1.0,
                y: 0.0
            };
    let dummy_gamestate: GameState = GameState {
        planets: vec![
            homebase.clone(),
            fort.clone(),
            mudhut.clone(),
        ],
        expeditions: vec![],
    };

    let mudhut_score = score(&homebase, &mudhut, &dummy_gamestate);
    let fort_score   = score(&homebase, &fort,   &dummy_gamestate);
    assert!(mudhut_score.1 < fort_score.1);
}

#[test]
fn test_score_empty_planet() {
    let homebase = Planet {
                name: "Homebase".to_string(),
                owner: Some(1),
                ship_count: 10,
                x: 0.0,
                y: 0.0
            };
    let empty = Planet {
                name: "Empty".to_string(),
                owner: None,
                ship_count: 0,
                x: 1.0,
                y: 0.0
            };
    let dummy_gamestate: GameState = GameState {
        planets: vec![
            homebase.clone(),
            empty.clone(),
        ],
        expeditions: vec![],
    };

    let empty_score = score(&homebase, &empty, &dummy_gamestate).1;
    assert!(empty_score > 0);
}
