use crate::utils::*;

pub struct Log(Vec<LogEntry>);

impl Log {
    pub fn new() -> Self {
        Log(Vec::new())
    }

    pub fn retrieve_term(&self, index: LogIndex) -> TermId {
        todo!()
    }

    pub fn add_entry(&mut self, entry: LogEntry) {
        self.0.push(entry);
    }
}
