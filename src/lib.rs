mod entry;
mod tests;
use entry::Entry;
use std::{collections::HashMap, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use std::hash::Hash;

use crate::entry::{EntryData, ProcessingData};

/// providing the interface to evaluate an impartial game with the Evaluator
pub trait Impartial<G>: Sized + Clone + Hash + Eq
where
    G: Impartial<Self>,
{
    fn get_parts(&self) -> Option<Vec<G>>;
    fn get_max_nimber(&self) -> Option<usize> {
        None
    }
    fn get_unique_moves(&self) -> Vec<G>;
}

/// Evaluates an impartial game
/// The generic arguments specify
/// a generalized version and a smaller part of a generalized impartial game
#[derive(Debug, Clone)]
pub struct Evaluator<G>
where
    G: Impartial<G>,
{
    data: Vec<Entry<G>>,
    index_map: HashMap<G, usize>,
    cancel_flag: Arc<AtomicBool>,
}

impl<G> Evaluator<G>
where
    G: Impartial<G>,
{
    pub fn new() -> Evaluator<G> {
        Evaluator {
            data: vec![],
            index_map: HashMap::new(),
            cancel_flag: Arc::new(AtomicBool::new(false))
        }
    }
    /// calculates the nimber of an impartial game
    pub fn get_nimber(&mut self, g: G) -> Option<usize> {
        self.get_bounded_nimber(g, usize::max_value())
    }
    /// calculates the nimber of an impartial game but stoppes if the evaluator
    /// is certain that the nimber of the game is above the bound
    pub fn get_bounded_nimber(&mut self, g: G, bound: usize) -> Option<usize> {
        let parts_indices = self.get_part_indices(g);
        self.get_bounded_nimber_by_parts(&parts_indices, bound)
    }
    /// gets bounded nimber given an index
    fn get_bounded_nimber_by_index(&mut self, index: usize, bound: usize) -> Option<usize> {
        if let EntryData::Done { nimber } =  self.data[index].data {
            return Some(nimber);
        }
        loop {
            if self.cancel_flag.load(Ordering::Relaxed){
                return None;
            }
            let data = self.get_processing_data_mut(index).unwrap();
            let nimber = data.get_smallest_possible_nimber();
            if nimber > bound {
                return None;
            }
            if !self.try_rule_out_nimber(index, nimber)? {
                self.data[index].data = EntryData::Done { nimber };
                return Some(nimber);
            }
        }
    }
    fn try_rule_out_nimber(&mut self, index: usize, nimber : usize) -> Option<bool> {
        if let Some(max_nimber) = self.data[index].game.get_max_nimber(){
            if max_nimber < nimber {
                return Some(false);
            }
        }
        let mut still_unprocessed_move_indices = vec![];
        let mut ruled_out_nimber = false; 
        while let Some(move_indices) = self.get_processing_data_mut(index).unwrap().pop_unprocessed_move() {
            if self.cancel_flag.load(Ordering::Relaxed){
                return None;
            }
            match self.get_bounded_nimber_by_parts(&move_indices, nimber) {
                Some(move_nimber) => {
                    self.get_processing_data_mut(index).unwrap().remove_nimber(move_nimber);
                    if nimber == move_nimber {
                        ruled_out_nimber = true;
                        break;
                    }
                },
                None => {
                    still_unprocessed_move_indices.push(move_indices);
                },
            }
        }
        self.get_processing_data_mut(index).unwrap().append_unprocessed_moves(still_unprocessed_move_indices);
        return Some(ruled_out_nimber);
    }
    /// gets the nimber of a game where the parts are given by the given indices
    fn get_bounded_nimber_by_parts(&mut self, indices: &Vec<usize>, bound: usize) -> Option<usize> {
        if indices.len() == 0 {
            return Some(0);
        }
        let mut modifier = 0;
        for index in &indices[0..indices.len() - 1]{
            modifier ^= self.get_bounded_nimber_by_index(*index, usize::MAX)?;
        }
        // if the last part has the _nimber == nimber xor modifier
        // the biggest possible nimber of the last part needed to check to make sure
        // that the total nimber = last nimber ^ modifier is less than the bound
        // is of the size bound | modifier
        Some(
            modifier ^ 
            self.get_bounded_nimber_by_index(
                *indices.last()?, 
                bound | modifier
            )?
        )
    }
    /// generates a vec of all moves of the entry given by the index
    /// a move is represented as a vector of indices refering to the parts the position reached after the move
    /// for better performance all pairs of parts are removed
    /// because they cancel each other out in the calculation of the nimber
    fn generate_move_indices(&mut self, index: usize) {
        //if the moves are already generated stop generating
        let entry = &mut self.data[index];
        if let EntryData::Stub {  } = entry.data {
            let mut moves = entry.game.get_unique_moves();
            //sort by the biggest possible nimber
            moves.sort_by(|a, b| a.get_max_nimber().cmp(&b.get_max_nimber()));
            let move_indices: Vec<Vec<usize>> = moves
                .into_iter()
                .map(|_move| self.get_part_indices(_move))
                .map(|part_indices| remove_pairs(part_indices))
                .collect();
            // reborrow entry beacuse self can be modified when getting move_indices
            let entry = &mut self.data[index];
            entry.data = entry::EntryData::Processing {
                data: ProcessingData::new(&entry.game, move_indices)
            };
        }
    }
    fn get_processing_data_mut(&mut self, index: usize) -> Option<&mut ProcessingData> {
        if let EntryData::Stub {  } = &self.data[index].data {
            self.generate_move_indices(index);
        }
        if let EntryData::Processing { data } = &mut self.data[index].data {
            Some(data)
        }
        else{
            None
        }
    }
    /// returns indices of g
    pub fn get_part_indices(&mut self, g: G) -> Vec<usize> {
        match g.get_parts() {
            Some(parts) => parts.iter().map(|part| self.get_index_of(part)).collect(),
            None => vec![self.get_index_of(&g)],
        }
    }
    pub fn get_index_of(&mut self, g: &G) -> usize {
        if let Some(index) = self.index_map.get(g) {
            *index
        } else {
            self.add_game(g.clone())
        }
    }
    pub fn add_game(&mut self, game: G) -> usize {
        let entry = Entry::new(game.clone());
        let index = self.data.len();
        self.index_map.insert(game, index);
        self.data.push(entry);
        index
    }
}

fn remove_pairs<T>(mut vec: Vec<T>) -> Vec<T>
where
    T: Eq + Ord,
{
    vec.sort();
    let mut result = Vec::with_capacity(vec.len());
    let mut iter = vec.into_iter().peekable();

    while let Some(x) = iter.next() {
        if iter.peek() == Some(&x) {
            iter.next(); // skip the duplicate
        } else {
            result.push(x);
        }
    }
    result
}
