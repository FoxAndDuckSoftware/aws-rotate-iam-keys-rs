use clap::{App, AppSettings, Arg};

#[must_use]
pub fn app() -> App<'static, 'static> {
    App::new("aws-rotate-iam-keys")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version("1.0.0")
        .author("Martin Kemp <martin@foxandducksoftware.co.uk>")
        .about("Rotates your IAM Access Keys\n\nhttps://github.com/FoxAndDuckSoftware/aws-rotate-iam-keys-rs")
        .arg(
            Arg::with_name("profile")
                .short("p")
                .long("profile")
                .takes_value(true)
                .help("profile(s) to rotate")
                .long_help("profile to rotate, you can specify multiple profiles, for example: `--profile=dev,prod` or `-p dev -p prod` to rotate all of those specified")
                .multiple(true)
                .use_delimiter(true)
                .required(true)
        )
        .arg(
            Arg::with_name("credfile")
                .long("credfile")
                .takes_value(true)
                .help("location of your aws credential file")
                .number_of_values(1)
                .multiple(false)
        )
        .arg(
            Arg::with_name("configfile")
                .long("configfile")
                .takes_value(true)
                .help("location of your aws config file")
                .number_of_values(1)
                .multiple(false)
        )
        .arg(
            Arg::with_name("disable")
                .short("D")
                .long("disable")
                .takes_value(false)
                .help("disable the access key instead of deleting it")
                .multiple(false)
        )
        .arg(
            Arg::with_name("dry-run")
                .short("d")
                .long("dry-run")
                .help("runs without affecting anything, useful to run before fully committing to rotate your keys")
                .multiple(false)
        )
}
