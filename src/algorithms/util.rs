use crate::data::{Expedition, Planet, PlayerId};

pub fn simulate_expeditions(
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
