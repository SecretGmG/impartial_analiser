mod entry;
use dashmap::DashMap;
use entry::Entry;
use std::hash::Hash;
use std::{
    hash::{DefaultHasher, Hasher},
    ptr::hash,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    usize,
};

mod tests;

use crate::entry::{EntryData, ProcessingData};

/// Provides the interface for evaluating an impartial game with the `Evaluator`.
pub trait Impartial<G>: Sized + Clone + Hash + Eq
where
    G: Impartial<Self>,
{
    /// Returns the components (subgames) of the game, if any.
    fn get_parts(&self) -> Option<Vec<G>>;

    /// Returns the maximum nimber this game could have, if known.
    fn get_max_nimber(&self) -> Option<usize> {
        None
    }

    /// Returns the list of successor game states (i.e., possible moves).
    fn get_moves(&self) -> Vec<G>;
}

/// Evaluates impartial games via memoized recursive computation of nimbers.
///
/// `G` is the game type, which must implement `Impartial<G>`.
#[derive(Debug, Clone)]
pub struct Evaluator<G>
where
    G: Impartial<G>,
{
    cache: Arc<DashMap<G, Entry<G>>>,
    cancel_flag: Arc<AtomicBool>,
}

impl<G> Evaluator<G>
where
    G: Impartial<G>,
{
    /// Constructs a new, empty evaluator.
    pub fn new() -> Evaluator<G> {
        Evaluator {
            cache: Arc::new(DashMap::new()),
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns a list of all known positions with their computed nimbers.
    pub fn get_nimbers(&self) -> Vec<(G, usize)> {
        self.cache
            .iter()
            .filter_map(|e| {
                let nimber = match e.data {
                    EntryData::Done { nimber } => Some(nimber),
                    _ => None,
                }?;
                Some((e.key().clone(), nimber))
            })
            .collect()
    }

    /// Returns the number of entries stored in the evaluator cache.
    pub fn get_cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Returns a handle to the evaluator’s cancellation flag.
    /// Can be set externally to abort ongoing computation.
    pub fn get_cancel_flag(&self) -> Arc<AtomicBool> {
        self.cancel_flag.clone()
    }

    /// Computes the nimber of the given game.
    /// Returns `None` if cancelled mid-computation.
    pub fn get_nimber(&self, g: &G) -> Option<usize> {
        self.get_bounded_nimber(g, usize::MAX)
    }

    /// Computes the nimber of a game, but aborts early if it can be proven
    /// that the nimber exceeds the provided upper bound.
    pub fn get_bounded_nimber(&self, g: &G, bound: usize) -> Option<usize> {
        let parts = g.get_parts().unwrap_or(vec![g.clone()]);
        self.get_bounded_nimber_by_parts(&parts, bound)
    }

    /// Computes the nimber of a specific game part with an upper bound.
    /// Returns `None` if cancelled or if nimber exceeds the bound.
    fn get_bounded_nimber_of_part(&self, part: &G, bound: usize) -> Option<usize> {
        if !self.cache.contains_key(part) {
            self.cache
                .insert(part.clone(), Entry::new(part.get_max_nimber()));
        }

        match &self.cache.get(part).unwrap().data {
            EntryData::Done { nimber } => return Some(*nimber),
            _ => (),
        }

        self.destub(part);

        loop {
            if self.cancel_flag.load(Ordering::Relaxed) {
                return None;
            }

            let nimber = {
                let entry = self.cache.get(part).unwrap();
                entry.get_smallest_possible_nimber().unwrap()
            };

            if nimber > bound {
                return None;
            }

            if !self.try_rule_out_nimber(part, nimber)? {
                {
                    let mut entry = self.cache.get_mut(part).unwrap();
                    entry.data = EntryData::Done { nimber };
                }
                return Some(nimber);
            }
        }
    }

    /// Attempts to prove that the given `nimber` cannot be the nimber of `game`.
    /// Returns `Some(true)` if it was successfully ruled out,
    /// `Some(false)` if the `nimber` is actually valid,
    /// and `None` if cancelled before a conclusion.
    fn try_rule_out_nimber(&self, game: &G, nimber: usize) -> Option<bool> {
        if let Some(max_nimber) = self.cache.get(game)?.max_nimber {
            if max_nimber < nimber {
                return Some(false);
            }
        }

        let mut still_unprocessed_move_indices = vec![];
        let mut ruled_out_nimber = false;

        loop {
            let parts_opt = {
                let mut guard = self.cache.get_mut(game)?;
                guard.pop_unprocessed_move().unwrap()
            };

            let Some(parts) = parts_opt else { break };

            if self.cancel_flag.load(Ordering::Relaxed) {
                return None;
            }

            match self.get_bounded_nimber_by_parts(&parts, nimber) {
                Some(move_nimber) => {
                    {
                        let mut guard = self.cache.get_mut(game)?;
                        guard.mark_impossible(move_nimber);
                    }
                    if nimber == move_nimber {
                        ruled_out_nimber = true;
                        break;
                    }
                }
                None => {
                    still_unprocessed_move_indices.push(parts);
                }
            }
        }

        {
            let mut guard = self.cache.get_mut(game)?;
            guard.append_unprocessed_moves(still_unprocessed_move_indices);
        }

        Some(ruled_out_nimber)
    }

    /// Computes the nimber of a sum of game parts under a bound.
    ///
    /// The result is computed as the XOR of the nimbers of each part,
    /// stopping early if it becomes clear the nimber would exceed the bound.
    fn get_bounded_nimber_by_parts(&self, parts: &Vec<G>, bound: usize) -> Option<usize> {
        if parts.len() == 0 {
            return Some(0);
        }
        let mut modifier = 0;
        for part in &parts[0..parts.len() - 1] {
            modifier ^= self.get_bounded_nimber_of_part(part, usize::MAX)?;
        }
        // The bound is adjusted with `| modifier` to ensure that the final XOR result
        // isn't incorrectly pruned: if any intermediate nimber exceeds the original bound,
        // but the XOR still stays within it, we don't want a false early exit.
        Some(modifier ^ self.get_bounded_nimber_of_part(parts.last()?, bound | modifier)?)
    }

    /// Initializes the move list for a game that is still a stub.
    ///
    /// For each move, the resulting game parts are reduced by canceling out
    /// symmetric pairs (since they XOR to 0).
    fn destub(&self, game: &G) {
        let is_stub = {
            let entry = self.cache.get_mut(game).unwrap();
            entry.is_stub()
        };
        if !is_stub {
            return;
        }

        let mut moves = game.get_moves();
        moves.sort_unstable_by_key(|m| {
            let mut hasher = DefaultHasher::new();
            hash(m, &mut hasher);
            hasher.finish()
        });
        moves.dedup();

        let move_indices: Vec<Vec<G>> = moves
            .into_iter()
            .map(|_move| match _move.get_parts() {
                Some(parts) => remove_pairs(parts),
                None => vec![_move.clone()],
            })
            .collect();

        {
            let mut entry = self.cache.get_mut(game).unwrap();
            entry.data = entry::EntryData::Processing {
                data: ProcessingData::new(move_indices),
            };
        }
    }
}

/// Removes consecutive pairs of equal elements in a sorted list.
/// Used to cancel out symmetric subgames when computing nimbers.
fn remove_pairs<G>(mut vec: Vec<G>) -> Vec<G>
where
    G: Impartial<G>,
{
    vec.sort_by_cached_key(|m| {
        let mut hasher = DefaultHasher::new();
        hash(m, &mut hasher);
        hasher.finish()
    });

    let mut read = 0;
    let mut write = 0;

    while read + 1 < vec.len() {
        if vec[read] == vec[read + 1] {
            read += 2;
        } else {
            vec.swap(read, write);
            read += 1;
            write += 1;
        }
    }
    if read < vec.len() {
        vec.swap(read, write);
        write += 1;
    }
    vec.truncate(write);
    vec
}
