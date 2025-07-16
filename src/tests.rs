#![cfg(test)]
use std::vec;

use crate::{Evaluator, Impartial};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
struct Kayles {
    kayles: Vec<usize>,
}

impl Impartial<Kayles> for Kayles {
    fn get_parts(&self) -> Option<Vec<Kayles>> {
        Some(self
            .kayles
            .iter()
            .map(|n| Kayles { kayles: vec![*n] })
            .collect())
    }

    fn get_max_nimber(&self) -> Option<usize> {
        Some(self.kayles.iter().sum())
    }

    fn get_moves(&self) -> Vec<Kayles> {
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
    // taken from the OEIS A002186
    let nimbers: Vec<usize> = vec![
        0, 1, 2, 3, 1, 4, 3, 2, 1, 4, 2, 6, 4, 1, 2, 7, 1, 4, 3, 2, 1, 4, 6, 7, 4, 1, 2, 8, 5, 4,
        7, 2, 1, 8, 6, 7, 4, 1, 2, 3, 1, 4, 7, 2, 1, 8, 2, 7, 4, 1, 2, 8, 1, 4, 7, 2, 1, 4, 2, 7,
        4, 1, 2, 8, 1, 4, 7, 2, 1, 8, 6, 7, 4, 1, 2, 8, 1, 4, 7, 2, 1, 8, 2, 7, 4, 1, 2, 8, 1, 4,
        7, 2, 1, 8, 2, 7, 4, 1, 2, 8, 1, 4, 7, 2, 1
    ];
    let mut eval: Evaluator<Kayles> = Evaluator::new();

    // test the later half of the nimbers, to make sure that the evaluator can handle inputs even if
    // smaller nimbers arent already cached.
    for i in nimbers.len()/2..nimbers.len() {
        assert_eq!(
            nimbers[i],
            eval.get_nimber(Kayles {
                kayles: vec![i]
            }).unwrap()
        );
    }
}
#[test]
fn test_cancellation() {
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::time::Duration;

    // Start with a fresh evaluator and evaluate a complex Kayles position
    let mut eval = Evaluator::new();
    let target = Kayles { kayles: vec![100] };

    // Spawn a thread to simulate cancellation after a short delay
    let cancel_flag1 = eval.cancel_flag.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(5));
        cancel_flag1.store(true, Ordering::Relaxed);
    });

    // Attempt to evaluate, expecting it to be cancelled
    let result = eval.get_nimber(target.clone());
    assert_eq!(result, None, "Evaluation should be cancelled");

    // Reset the cancellation flag
    eval.cancel_flag.store(false, Ordering::Relaxed);

    // Try again — should continue from cached state
    let result2 = eval.get_nimber(target.clone());
    assert!(result2.is_some(), "Evaluation should complete after resuming");

    // Cache should now be valid; validate result against a clean evaluator
    let mut fresh_eval = Evaluator::new();
    let expected = fresh_eval.get_nimber(target.clone()).unwrap();
    assert_eq!(
        result2.unwrap(),
        expected,
        "Nimber after cancellation-resume should match fresh evaluation"
    );

    // Do another cancellation-resume cycle on a new, larger input
    let mut eval2 = eval.clone();
    let new_target = Kayles { kayles: vec![200] };
    let cancel_flag2 = eval2.cancel_flag.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        cancel_flag2.store(true, Ordering::Relaxed);
    });

    let result3 = eval2.get_nimber(new_target.clone());
    assert_eq!(result3, None, "Second cancellation should also interrupt");

    eval2.cancel_flag.store(false, Ordering::Relaxed);
    let cancel_flag3 = eval2.cancel_flag.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        cancel_flag3.store(true, Ordering::Relaxed);
    });
    let result4 = eval2.get_nimber(new_target.clone());
    assert_eq!(result4, None, "Third cancellation should still interrupt");

    eval2.cancel_flag.store(false, Ordering::Relaxed);
    let result5 = eval2.get_nimber(new_target.clone()).unwrap();

    let mut fresh_eval2 = Evaluator::new();
    let expected2 = fresh_eval2.get_nimber(new_target.clone()).unwrap();
    assert_eq!(
        result5,
        expected2,
        "Result after second resume should match fresh computation"
    );
}
