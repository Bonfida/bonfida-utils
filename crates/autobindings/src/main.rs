use bonfida_autobindings::{command, process};

pub fn main() {
    let command = command();
    let matches = command.get_matches();
    process(&matches);
}
