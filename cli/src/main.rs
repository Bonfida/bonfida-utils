use clap::Command;

fn main() {
    let matches = Command::new("bonfida")
        .about("Development utils for Solana programs written in the Bonfida style")
        .subcommand(bonfida_autobindings::command())
        .subcommand(bonfida_autodoc::command())
        .subcommand(bonfida_autoproject::command())
        .subcommand(bonfida_benchviz::command())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .get_matches();
    match matches.subcommand().unwrap() {
        ("autobindings", m) => bonfida_autobindings::process(m),
        ("autodoc", m) => bonfida_autodoc::process(m),
        ("autoproject", m) => bonfida_autoproject::process(m),
        ("benchviz", m) => bonfida_benchviz::process(m),
        _ => unreachable!(),
    }
}
