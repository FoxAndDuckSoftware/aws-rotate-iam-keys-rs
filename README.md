[![CI][ci_badge]][ci_link]
[![Built with cargo-make][cargo_make_badge]][cargo_make]
[![codecov][codecov_badge]][codecov]
[![FOSSA Status][fossa_badge]][fossa]

## Usage:
```
rotate-iam-keys [FLAGS] [OPTIONS] --profile <profile>...

FLAGS:
    -D, --disable
            disable the access key instead of deleting it

    -d, --dry-run
            runs without affecting anything, useful to run before fully committing to rotate your keys

    -h, --help
            Prints help information

    -V, --version
            Prints version information


OPTIONS:
        --configfile <configfile>
            location of your aws config file

        --credfile <credfile>
            location of your aws credential file

    -p, --profile <profile>...
            profile to rotate, you can specify multiple profiles, for example: `--profile=dev,prod` or `-p dev -p prod`
            to rotate all of those specified
```
## License
Licensed under either of:

* Apache License, Version 2.0, ([LICENSE-APACHE][license_apache])
* MIT license ([LICENSE-MIT][license_mit])

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as laid down in the Apache-2.0 license, will be dual licensed as above, without any additional terms or conditions.

All commits MUST be signed off using `-s` which certifies that you wrote or otherwise have the right to submit the code in accordance with [the Developer Certificate of Origin.][DCO]

## About
Copyright © 2020 Fox and Duck Software Ltd

Registered in England & Wales No. 9546077


[license_apache]: https://github.com/FoxAndDuckSoftware/aws-rotate-iam-keys-rs/blob/master/LICENSE-APACHE
[license_mit]: https://github.com/FoxAndDuckSoftware/aws-rotate-iam-keys-rs/blob/master/LICENSE-MIT

[ci_link]: https://github.com/FoxAndDuckSoftware/aws-rotate-iam-keys-rs/actions?workflow=CI
[ci_badge]: https://github.com/FoxAndDuckSoftware/aws-rotate-iam-keys-rs/workflows/CI/badge.svg

[cargo_make]: https://sagiegurari.github.io/cargo-make
[cargo_make_badge]: https://sagiegurari.github.io/cargo-make/assets/badges/cargo-make.svg

[codecov]: https://codecov.io/gh/FoxAndDuckSoftware/aws-rotate-iam-keys-rs
[codecov_badge]: https://codecov.io/gh/FoxAndDuckSoftware/aws-rotate-iam-keys-rs/branch/master/graph/badge.svg?token=ZLEAWJBDQ4

[fossa]: https://app.fossa.com/projects/git%2Bgithub.com%2FFoxAndDuckSoftware%2Faws-rotate-iam-keys-rs?ref=badge_small
[fossa_badge]: https://app.fossa.com/api/projects/git%2Bgithub.com%2FFoxAndDuckSoftware%2Faws-rotate-iam-keys-rs.svg?type=small

[DCO]: https://developercertificate.org/
