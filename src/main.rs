mod context;
mod log;
mod messages;
mod raft_state;
mod utils;

use crate::utils::*;

fn main() {
    println!("Hello, world!");
}

// trait StateMachine {
//     type State;
//     type Message;
//     fn apply(&mut self, state: &mut Self::State, message: Self::Message);
// }
