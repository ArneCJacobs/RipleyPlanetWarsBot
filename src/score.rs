use crate::data::PlayerId;
use crate::state::State;

/// The design score, lower is better, built from two `+`/`-` only terms:
///
/// 1. Total ship score: sum of all enemy ships minus sum of all our ships. Neutral planets are not
///    counted (a fair fight on a neutral destroys one of our ships per neutral ship, leaving
///    `enemy - allied` unchanged, so counting them makes neutral attacks score-neutral).
///
/// 2. Static defence-ability: for every planet `p`, each other planet `pi` exerts an influence of
///    `pi.ships / distance(p, pi)^2` (inverse-square, like gravity). Our influence lowers the score
///    (a location is well defended when our forces are near and strong); the enemy's raises it. This
///    couples ship count with position, so concentrating force near the enemy is preferred.
pub fn get_score_state(me_id: PlayerId, state: &State) -> f64 {
    let planets = &state.current_state.planets;
    let mut score = 0.0;

    // 1. total ship score
    for planet in planets {
        match planet.owner {
            Some(owner) if owner == me_id => score -= planet.ship_count as f64,
            Some(_) => score += planet.ship_count as f64,
            None => {}
        }
    }

    // 2. static defence-ability (inverse-square influence)
    for p in planets {
        for pi in planets {
            if p.name == pi.name {
                continue; // a planet exerts no influence on itself (distance 0)
            }
            let distance = p.distance(pi) as f64;
            let influence = pi.ship_count as f64 / (distance * distance);
            match pi.owner {
                Some(owner) if owner == me_id => score -= influence,
                Some(_) => score += influence,
                None => {}
            }
        }
    }

    score
}

#[cfg(test)]
mod test {
    use crate::data::{Expedition, Input, Move, Planet, ME_ID, OTHER_ID};
    use crate::score::get_score_state;
    use crate::state::{State, apply_simulated_moves};

    fn planet(name: &str, x: f32, y: f32, owner: Option<u8>, ship_count: i64) -> Planet {
        Planet { ship_count, x, y, owner, name: name.to_string(), index: 0 }
    }

    /// Two neutral planets equidistant from our start, both capturable with the ships we hold.
    /// Capturing the one with fewer ships costs fewer of ours, so after 20 turns of growth it leaves
    /// more allied ships and therefore a lower (better) score than capturing the stronger one.
    #[test]
    fn capturing_the_weaker_of_two_equidistant_neutrals_scores_better() {
        let base = State::new(Input {
            planets: vec![
                planet("home", 0.0, 0.0, Some(ME_ID), 20),
                planet("weak", 10.0, 0.0, None, 2),
                planet("strong", -10.0, 0.0, None, 10),
            ],
            expeditions: vec![],
        });

        let attack_weak = vec![Move::new("home".to_string(), "weak".to_string(), 11)];
        let attack_strong = vec![Move::new("home".to_string(), "strong".to_string(), 11)];

        let score_weak = get_score_state(ME_ID, &apply_simulated_moves(ME_ID, &attack_weak, base.clone()).advance(20));
        let score_strong = get_score_state(ME_ID, &apply_simulated_moves(ME_ID, &attack_strong, base.clone()).advance(20));

        assert!(
            score_weak < score_strong,
            "capturing the weaker neutral should score better (lower): weak={score_weak}, strong={score_strong}"
        );
    }

    /// An allied and an enemy planet, equal ships and equidistant from home, both reachable.
    /// Capturing the enemy removes its ships from the enemy total (a double swing), so it scores
    /// better (lower) than merely reinforcing the ally, which only shuffles ships we already own.
    #[test]
    fn capturing_an_enemy_scores_better_than_reinforcing_an_ally() {
        let base = State::new(Input {
            planets: vec![
                planet("home", 0.0, 0.0, Some(ME_ID), 20),
                planet("ally", -1.0, 0.0, Some(ME_ID), 5),
                planet("enemy", 1.0, 0.0, Some(OTHER_ID), 5),
            ],
            expeditions: vec![],
        });

        let capture_enemy = vec![Move::new("home".to_string(), "enemy".to_string(), 10)];
        let reinforce_ally = vec![Move::new("home".to_string(), "ally".to_string(), 10)];

        let score_capture = get_score_state(ME_ID, &apply_simulated_moves(ME_ID, &capture_enemy, base.clone()).advance(20));
        let score_reinforce = get_score_state(ME_ID, &apply_simulated_moves(ME_ID, &reinforce_ally, base.clone()).advance(20));

        assert!(
            score_capture < score_reinforce,
            "capturing the enemy should score better (lower) than reinforcing the ally: capture={score_capture}, reinforce={score_reinforce}"
        );
    }

