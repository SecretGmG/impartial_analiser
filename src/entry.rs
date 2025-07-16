use crate::Impartial;
use sorted_vec::SortedSet;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct ProcessingData{
    unprocessed_move_indices : Vec<Vec<usize>>,
    max_possible_nimber : Option<usize>,
    impossible_nimbers : SortedSet<usize>,
}

impl ProcessingData {
    pub fn new<G : Impartial<G>>(game: &G, move_indices : Vec<Vec<usize>>) -> ProcessingData
    {
        ProcessingData{
            unprocessed_move_indices: move_indices,
            max_possible_nimber: game.get_max_nimber(),
            impossible_nimbers: SortedSet::new()
        }
    }
    pub fn get_smallest_possible_nimber(&self) -> usize{    
            let len = self.impossible_nimbers.len();
            (0..len)
                .zip(&self.impossible_nimbers)
                .find(|(a,b)| a!=*b)
                .map(|(a,_)| a)
                .unwrap_or(len)
        }
    pub fn remove_nimber(&mut self, nimber: usize){
            self.impossible_nimbers.find_or_push(nimber);
        }
    pub fn pop_unprocessed_move(&mut self) -> Option<Vec<usize>>{
        self.unprocessed_move_indices.pop()
    }
    pub fn append_unprocessed_moves(&mut self, other : Vec<Vec<usize>>) {
        self.unprocessed_move_indices.extend(other);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) enum EntryData{
    Stub {},
    Processing {data : ProcessingData},
    Done { nimber : usize}
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct Entry<G>
where
    G: Impartial<G>,
{
    pub game : G,
    pub data : EntryData,
}
impl<G> Entry<G>
where
    G: Impartial<G>
{
    pub fn new(game: G) -> Entry<G> {
        Self { game: game, data: EntryData::Stub {  } }
    }
    pub fn is_stub(&self) -> bool {
        match self.data {
            EntryData::Stub {  } => true,
            _ => false
        }
    }
}