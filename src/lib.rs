mod entry;
mod tests;
use entry::Entry;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::collections::HashMap;

/// providing the interface to evaluate an impartial game with the Evaluator
pub trait Impartial<G>: Sized + Clone + Hash + Eq
where
    G: Impartial<G>,
{
    fn get_parts(self) -> Vec<G>;
    fn get_max_nimber(&self) -> u16;
    fn get_possible_nimbers(&self) -> Vec<u16> {
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
    pub fn get_nimber(&mut self, g: G) -> u16 {
        return self.get_nimber_bounded(g, u16::max_value()).unwrap();
    }
    /// calculates the nimber of an impartial game but stoppes if the evaluator
    /// is certain that the nimber of the game is above the bound
    pub fn get_nimber_bounded(&mut self, g: G, bound: u16) -> Option<u16> {
        let parts_indices = self.get_part_indices(g);
        return self.get_nimber_by_part_indices(&parts_indices, bound);
    }
    /// gets bounded nimber given an index
    fn get_nimber_by_index(&mut self, index: usize, bound: u16) -> Option<u16> {
        //test if there is a child game for which child_nimber < nimber
        loop {
            let entry = &self.data[index];

            if let Some(nimber) = entry.get_nimber(){
                return Some(nimber);
            }

            let smallest_possible_nimber = entry.get_smallest_possible_nimber();

            if smallest_possible_nimber > bound {
                return None;
            }
            //try to rule out this smallest possible nimber
            if !self.try_to_rule_out_this_nimber(index, smallest_possible_nimber){
                self.data[index].set_nimber(smallest_possible_nimber);
                return self.data[index].get_nimber();
            }
        }
    }
    /// gets the nimber of a game where the parts are given by the given indices
    fn get_nimber_by_part_indices(&mut self, indices: &Vec<usize>, nimber: u16) -> Option<u16> {
        let mut modifier = 0;

        if indices.len() == 0 {
            return Some(0);
        }
        //accumulate all the nimbers of the first (n-1) parts
        for part_index in 0..(indices.len() - 1) {
            modifier ^= self
                .get_nimber_by_index(indices[part_index], u16::MAX)
                .unwrap();
        }

        //index of the last part of the current child game
        let last_part = indices.last().unwrap();
        //if the last part has the _nimber == nimber xor modifier
        match self.get_nimber_by_index(*last_part, nimber + modifier) {
            Some(last_nimber) => Some(last_nimber ^ modifier),
            None => None,
        }
    }
    ///checks if self.get(index) has a move with a given nimber
    /// there might be some performance improvement here todo!()
    fn try_to_rule_out_this_nimber(&mut self, index: usize, nimber: u16) -> bool{
        let move_indices = self.get_move_indices(index);

        if move_indices.len() == 0 {
            self.data[index].set_nimber(0);
            return nimber != 0;
        }

        for i in 0..move_indices.len() {
            match self.get_nimber_by_part_indices(&move_indices[i], nimber) {
                Some(move_nimber) => {
                    self.data[index].remove_nimber(move_nimber);
                    if move_nimber == nimber {
                        return true;
                    }
                }
                None => continue,
            }
        }
        return false;
    }
    /// generates a vec of all moves of the entry given by the index
    /// a move is represented as a vector of indices refering to the parts the position reached after the move
    /// for better performance all pairs of parts are removed 
    /// because they cancel each other out in the calculation of the nimber
    fn get_move_indices(&mut self, index: usize) -> Vec<Vec<usize>> {
        //if the moves are already generated stop generating
        match self.data[index].get_move_indices() {
            Some(move_indices) => move_indices.clone(),
            None => {
                let mut moves = self.data[index].get_unique_moves();

                //sort by the biggest possible nimber
                moves.sort_by(|a, b| a.get_max_nimber().cmp(&b.get_max_nimber()));

                let move_indices: Vec<Vec<usize>> = moves
                    .into_iter()
                    .map(|_move| self.get_part_indices(_move))
                    .map(|part_indices| remove_pairs(part_indices))
                    .collect();

                self.data[index].set_child_indices(move_indices.clone());
                move_indices
            }
        }
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

fn remove_pairs<T>(mut vec :Vec<T>) -> Vec<T>
where T: Eq + Ord
{
    vec.sort();
    let mut i = 0;
    while i + 1 < vec.len() {
        if vec[i] == vec[i+1] {
            vec.remove(i);
            vec.remove(i);
        }
        else{
            i += 1;
        }
    }
    return vec;
}