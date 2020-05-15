use std::env;
use std::fs::{self};
use std::process;

use clap::Shell;

include!("src/app.rs");

fn main() {
    // OUT_DIR is set by Cargo and it's where any additional build artifacts
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
    app.gen_completions("rotate-iam-keys", Shell::Bash, &outdir);
    app.gen_completions("rotate-iam-keys", Shell::Zsh, &outdir);
    app.gen_completions("rotate-iam-keys", Shell::Fish, &outdir);
    app.gen_completions("rotate-iam-keys", Shell::PowerShell, &outdir);
}
