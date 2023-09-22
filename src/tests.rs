#![cfg(test)]
use std::vec;

use crate::{Evaluator, Impartial};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
struct Kayles {
    kayles: Vec<u16>,
}

impl Impartial<Kayles> for Kayles {
    fn get_parts(self) -> Vec<Kayles> {
        return self
            .kayles
            .iter()
            .map(|n| Kayles { kayles: vec![*n] })
            .collect();
    }

    fn get_max_nimber(&self) -> u16 {
        return self.kayles.iter().sum();
    }

    fn get_unique_moves(&self) -> Vec<Kayles> {
        let mut moves: Vec<Kayles> = vec![];
        for i in 0..self.kayles.len() {
            let size = (self.kayles[i] + 1) / 2;
            for j in 1..=size {
                let mut _move = self.kayles.clone();
                _move[i] -= j;
                _move.push(j - 1);
                moves.push(Kayles {
                    kayles: _move.into_iter().filter(|x| *x != 0).collect(),
                });
            }
            let size = (self.kayles[i] + 2) / 2;
            for j in 2..=size {
                let mut _move = self.kayles.clone();
                _move[i] -= j;
                _move.push(j - 2);
                moves.push(Kayles {
                    kayles: _move.into_iter().filter(|x| *x != 0).collect(),
                });
            }
        }
        return moves;
    }
}

#[test]
fn test_aperiodic_kayles_nimbers() {
    let nimbers: Vec<u16> = vec![
        0, 1, 2, 3, 1, 4, 3, 2, 1, 4, 2, 6, 4, 1, 2, 7, 1, 4, 3, 2, 1, 4, 6, 7, 4, 1, 2, 8, 5, 4,
        7, 2, 1, 8, 6, 7, 4, 1, 2, 3, 1, 4, 7, 2, 1, 8, 2, 7, 4, 1, 2, 8, 1, 4, 7, 2, 1, 4, 2, 7,
        4, 1, 2, 8, 1, 4, 7, 2, 1, 8, 6, 7, 4, 1, 2, 8, 1, 4, 7, 2, 1, 8, 2, 7, 4, 1, 2, 8, 1, 4,
        7, 2, 1, 8, 2, 7, 4, 1, 2, 8, 1, 4, 7, 2, 1
    ];
    let mut eval: Evaluator<Kayles> = Evaluator::new();

    for i in nimbers.len()/2..nimbers.len() {
        assert_eq!(
            nimbers[i],
            eval.get_nimber(Kayles {
                kayles: vec![i as u16]
            })
        );
    }
}
#[test]
fn test_bound_creation_for_last_part_calculations() {
    for bound in 0..10 as usize {
        for modifier in 0..(5 * bound) {
            let mut detected_nimbers = vec![];
            for i in 0..=(bound + modifier) {
                detected_nimbers.push(i ^ modifier);
            }
            detected_nimbers.sort();
            assert_eq!(detected_nimbers[bound], bound);
        }
    }
}
