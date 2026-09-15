# COCOX

A Conventional Commitlint binary tool: checks whether a commit message follows the Conventional Commits format.

[Work in Progress] Drop-in replacement of [opensource-nepal/commitlint](https://github.com/opensource-nepal/commitlint)

## Development

Run cocox
```shell
cargo run -- "my commit message"

cargo run -- --file "/foo/bar"


# using commit hash
cargo run -- --hash xxxx-xxxx

# using  range of hashes: 
cargo run -- --from-hash xxxx-xxxx --to-hash xxxx-xxxx

# --to-hash points to HEAD by default
cargo run -- --from-hash xxxx-xxxx 

```

## Usage: 
```
A Conventional Commitlint binary tool: checks whether a commit message follows the Conventional Commits format.

Usage: cocox [OPTIONS] <MESSAGE|--file <FILE>|--hash <HASH>|--from-hash <FROM_HASH>>

Arguments:
  [MESSAGE]  The commit message to be checked

Options:
      --file <FILE>
          Lint the message in a file, for example .git/COMMIT_EDITMSG. Git comment lines are removed
      --hash <HASH>
          Lint the message of one commit
      --from-hash <FROM_HASH>
          Inclusive lower bound of the commit range to lint
      --to-hash <TO_HASH>
          Inclusive upper bound of the commit range to lint. Requires --from-hash. [default: HEAD]
      --skip-detail
          Skip the detailed error message check
      --hide-input
          Hide input from stdout
  -q, --quiet
          Ignore stdout and stderr
  -v, --verbose
          Verbose output
      --max-header-length <MAX_HEADER_LENGTH>
          Maximum header length to check. If not specified, the header length is not checked.
  -h, --help
          Print help
  -V, --version
          Print version
```
