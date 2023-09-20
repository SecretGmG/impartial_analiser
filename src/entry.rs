use crate::Impartial;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub(super) struct Entry<G>
where
    G: Impartial<G>,
{
    game: G,
    possible_nimbers: Vec<u16>,
    move_indices: Option<Vec<Vec<usize>>>,
}

impl<G> Entry<G>
where
    G: Impartial<G>
{
    pub fn new(game: G) -> Entry<G> {
        Entry {
            possible_nimbers : game.get_possible_nimbers(),
            game: game,
            move_indices: None,
        }
    }
    pub fn get_nimber(&self) -> Option<u16>{
        if self.possible_nimbers.len() == 1{
            Some(self.possible_nimbers[0])
        }
        else{
            None
        }
    }
    pub fn remove_nimber(&mut self, nimber: u16){
        match self.possible_nimbers.binary_search(&nimber) {
            Ok(i) => _ = self.possible_nimbers.remove(i),
            Err(_) => (),
        }
    }
    pub fn set_nimber(&mut self, nimber: u16){
        self.possible_nimbers = vec![nimber];
    }
    pub fn get_smallest_possible_nimber(&self) -> u16{
        self.possible_nimbers[0]
    }
    pub fn get_move_indices(&self) -> Option<&Vec<Vec<usize>>> {
        self.move_indices.as_ref()
    }
    pub fn set_child_indices(&mut self, child_indices: Vec<Vec<usize>>) {
        self.move_indices = Some(child_indices);
    }
    pub fn get_unique_moves(&self) -> Vec<G> {
        self.game.get_unique_moves()
    }
}
