use clap::Command;

fn main() {
    let matches = Command::new("bonfida")
        .about("Development utils for Solana programs written in the Bonfida style")
        .subcommand(autobindings::command())
        .subcommand(autodoc::command())
        .subcommand(autoproject::command())
        .subcommand(benchviz::command())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .get_matches();
    match matches.subcommand().unwrap() {
        ("autobindings", m) => autobindings::process(m),
        ("autodoc", m) => autodoc::process(m),
        ("autoproject", m) => autoproject::process(m),
        ("benchviz", m) => benchviz::process(m),
        _ => unreachable!(),
    }
}
