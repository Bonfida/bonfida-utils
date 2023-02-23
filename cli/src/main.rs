use clap::Command;

fn main() {
    let matches = Command::new("bonfida")
        .about("Development utils for Solana programs written in the Bonfida style")
        .subcommand(bonfida_autobindings::command().name("autobindings"))
        .subcommand(bonfida_autodoc::command().name("autodoc"))
        .subcommand(bonfida_autoproject::command().name("autoproject"))
        .subcommand(bonfida_benchviz::command().name("benchviz"))
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
