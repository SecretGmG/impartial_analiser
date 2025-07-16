mod entry;
mod tests;
use entry::Entry;
use std::{collections::HashMap, hash::DefaultHasher, ptr::hash, sync::{atomic::{AtomicBool, Ordering}, Arc}};
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
    fn get_moves(&self) -> Vec<G>;
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
    pub fn get_nimbers(&self) -> Vec<(G, usize)>{
        self.data.iter().filter_map(|e| {
            let nimber = match  e.data {
                EntryData::Done { nimber } => Some(nimber),
                _ => None,
            }?;
            Some((e.game.clone(), nimber))
        }).collect()
    }
    pub fn get_cache_size(&self) -> usize{
        self.data.len()
    }
    pub fn get_cancel_flag(&self) -> Arc<AtomicBool>{
        self.cancel_flag.clone()
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
            let data = self.get_processing_data_mut(index);
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
        while let Some(move_indices) = self.get_processing_data_mut(index).pop_unprocessed_move() {
            if self.cancel_flag.load(Ordering::Relaxed){
                return None;
            }
            match self.get_bounded_nimber_by_parts(&move_indices, nimber) {
                Some(move_nimber) => {
                    self.get_processing_data_mut(index).remove_nimber(move_nimber);
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
        self.get_processing_data_mut(index).append_unprocessed_moves(still_unprocessed_move_indices);
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
        if entry.is_stub() {
            let mut moves = entry.game.get_moves();
            let mut hasher = DefaultHasher::new();
            moves.sort_by_key(|m| hash(m, &mut hasher));
            moves.dedup();
            moves.sort_by_cached_key(|m| m.get_max_nimber());
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
    fn get_processing_data_mut(&mut self, index: usize) -> &mut ProcessingData {
        if self.data[index].is_stub() {
            self.generate_move_indices(index);
        }
        if let EntryData::Processing { data } = &mut self.data[index].data {
            return data;
        }
        panic!("get_processing_data_mut can only be called in branches where the entry is not already done")
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
    T: Eq + Ord + Copy,
{
    vec.sort();

    let mut read = 0;
    let mut write = 0;

    while read+1 < vec.len(){
        if vec[read] == vec[read+1]{
            read += 2;
        }
        else{
            vec[write] = vec[read];
            read += 1;
            write += 1;
        }
    }
    if read < vec.len() {
        vec[write] = vec[read];
        write += 1;
    }
    vec.truncate(write);
    vec
}
