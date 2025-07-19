use sorted_vec::SortedSet;

use crate::Impartial;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct ProcessingData<G> {
    unprocessed_split_moves: Vec<Vec<G>>,
    impossible_nimbers: SortedSet<usize>,
}

impl<G> ProcessingData<G>
where
    G: Impartial<G>,
{
    pub fn new(moves: Vec<Vec<G>>) -> ProcessingData<G> {
        ProcessingData {
            unprocessed_split_moves: moves,
            impossible_nimbers: SortedSet::new(),
        }
    }
    pub fn get_smallest_possible_nimber(&self) -> usize {
        let len = self.impossible_nimbers.len();
        (0..len)
            .zip(&self.impossible_nimbers)
            .find(|(a, b)| a != *b)
            .map(|(a, _)| a)
            .unwrap_or(len)
    }
    pub fn mark_impossible(&mut self, nimber: usize) {
        self.impossible_nimbers.find_or_push(nimber);
    }
    pub fn pop_unprocessed_move(&mut self) -> Option<Vec<G>> {
        self.unprocessed_split_moves.pop()
    }
    pub fn append_unprocessed_moves(&mut self, other: Vec<Vec<G>>) {
        self.unprocessed_split_moves.extend(other);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) enum EntryData<G>
where
    G: Impartial<G>,
{
    Stub {},
    Processing { data: ProcessingData<G> },
    Done { nimber: usize },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct Entry<G>
where
    G: Impartial<G>,
{
    pub data: EntryData<G>,
    pub max_nimber: Option<usize>,
}
impl<G> Entry<G>
where
    G: Impartial<G>,
{
    pub fn new(max_nimber: Option<usize>) -> Entry<G> {
        Self {
            max_nimber,
            data: EntryData::Stub {},
        }
    }
    pub fn is_stub(&self) -> bool {
        match self.data {
            EntryData::Stub {} => true,
            _ => false,
        }
    }
    pub fn get_nimber(&self) -> Option<usize> {
        match &self.data {
            EntryData::Done { nimber } => Some(*nimber),
            _ => None
        }
    }

    pub fn get_smallest_possible_nimber(&self) -> Option<usize> {
        match &self.data {
            EntryData::Processing { data } => Some(data.get_smallest_possible_nimber()),
            _ => None,
        }
    }
    pub fn mark_impossible(&mut self, nimber: usize) {
        match &mut self.data {
            EntryData::Processing { data } => data.mark_impossible(nimber),
            _ => (),
        }
    }
    pub fn pop_unprocessed_move(&mut self) -> Option<Option<Vec<G>>> {
        match &mut self.data {
            EntryData::Processing { data } => Some(data.pop_unprocessed_move()),
            _ => None,
        }
    }
    pub fn append_unprocessed_moves(&mut self, other: Vec<Vec<G>>) {
        match &mut self.data {
            EntryData::Processing { data } => data.append_unprocessed_moves(other),
            _ => (),
        }
    }
}
