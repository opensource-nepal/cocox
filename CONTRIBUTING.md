# Contributing Guide

Thank you for your interest in contributing to the **cocox** project!
Your contributions will help improve and enhance this tool.
Please take a moment to review the following guidelines before getting started.

cocox is a Rust port of [commitlint](https://github.com/opensource-nepal/commitlint).
The Python tool is the reference implementation. When the two disagree about the result
for the same commit message, cocox is the one that is wrong, unless a maintainer decides
otherwise. Keeping that in mind will save you time.

This guide follows the structure of the
[commitlint contributing guide](https://github.com/opensource-nepal/commitlint/blob/main/CONTRIBUTING.md),
with the Rust toolchain in place of the Python one.

## Prerequisites

Before contributing, ensure that you have the following:

- **Rust 1.85 or newer**. The project uses edition 2024, which older toolchains
  cannot build. Install it with [rustup](https://rustup.rs/), then run `rustup update stable`.
- **Git**, because several tests create real repositories and shell out to `git`.
- Optionally, **Python 3.10 or newer** and `pip`, if you want to compare against the
  reference implementation. See [Checking parity](#checking-parity).

## Getting Started

To set up the project on your local machine, follow these steps:

1. **Fork** the repository on GitHub.
2. **Clone** the forked repository to your local machine:

   ```bash
   git clone https://github.com/<your-username>/cocox.git
   cd cocox
   ```

3. **Build** the project:

   ```bash
   cargo build
   ```

4. **Verify your setup**:

   ```bash
   cargo run -- "feat: verify my setup"
   ```

   You should see `Commit validation: successful!`.

## Tests

Run the whole suite:

```bash
cargo test
```

Run one part of it:

```bash
cargo test --lib                  # unit tests inside src/
cargo test --test cli             # end-to-end tests that run the binary
cargo test --test git_helpers     # tests that create temporary git repositories
```

Some tests change the working directory of the test process, so they are marked
`#[serial]`. Keep that attribute on any test that uses the `TestRepo` helper, or unrelated
tests will fail in confusing ways.

## Code quality

Run all of these before you open a pull request:

```bash
cargo build
cargo test
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
```

`cargo fmt --all` fixes formatting for you. The project uses default rustfmt settings, so
there is nothing to configure and nothing to argue about.

## Checking parity

cocox has to behave like the Python tool. The most reliable way to settle a question about
correct behavior is to run both and compare, rather than reading the Rust and reasoning
about it.

```bash
pip install commitlint==2.0.0
cargo build

./target/debug/cocox "feat:" ; echo "exit=$?"
commitlint "feat:"           ; echo "exit=$?"
```

Compare three things: the exit code, standard output, and standard error. The upstream
project also keeps a table of test cases at `tests/fixtures/linter.py` in its repository,
which is a good source of examples when you add a rule.

Python and Rust differ in ways that are easy to miss when porting. Character counts,
regular expression anchoring, newline handling and text trimming all have traps. If you
are changing the linter, read `AGENTS.md`, which collects the ones that have already
caused bugs here.

## Pull Requests

We welcome and appreciate pull requests from the community. To contribute:

1. **Fork** the repository and create a new branch based on the `main` branch:

   ```bash
   git checkout -b <your-branch-name>
   ```

2. **Write tests** for your changes. If you are fixing a bug, add the input that was
   broken as a test case first, so the diff shows the behavior change.
3. **Keep one concern per pull request.** We squash on merge and the changelog is built
   from commit messages, so a pull request that does two unrelated things becomes one
   confusing changelog entry.
4. **Follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)** for
   commit messages. This project lints commit messages, so its own history should pass its
   own lint. Examples:

   - `feat: add commit message validation`
   - `fix(parser): resolve message parsing issue`

   You can check your message with the tool itself:

   ```bash
   cargo run -- "feat: add commit message validation"
   ```

5. **Push** your branch to your forked repository:

   ```bash
   git push origin <your-branch-name>
   ```

6. **Create a Pull Request**:

   - Open a pull request from your branch to the `main` branch of the original repository.
   - Provide a clear and concise description of the changes, along with relevant context.
   - If the change affects the command line interface, update `README.md` in the same
     pull request.

7. **Review & Feedback**:

   - Participate in the code review process and address any feedback promptly.

## Release

The release process, changelog, and versioning are managed by
[release-please](https://github.com/googleapis/release-please). Versions are automatically
determined based on Conventional Commit types, and the changelog is generated from commit
messages. A new release is published only after the release pull request is merged.

Binaries are built and attached to the release by
[cargo-dist](https://github.com/axodotdev/cargo-dist), configured in `dist-workspace.toml`.

## License

By contributing to this project, you agree that your contributions will be licensed under
the **GPL-3.0 License**. Refer to the [LICENSE](./LICENSE) file for more details.

## Other Ways to Contribute

Even if you don't contribute code, you can still help:

- **Report a difference** between cocox and commitlint. A commit message that the two
  treat differently is a useful bug report on its own.
- **Spread the word** about this tool.
- Write a blog or article about how you use this project.
- Share your best practices, examples, or ideas with us.

Thank you for contributing to **cocox**! 🎉
