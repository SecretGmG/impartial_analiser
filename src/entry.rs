use crate::Impartial;
use serde::{Serialize, Deserialize};


#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
enum EntryState{
    Done {nimber : usize},
    Processing {unprocessed_move_indices : Option<Vec<Vec<usize>>>, max_possible_nimber : Option<usize>, impossible_nimbers : Vec<usize>}
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub(super) struct Entry<G>
where
    G: Impartial<G>,
{
    state : EntryState,
    game: G,
}

impl<G> Entry<G>
where
    G: Impartial<G>
{
    pub fn new(game: G) -> Entry<G> {
        Entry {
            state : EntryState::Processing { unprocessed_move_indices: None, max_possible_nimber: game.get_max_nimber(), impossible_nimbers: game.get_impossible_nimbers() },
            game: game,
        }
    }
    pub fn get_nimber(&self) -> Option<usize>{
        match self.state {
            EntryState::Done { nimber } => Some(nimber),
            _ => None
        }
    }
    pub fn remove_nimber(&mut self, nimber: usize){
        match &mut self.state {
            EntryState::Processing { unprocessed_move_indices, max_possible_nimber , impossible_nimbers} =>
                if !impossible_nimbers.contains(&nimber) {impossible_nimbers.concat(nimber);},
            _ => panic!("cannot remove nimber from Entry that is Done")
        }
    }
    pub fn set_nimber(&mut self, nimber: usize){
        self.possible_nimbers = vec![nimber];
    }
    pub fn get_smallest_possible_nimber(&self) -> usize{
        self.possible_nimbers[0]
    }
    pub fn get_next_unprocessed_move_index(&mut self) -> Option<Vec<usize>> {
        self.unprocessed_move_indices.as_mut().expect("the move indices should already be generated").pop()
    }
    pub fn add_unprocessed_move_indices(&mut self, mut new_unprocessed_move_indices: Vec<Vec<usize>>){
        self.unprocessed_move_indices.as_mut().expect("the move indices should already be generated").append(&mut new_unprocessed_move_indices);
    }
    pub fn are_move_indices_generated(&self) -> bool{
        return self.unprocessed_move_indices.is_some();
    }
    pub fn set_child_indices(&mut self, unprocessed_move_indices: Vec<Vec<usize>>) {
        self.unprocessed_move_indices = Some(unprocessed_move_indices);
    }
    pub fn get_unique_moves(&self) -> Vec<G> {
        self.game.get_unique_moves()
    }
}
