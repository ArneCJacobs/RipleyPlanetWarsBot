use crate::data::Move;


pub fn consolidate_moves(mut moves: Vec<Move>) -> Vec<Move> {
    moves.sort_unstable_by_key(|mv| (mv.origin.clone(), mv.destination.clone()));

    let mut new_moves = Vec::new();
    let mut index = 0;
    while index < moves.len() {
        let mut mv = moves[index].clone();
        let mut run_last_index = index + 1; 
        for (other_index, other_mv) in moves.iter().enumerate().skip(index+1) {
            if other_mv.origin == mv.origin && other_mv.destination == mv.destination {
                run_last_index = other_index + 1;
                mv.ship_count += other_mv.ship_count;
            } else {
                break;
            }
        };

        new_moves.push(mv);
        index = run_last_index;

    }
    new_moves.into_iter().filter(|mv| mv.ship_count != 0).collect()
}

#[cfg(test)]
mod test {
    use crate::{data::Move, utils::consolidate_moves};

    #[test]
    fn consolidate_failure_case_1() {
        let moves_before = vec![
            Move { origin: "1-3".to_string(), destination: "1-2".to_string(), ship_count: 1 },
            Move { origin: "1-4".to_string(), destination: "1-2".to_string(), ship_count: 1 },
            Move { origin: "1-5".to_string(), destination: "1-2".to_string(), ship_count: 1 },
            Move { origin: "1-6".to_string(), destination: "1-2".to_string(), ship_count: 1 },
            Move { origin: "2-4".to_string(), destination: "2-6".to_string(), ship_count: 1 },
            Move { origin: "2-5".to_string(), destination: "2-6".to_string(), ship_count: 1 },
            Move { origin: "1-3".to_string(), destination: "1-2".to_string(), ship_count: 1 },
        ];

        let actual = consolidate_moves(moves_before);

        let moves_expected = vec![
            Move { origin: "1-3".to_string(), destination: "1-2".to_string(), ship_count: 2 },
            Move { origin: "1-4".to_string(), destination: "1-2".to_string(), ship_count: 1 },
            Move { origin: "1-5".to_string(), destination: "1-2".to_string(), ship_count: 1 },
            Move { origin: "1-6".to_string(), destination: "1-2".to_string(), ship_count: 1 },
            Move { origin: "2-4".to_string(), destination: "2-6".to_string(), ship_count: 1 },
            Move { origin: "2-5".to_string(), destination: "2-6".to_string(), ship_count: 1 },
        ];

        assert_eq!(actual, moves_expected);

    }

    #[test]
    fn consolidate_failure_case_2() {
        let moves_before = vec![
            Move { origin: "1-3".to_string(), destination: "1-2".to_string(), ship_count: 1 },
            Move { origin: "1-3".to_string(), destination: "1-2".to_string(), ship_count: 1 },
        ];

        let actual = consolidate_moves(moves_before);

        let moves_expected = vec![
            Move { origin: "1-3".to_string(), destination: "1-2".to_string(), ship_count: 2 },
        ];

        assert_eq!(actual, moves_expected);

    }
}
