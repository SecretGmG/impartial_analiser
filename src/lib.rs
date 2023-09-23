mod entry;
mod tests;
use entry::Entry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

/// providing the interface to evaluate an impartial game with the Evaluator
pub trait Impartial<G>: Sized + Clone + Hash + Eq
where
    G: Impartial<G>,
{
    fn get_parts(self) -> Vec<G>;
    fn get_max_nimber(&self) -> usize;
    fn get_possible_nimbers(&self) -> Vec<usize> {
        (0..=self.get_max_nimber()).collect()
    }
    fn get_unique_moves(&self) -> Vec<G>;
}

/// Evaluates an impartial game
/// The generic arguments specify
/// a generalized version and a smaller part of a generalized impartial game
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluator<G>
where
    G: Impartial<G>,
{
    data: Vec<Entry<G>>,
    index_map: HashMap<G, usize>,
}

impl<G> Evaluator<G>
where
    G: Impartial<G>,
{
    pub fn new() -> Evaluator<G> {
        Evaluator {
            data: vec![],
            index_map: HashMap::new(),
        }
    }
    /// calculates the nimber of an impartial game
    pub fn get_nimber(&mut self, g: G) -> usize {
        return self.get_bounded_nimber(g, usize::max_value()).unwrap();
    }
    /// calculates the nimber of an impartial game but stoppes if the evaluator
    /// is certain that the nimber of the game is above the bound
    pub fn get_bounded_nimber(&mut self, g: G, bound: usize) -> Option<usize> {
        let parts_indices = self.get_part_indices(g);
        return self.get_bounded_nimber_by_parts(&parts_indices, bound);
    }
    fn try_rule_out_smallest_possible_nimber(&mut self, index: usize) {
        self.get_move_indices(index);
        let nimber = self.data[index].get_smallest_possible_nimber();

        let mut still_unprocessed_move_indices = vec![];
        
        while let Some(move_indices) = self.data[index].get_next_unprocessed_move_index() {
            match self.get_bounded_nimber_by_parts(&move_indices, nimber) {
                Some(move_nimber) => {
                    self.data[index].remove_nimber(move_nimber);
                    if move_nimber == nimber {
                        self.data[index].add_unprocessed_move_indices(still_unprocessed_move_indices);
                        return;
                    }
                }
                //since the move was not fully prcessed we need to add it back to the unprocessed moves later
                None => {
                    still_unprocessed_move_indices.push(move_indices);
                },
            }
        }
        self.data[index].set_nimber(nimber);
    }
    /// gets bounded nimber given an index
    fn get_bounded_nimber_by_index(&mut self, index: usize, bound: usize) -> Option<usize> {
        loop {
            let entry = &self.data[index];

            if let Some(nimber) = entry.get_nimber() {
                return Some(nimber);
            }
            if entry.get_smallest_possible_nimber() > bound {
                return None;
            }
            self.try_rule_out_smallest_possible_nimber(index);
        }
    }
    /// gets the nimber of a game where the parts are given by the given indices
    fn get_bounded_nimber_by_parts(&mut self, indices: &Vec<usize>, bound: usize) -> Option<usize> {
        if indices.len() == 0 {
            return Some(0);
        }
        let modifier = indices[0..indices.len() - 1]
            .iter()
            .fold(0, |modifier, index| {
                modifier ^ self.get_bounded_nimber_by_index(*index, usize::MAX).unwrap()
            });
        //index of the last part of the current child game
        let last_part = indices.last().unwrap();
        //if the last part has the _nimber == nimber xor modifier
        match self.get_bounded_nimber_by_index(*last_part, bound + modifier) {
            Some(last_nimber) => Some(last_nimber ^ modifier),
            None => None,
        }
    }
    /// generates a vec of all moves of the entry given by the index
    /// a move is represented as a vector of indices refering to the parts the position reached after the move
    /// for better performance all pairs of parts are removed
    /// because they cancel each other out in the calculation of the nimber
    fn get_move_indices(&mut self, index: usize) {
        //if the moves are already generated stop generating
        if self.data[index].are_move_indices_generated() {
            return;
        }
        let mut moves = self.data[index].get_unique_moves();

        //sort by the biggest possible nimber
        moves.sort_by(|a, b| a.get_max_nimber().cmp(&b.get_max_nimber()));

        let move_indices: Vec<Vec<usize>> = moves
            .into_iter()
            .map(|_move| self.get_part_indices(_move))
            .map(|part_indices| remove_pairs(part_indices))
            .collect();

        self.data[index].set_child_indices(move_indices);
    }
    pub fn get_part_indices(&mut self, g: G) -> Vec<usize> {
        g.get_parts()
            .iter()
            .map(|part| self.get_index_of(part))
            .collect()
    }
    pub fn get_index_of(&mut self, g: &G) -> usize {
        if let Some(index) = self.index_map.get(g) {
            return *index;
        } else {
            return self.add_game(g.clone());
        }
    }
    pub fn add_game(&mut self, game: G) -> usize {
        let entry = Entry::new(game.clone());
        let index = self.data.len();
        self.index_map.insert(game, index);
        self.data.push(entry);
        return index;
    }
}

fn remove_pairs<T>(mut vec: Vec<T>) -> Vec<T>
where
    T: Eq + Ord,
{
    vec.sort();
    let mut i = 0;
    while i + 1 < vec.len() {
        if vec[i] == vec[i + 1] {
            vec.remove(i);
            vec.remove(i);
        } else {
            i += 1;
        }
    }
    return vec;
}
