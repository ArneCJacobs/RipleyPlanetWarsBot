use std::fs::File;

use RipleyPlanetWarsBot::{
    algorithms::ripley_greedy_optimization::{RipleyGreedyOptimization, add_loopback_moves, map_horizon, neighbour},
    data::{Input, Move, ME_ID},
    state::State,
};

fn threat_and_shortfall(state: &State, planet_name: &str) -> (f64, f64, i64) {
    let planet = state.current_state.planets.iter().find(|p| p.name == planet_name).unwrap();
    let mut threat = 0.0;
    for other in &state.current_state.planets {
        if other.owner.is_some() && other.owner != Some(ME_ID) {
            let travel = planet.distance(other).ceil() as f64;
            threat += other.ship_count as f64 / (1.0 + travel);
        }
    }
    let shortfall = (threat - planet.ship_count as f64).max(0.0);
    (threat, shortfall, planet.ship_count)
}

fn main() {
    let file = File::open("spiral85.json").unwrap();
    let input: Input = serde_json::from_reader(file).unwrap();
    let state = State::new(input);
    let horizon = map_horizon(&state);
    let mut bot = RipleyGreedyOptimization::new(ME_ID);

    eprintln!("horizon={horizon}");
    eprintln!("--- my planets: ships / threat / shortfall ---");
    let mine: Vec<String> = state.current_state.planets.iter()
        .filter(|p| p.owner == Some(ME_ID)).map(|p| p.name.clone()).collect();
    for name in &mine {
        let (threat, shortfall, ships) = threat_and_shortfall(&state, name);
        eprintln!("  {name:8} ships={ships:3} threat={threat:6.1} shortfall={shortfall:6.1}");
    }

    // safest = biggest margin (ships - threat); most threatened = biggest shortfall
    let source = mine.iter().max_by(|a, b| {
        let (ta, _, sa) = threat_and_shortfall(&state, a);
        let (tb, _, sb) = threat_and_shortfall(&state, b);
        (sa as f64 - ta).partial_cmp(&(sb as f64 - tb)).unwrap()
    }).unwrap().clone();
    let target = mine.iter().max_by(|a, b| {
        threat_and_shortfall(&state, a).1.partial_cmp(&threat_and_shortfall(&state, b).1).unwrap()
    }).unwrap().clone();
    let source_ships = threat_and_shortfall(&state, &source).2;
    eprintln!("\nsource (safest) = {source} ({source_ships} ships), target (most threatened) = {target}");

    // hold everything
    let mut hold = vec![];
    add_loopback_moves(ME_ID, &state, &mut hold);
    // reinforce: source sends all its ships to target
    let mut reinforce = vec![Move::new(source.clone(), target.clone(), source_ships)];
    add_loopback_moves(ME_ID, &state, &mut reinforce);
    let src = state.current_state.planets.iter().find(|p| p.name == source).unwrap();
    let tgt = state.current_state.planets.iter().find(|p| p.name == target).unwrap();
    let arrival = src.distance(tgt).ceil() as i64;

    eprintln!("\n--- rollout score (lower better) ---");
    eprintln!("hold                      = {:.1}", bot.score_candidate(&state, &hold, 0, horizon));
    eprintln!("reinforce {source}->{target} = {:.1}", bot.score_candidate(&state, &reinforce, arrival, horizon));

    // does the search ever propose reinforcing the target from the safe source?
    let seed = hold.clone();
    let samples = 20000;
    let mut to_target = 0;
    let mut from_source = 0;
    for _ in 0..samples {
        let (moves, _) = neighbour(&state, &seed);
        if moves.iter().any(|m| m.destination == target && m.origin != target) { to_target += 1; }
        if moves.iter().any(|m| m.origin == source && m.destination != source) { from_source += 1; }
    }
    eprintln!("\n--- {samples} neighbour samples from the hold seed ---");
    eprintln!("proposals containing a move TO the threatened target {target}: {to_target}");
    eprintln!("proposals moving ships OFF the safe source {source}:        {from_source}");
}
