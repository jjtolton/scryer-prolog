use std::collections::BTreeSet;
use std::env;

#[derive(Debug)]
pub struct MachineArgs {
    pub add_history: bool,
    pub halt_on_error: bool,
}

impl MachineArgs {
    pub fn new() -> Self {
        let args: BTreeSet<String> = env::args().collect();
        Self {
            add_history: !args.contains("--no-add-history"),
            halt_on_error: args.contains("--halt-on-error"),
        }
    }
}

impl Default for MachineArgs {
    fn default() -> Self {
        Self::new()
    }
}
