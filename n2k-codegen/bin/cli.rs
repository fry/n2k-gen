use n2k_codegen::N2kCodeGenOpts;
use std::{collections::HashSet, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opts {
    #[structopt(long)]
    pub pgns_xml: String,
    #[structopt(short = "p", long = "pgn")]
    pub pgns: Vec<u32>,
    #[structopt(short, long)]
    pub output: PathBuf,
    #[structopt(short, long)]
    pub crate_name: String,
}

pub fn main() {
    env_logger::init();
    let opts = Opts::from_args();

    let args = N2kCodeGenOpts {
        pgns_xml: opts.pgns_xml,
        pgns: opts.pgns.iter().cloned().collect(),
        output: opts.output,
        generate_crate: Some(opts.crate_name),
    };

    n2k_codegen::codegen(args);
}
