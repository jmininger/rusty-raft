use crate::utils::*;

#[derive(Debug, Clone)]
pub struct Log(Vec<LogEntry>);

impl Log {
    // Consider making htis a HashMap keyed on LogIndex
    // Alternatively create a VecWithOffset type that allows me to index a compacted log with its
    // actual index
    pub fn new() -> Self {
        Log(Vec::new())
    }

    fn add_entry(&mut self, entry: LogEntry) {
        // Maybe add verification here?
        self.0.push(entry);
    }

    pub fn invalid_term_bounds(&self, index: LogIndex, term: TermId) -> bool {
        let is_valid = self
            .0
            .get(index)
            .map(|entry| entry.term == term)
            // If the entry doesnt exist at all then we also return false
            .unwrap_or(false);
        !is_valid
    }

    pub fn append_entries(&mut self, entries: Vec<LogEntry>) -> Result<(), ()> {
        // If an existing entry conflicts with a new one (same index
        // but different terms), delete the existing entry and all that
        // Append all new entries not already in the log
        todo!()
    }

    pub fn get_last_index(&self) -> LogIndex {
        self.0.last().map(|entry| entry.index).unwrap_or(0)
    }
}