    /// A neutral and an enemy planet, equal ships and equidistant from home, both capturable.
    /// Capturing the enemy removes it from the enemy total; capturing the neutral leaves the enemy
    /// alive and growing. So capturing the enemy scores better (lower).
    #[test]
    fn capturing_an_enemy_scores_better_than_capturing_an_equal_neutral() {
        let base = State::new(Input {
            planets: vec![
                planet("home", 0.0, 0.0, Some(ME_ID), 20),
                planet("neutral", -1.0, 0.0, None, 5),
                planet("enemy", 1.0, 0.0, Some(OTHER_ID), 5),
            ],
            expeditions: vec![],
        });

        let capture_enemy = vec![Move::new("home".to_string(), "enemy".to_string(), 10)];
        let capture_neutral = vec![Move::new("home".to_string(), "neutral".to_string(), 10)];

        let score_enemy = get_score_state(ME_ID, &apply_simulated_moves(ME_ID, &capture_enemy, base.clone()).advance(20));
        let score_neutral = get_score_state(ME_ID, &apply_simulated_moves(ME_ID, &capture_neutral, base.clone()).advance(20));

        assert!(
            score_enemy < score_neutral,
            "capturing the enemy should score better (lower) than capturing an equal neutral: enemy={score_enemy}, neutral={score_neutral}"
        );
    }

    /// Home plus two neutrals: a close one that is harder (more ships, home can't afford it yet) and
    /// a far one that is easier (fewer ships, affordable now). Waiting to grow and then taking the
    /// close-hard planet still captures it sooner (turn 6) than the far-easy one whose ships crawl
    /// across the map (turn 20). Owned sooner means more growth by the horizon, so the close capture
    /// scores better (lower) despite the wait and the higher cost.
    #[test]
    fn capturing_a_closer_harder_neutral_beats_a_far_easy_one_when_owned_sooner() {
        const HORIZON: i64 = 30;
        let base = State::new(Input {
            planets: vec![
                planet("home", 0.0, 0.0, Some(ME_ID), 10),
                planet("close", 3.0, 0.0, None, 12),
                planet("far", 20.0, 0.0, None, 8),
            ],
            expeditions: vec![],
        });

        // immediately capture the far, easy planet
        let far = apply_simulated_moves(ME_ID, &vec![Move::new("home".to_string(), "far".to_string(), 10)], base.clone())
            .advance(HORIZON);

        // wait 3 turns to grow to 13, then capture the close, hard planet
        let close = base.clone().advance(3);
        let close = apply_simulated_moves(ME_ID, &vec![Move::new("home".to_string(), "close".to_string(), 13)], close)
            .advance(HORIZON - 3);

        let score_close = get_score_state(ME_ID, &close);
        let score_far = get_score_state(ME_ID, &far);

        assert!(
            score_close < score_far,
            "waiting for the closer harder planet (owned sooner) should score better (lower): close={score_close}, far={score_far}"
        );
    }

    /// Home and a neutral with far too many ships to capture. Attacking it only throws our ships
    /// away (destroyed in a fight we can't win, the neutral never flips), leaving fewer allied ships
    /// than simply holding. So holding scores better (lower) and the score incentivises the hold.
    #[test]
    fn holding_beats_attacking_an_uncapturable_neutral() {
        let base = State::new(Input {
            planets: vec![
                planet("home", 0.0, 0.0, Some(ME_ID), 10),
                planet("neutral", 5.0, 0.0, None, 100),
            ],
            expeditions: vec![],
        });

        let hold = apply_simulated_moves(ME_ID, &vec![], base.clone()).advance(20);
        let attack = apply_simulated_moves(ME_ID, &vec![Move::new("home".to_string(), "neutral".to_string(), 10)], base.clone())
            .advance(20);

        let score_hold = get_score_state(ME_ID, &hold);
        let score_attack = get_score_state(ME_ID, &attack);

        assert!(
            score_hold < score_attack,
            "holding should score better (lower) than attacking an uncapturable neutral: hold={score_hold}, attack={score_attack}"
        );
    }

