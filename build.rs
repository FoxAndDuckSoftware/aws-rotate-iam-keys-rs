use std::env;
use std::fs::{self};
use std::process;

use clap::Shell;

include!("src/app.rs");

fn main() {
    // Cargo sets OUT_DIR, and it is where any additional build artifacts
    // are written.
    let outdir = match env::var_os("OUT_DIR") {
        Some(outdir) => outdir,
        None => {
            eprintln!(
                "OUT_DIR environment variable not defined. \
                 Please file a bug: \
                 https://github.com/FoxAndDuckSoftware/aws-rotate-iam-keys-rs/issues/new"
            );
            process::exit(1);
        }
    };
    fs::create_dir_all(&outdir).unwrap();

    // Use clap to build completion files.
    let mut app = app();
    const BIN_NAME: &str = "rotate-iam-keys";
    app.gen_completions(BIN_NAME, Shell::Bash, &outdir);
    app.gen_completions(BIN_NAME, Shell::Zsh, &outdir);
    app.gen_completions(BIN_NAME, Shell::Fish, &outdir);
    app.gen_completions(BIN_NAME, Shell::PowerShell, &outdir);
}
