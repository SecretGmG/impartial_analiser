use crate::Impartial;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub(super) struct Entry<G>
where
    G: Impartial<G>,
{
    game: G,
    possible_nimbers: Vec<usize>,
    unprocessed_move_indices: Option<Vec<Vec<usize>>>,
}

impl<G> Entry<G>
where
    G: Impartial<G>
{
    pub fn new(game: G) -> Entry<G> {
        Entry {
            possible_nimbers : game.get_possible_nimbers(),
            game: game,
            unprocessed_move_indices: None,
        }
    }
    pub fn get_nimber(&self) -> Option<usize>{
        if self.possible_nimbers.len() == 1{
            Some(self.possible_nimbers[0])
        }
        else{
            None
        }
    }
    pub fn remove_nimber(&mut self, nimber: usize){
        match self.possible_nimbers.binary_search(&nimber) {
            Ok(i) => _ = self.possible_nimbers.remove(i),
            Err(_) => (),
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
