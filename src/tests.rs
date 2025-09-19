#![cfg(test)]
use std::{cmp::min, vec};

use crate::{Evaluator, Impartial};

const MAX_REMOVE: usize = 2;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
struct Kayles {
    kayles: usize,
}

impl Impartial for Kayles {
    fn get_max_nimber(&self) -> Option<usize> {
        Some(self.kayles)
    }

    fn get_split_moves(&self) -> Vec<Vec<Kayles>> {
        let mut moves = vec![];

        for i in 1..=min(self.kayles, MAX_REMOVE) {
            moves.push(vec![Kayles {
                kayles: self.kayles - i,
            }]);
        }
        // i corresponds to the number of kayles to remove
        for i in 1..=min(self.kayles.saturating_sub(2), MAX_REMOVE) {
            // j corresponds to the number of kayles on the left heap
            for j in 1..=((self.kayles - i) / 2) {
                moves.push(vec![
                    Kayles { kayles: j },
                    Kayles {
                        kayles: self.kayles - i - j,
                    },
                ]);
            }
        }
        moves
    }
}

#[test]
fn test_simple_kayle_nimbers() {
    let nimbers: Vec<usize> = vec![0, 1, 2, 3];
    let eval: Evaluator<Kayles> = Evaluator::new();

    // test the later half of the nimbers, to make sure that the evaluator can handle inputs even if
    // smaller nimbers arent already cached.
    for i in nimbers.len() / 2..nimbers.len() {
        assert_eq!(nimbers[i], eval.get_nimber(&Kayles { kayles: i }).unwrap());
    }
}

#[test]
fn test_aperiodic_kayles_nimbers() {
    // taken from the OEIS A002186
    let nimbers: Vec<usize> = vec![
        0, 1, 2, 3, 1, 4, 3, 2, 1, 4, 2, 6, 4, 1, 2, 7, 1, 4, 3, 2, 1, 4, 6, 7, 4, 1, 2, 8, 5, 4,
        7, 2, 1, 8, 6, 7, 4, 1, 2, 3, 1, 4, 7, 2, 1, 8, 2, 7, 4, 1, 2, 8, 1, 4, 7, 2, 1, 4, 2, 7,
        4, 1, 2, 8, 1, 4, 7, 2, 1, 8, 6, 7, 4, 1, 2, 8, 1, 4, 7, 2, 1, 8, 2, 7, 4, 1, 2, 8, 1, 4,
        7, 2, 1, 8, 2, 7, 4, 1, 2, 8, 1, 4, 7, 2, 1,
    ];
    let eval: Evaluator<Kayles> = Evaluator::new();

    // test the later half of the nimbers, to make sure that the evaluator can handle inputs even if
    // smaller nimbers arent already cached.
    for i in nimbers.len() / 2..nimbers.len() {
        assert_eq!(nimbers[i], eval.get_nimber(&Kayles { kayles: i }).unwrap());
    }
}
#[test]
fn test_cancellation() {
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::time::Duration;

    // Start with a fresh evaluator and evaluate a complex Kayles position
    let eval = Evaluator::new();
    let target = Kayles { kayles: 200 };

    // Spawn a thread to simulate cancellation after a short delay
    let cancel_flag1 = eval.cancel_flag.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(5));
        cancel_flag1.store(true, Ordering::Relaxed);
    });

    // Attempt to evaluate, expecting it to be cancelled
    let result = eval.get_nimber(&target);
    assert_eq!(result, None, "Evaluation should be cancelled");

    // Reset the cancellation flag
    eval.cancel_flag.store(false, Ordering::Relaxed);

    // Try again — should continue from cached state
    let result2 = eval.get_nimber(&target);
    assert!(
        result2.is_some(),
        "Evaluation should complete after resuming"
    );

    // Cache should now be valid; validate result against a clean evaluator
    let fresh_eval = Evaluator::new();
    let expected = fresh_eval.get_nimber(&target).unwrap();
    assert_eq!(
        result2.unwrap(),
        expected,
        "Nimber after cancellation-resume should match fresh evaluation"
    );

    // Do another cancellation-resume cycle on a new, larger input
    let eval2 = eval.clone();
    let new_target = Kayles { kayles: 300 };
    let cancel_flag2 = eval2.cancel_flag.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(5));
        cancel_flag2.store(true, Ordering::Relaxed);
    });

    let result3 = eval2.get_nimber(&new_target);
    assert_eq!(result3, None, "Second cancellation should also interrupt");

    eval2.cancel_flag.store(false, Ordering::Relaxed);
    let cancel_flag3 = eval2.cancel_flag.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(5));
        cancel_flag3.store(true, Ordering::Relaxed);
    });
    let result4 = eval2.get_nimber(&new_target);
    assert_eq!(result4, None, "Third cancellation should still interrupt");

    eval2.cancel_flag.store(false, Ordering::Relaxed);
    let result5 = eval2.get_nimber(&new_target).unwrap();

    let fresh_eval2 = Evaluator::new();
    let expected2 = fresh_eval2.get_nimber(&new_target).unwrap();
    assert_eq!(
        result5, expected2,
        "Result after second resume should match fresh computation"
    );
}