    /// Five planets in a row `e a a a e`. The left ally faces an incoming enemy force it cannot
    /// survive alone; the right ally can defend its own incoming attack. The middle ally has surplus
    /// to send to one side. Reinforcing the threatened left ally saves it from capture (it stays
    /// ours); reinforcing the already-safe right ally wastes the ships and lets the left ally fall to
    /// the enemy. So reinforcing the left ally scores better (lower).
    #[test]
    fn reinforcing_the_threatened_ally_beats_reinforcing_the_safe_one() {
        let base = State::new(Input {
            planets: vec![
                planet("eL", 0.0, 0.0, Some(OTHER_ID), 20),
                planet("aL", 10.0, 0.0, Some(ME_ID), 3),
                planet("aM", 20.0, 0.0, Some(ME_ID), 30),
                planet("aR", 30.0, 0.0, Some(ME_ID), 15),
                planet("eR", 40.0, 0.0, Some(OTHER_ID), 20),
            ],
            expeditions: vec![
                Expedition { id: 1, ship_count: 20, origin: "eL".to_string(), destination: "aL".to_string(), owner: OTHER_ID, turns_remaining: 10 },
                Expedition { id: 2, ship_count: 5, origin: "eR".to_string(), destination: "aR".to_string(), owner: OTHER_ID, turns_remaining: 10 },
            ],
        });

        let reinforce_left = apply_simulated_moves(ME_ID, &vec![Move::new("aM".to_string(), "aL".to_string(), 15)], base.clone())
            .advance(20);
        let reinforce_right = apply_simulated_moves(ME_ID, &vec![Move::new("aM".to_string(), "aR".to_string(), 15)], base.clone())
            .advance(20);

        let score_left = get_score_state(ME_ID, &reinforce_left);
        let score_right = get_score_state(ME_ID, &reinforce_right);

        assert!(
            score_left < score_right,
            "reinforcing the threatened left ally should score better (lower): left={score_left}, right={score_right}"
        );
    }

    /// Two allies, each too weak to take a neutral alone, and two neutrals. Combining both allies on
    /// one neutral captures it (the waves stack: 6 leaves it at 4, the next 6 takes it); splitting
    /// one ally per neutral captures neither and wastes the ships. The capture grows into extra
    /// allied ships, so combining scores better (lower).
    #[test]
    fn combining_both_allies_to_capture_one_neutral_beats_splitting() {
        let base = State::new(Input {
            planets: vec![
                planet("a1", 0.0, 10.0, Some(ME_ID), 6),
                planet("a2", 10.0, 10.0, Some(ME_ID), 6),
                planet("n1", 0.0, 0.0, None, 10),
                planet("n2", 10.0, 0.0, None, 10),
            ],
            expeditions: vec![],
        });

        let combine = apply_simulated_moves(ME_ID, &vec![
            Move::new("a1".to_string(), "n1".to_string(), 6),
            Move::new("a2".to_string(), "n1".to_string(), 6),
        ], base.clone()).advance(20);

        let split = apply_simulated_moves(ME_ID, &vec![
            Move::new("a1".to_string(), "n1".to_string(), 6),
            Move::new("a2".to_string(), "n2".to_string(), 6),
        ], base.clone()).advance(20);

        let score_combine = get_score_state(ME_ID, &combine);
        let score_split = get_score_state(ME_ID, &split);

        assert!(
            score_combine < score_split,
            "combining both allies to capture one neutral should score better (lower): combine={score_combine}, split={score_split}"
        );
    }

    /// Home can take either a near single neutral (10 ships, captured early) or a far cluster of two
    /// small neutrals (captured much later). Over a long horizon two planets grow more ships than
    /// one, so despite the later capture and slightly higher cost the cluster scores better (lower).
    #[test]
    fn a_far_two_planet_cluster_beats_a_near_single_neutral_in_the_long_run() {
        const HORIZON: i64 = 100;
        let base = State::new(Input {
            planets: vec![
                planet("home", 0.0, 0.0, Some(ME_ID), 30),
                planet("near", 2.0, 0.0, None, 10),
                planet("c1", 20.0, 0.0, None, 5),
                planet("c2", 20.0, 2.0, None, 5),
            ],
            expeditions: vec![],
        });

        let take_near = apply_simulated_moves(ME_ID, &vec![Move::new("home".to_string(), "near".to_string(), 11)], base.clone())
            .advance(HORIZON);
        let take_cluster = apply_simulated_moves(ME_ID, &vec![
            Move::new("home".to_string(), "c1".to_string(), 6),
            Move::new("home".to_string(), "c2".to_string(), 6),
        ], base.clone()).advance(HORIZON);

        let score_near = get_score_state(ME_ID, &take_near);
        let score_cluster = get_score_state(ME_ID, &take_cluster);

        assert!(
            score_cluster < score_near,
            "taking the far two-planet cluster should score better (lower) in the long run: cluster={score_cluster}, near={score_near}"
        );
    }

