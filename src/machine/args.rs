use std::collections::BTreeSet;
use std::env;

#[derive(Debug)]
pub struct MachineArgs {
    pub add_history: bool,
    pub always_halt: bool,
}

impl MachineArgs {
    pub fn new() -> Self {
        let args: BTreeSet<String> = env::args().collect();
        Self {
            add_history: !args.contains("--no-add-history"),
            always_halt: args.contains("--always-halt"),
        }
    }
}

impl Default for MachineArgs {
    fn default() -> Self {
        Self::new()
    }
}