    /// One enemy and two allies, one near the enemy and one far. The only expedition is the far ally
    /// reinforcing the near one; the ships arrive well within the horizon. Concentrating force at the
    /// front (near the enemy) should score better than leaving it idle in the rear, so reinforcing
    /// should score better (lower) than everyone holding.
    #[test]
    fn reinforcing_a_front_ally_scores_better_than_holding() {
        let base = State::new(Input {
            planets: vec![
                planet("enemy", 0.0, 0.0, Some(OTHER_ID), 10),
                planet("close", 5.0, 0.0, Some(ME_ID), 5),
                planet("far", 30.0, 0.0, Some(ME_ID), 20),
            ],
            expeditions: vec![],
        });

        // horizon well past the far->close travel (25) so the reinforcement lands and counts
        let hold = apply_simulated_moves(ME_ID, &vec![], base.clone()).advance(40);
        let reinforce = apply_simulated_moves(ME_ID, &vec![Move::new("far".to_string(), "close".to_string(), 10)], base.clone())
            .advance(40);

        let score_hold = get_score_state(ME_ID, &hold);
        let score_reinforce = get_score_state(ME_ID, &reinforce);

        assert!(
            score_reinforce < score_hold,
            "reinforcing the front ally should score better (lower) than holding: reinforce={score_reinforce}, hold={score_hold}"
        );
    }

    /// Home faces an incoming attack it can just survive by holding. Sending its garrison off to
    /// grab a neutral leaves it too weak and it falls to the enemy. Losing home (a planet plus its
    /// ships handed to the enemy) is far worse than skipping the neutral, so holding scores better.
    #[test]
    fn stripping_a_threatened_planet_to_grab_a_neutral_is_worse_than_holding() {
        let base = State::new(Input {
            planets: vec![
                planet("home", 0.0, 0.0, Some(ME_ID), 10),
                planet("neutral", 3.0, 0.0, None, 3),
                planet("enemy", 10.0, 0.0, Some(OTHER_ID), 10),
            ],
            expeditions: vec![
                Expedition { id: 1, ship_count: 12, origin: "enemy".to_string(), destination: "home".to_string(), owner: OTHER_ID, turns_remaining: 5 },
            ],
        });

        let hold = apply_simulated_moves(ME_ID, &vec![], base.clone()).advance(20);
        let strip = apply_simulated_moves(ME_ID, &vec![Move::new("home".to_string(), "neutral".to_string(), 6)], base.clone())
            .advance(20);

        let score_hold = get_score_state(ME_ID, &hold);
        let score_strip = get_score_state(ME_ID, &strip);

        assert!(
            score_hold < score_strip,
            "stripping the threatened home to grab a neutral should score worse than holding: hold={score_hold}, strip={score_strip}"
        );
    }

    /// A near neutral is capturable, but the enemy already has an expedition inbound that will retake
    /// it right after we do. Grabbing it spends ships to briefly own a planet the enemy takes anyway,
    /// so it is worse than holding (which loses nothing and lets the enemy spend on the neutral).
    #[test]
    fn capturing_a_planet_you_will_immediately_lose_is_worse_than_holding() {
        let base = State::new(Input {
            planets: vec![
                planet("home", 0.0, 0.0, Some(ME_ID), 20),
                planet("neutral", 5.0, 0.0, None, 5),
                planet("enemy", 10.0, 0.0, Some(OTHER_ID), 10),
            ],
            expeditions: vec![
                Expedition { id: 1, ship_count: 10, origin: "enemy".to_string(), destination: "neutral".to_string(), owner: OTHER_ID, turns_remaining: 8 },
            ],
        });

        let hold = apply_simulated_moves(ME_ID, &vec![], base.clone()).advance(20);
        let overextend = apply_simulated_moves(ME_ID, &vec![Move::new("home".to_string(), "neutral".to_string(), 6)], base.clone())
            .advance(20);

        let score_hold = get_score_state(ME_ID, &hold);
        let score_overextend = get_score_state(ME_ID, &overextend);

        assert!(
            score_hold < score_overextend,
            "grabbing a planet the enemy immediately retakes should score worse than holding: hold={score_hold}, overextend={score_overextend}"
        );
    }
}
