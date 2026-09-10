# AGENTS.md

Instructions for AI coding agents working on cocox. This is the single source of truth for
agent guidance. Tool-specific files, such as `CLAUDE.md`, only point here.

Read [CONTRIBUTING.md](./CONTRIBUTING.md) first for setup, the commands to run, and the
pull request process. This file covers only what an agent gets wrong by default.

Everything below was verified by running it. Keep it that way. A confident but wrong
instruction is worse than a missing one, because the next contributor trusts it.

## The parity contract

cocox is a Rust port of [commitlint](https://github.com/opensource-nepal/commitlint). The
Python tool is the reference. If the two disagree for the same input, cocox is wrong,
unless a maintainer decides otherwise in writing.

Prove parity, never assume it. Run both and compare the exit code, stdout and stderr:

```bash
pip install commitlint==2.0.0
cargo build
./target/debug/cocox "feat:" ; echo "exit=$?"
commitlint "feat:"           ; echo "exit=$?"
```

Upstream's own table of cases is at `tests/fixtures/linter.py` in its repository. It is
ported here as `tests/upstream_parity.rs`.

Two upstream behaviors are defects, not contracts. Do not copy them: upstream raises
`IndexError` on an empty or comment-only `--file`, and it accepts an arbitrarily large
`--max-header-length`.

## Porting traps

Python and Rust look alike at each of these points and behave differently. Each one has
produced a real defect, either on `main` or in a pull request against it.

1. **Count characters, not bytes.** Python `len()` on a str counts code points; Rust
   `.len()` counts UTF-8 bytes. `"feat: नमस्ते संसार"` is 18 characters and 40 bytes. Use
   `.chars().count()`. `clippy.toml` denies `str::len` for this reason.
2. **A missing regex group is falsy, not an early return.** Python `m.group("x")` is
   `None` when the group did not participate, and `if not ...` catches it. In Rust,
   `captures.name("x")?` returns from the whole function and the error is never recorded.
   Write `captures.name("x").map_or("", |m| m.as_str())` instead.
3. **Normalize newlines where you read input.** Upstream reads git output with
   `text=True` and files with `open()`, so Python converts `\r\n` and lone `\r` to `\n`
   before linting. cocox reads raw bytes. Normalize in the readers only. Upstream does not
   normalize a message passed as a command-line argument, so neither must cocox.
4. **Anchor every regex you port.** Python `re.match` anchors at position 0; the Rust
   `regex` crate searches unless you write `^`. Note also that Rust `$` does not match
   before a trailing newline the way Python `$` does.
5. **`splitlines()` is not `lines()`.** Python also breaks on `\r`, `\x0b`, `\x0c`,
   `\x1c`, `\x1d`, `\x1e`, `\x85`, U+2028 and U+2029. Rust `lines()` breaks on `\n` only.
6. **`.strip()` is not `.trim()`.** Trim at exactly the places upstream strips, and note
   that Python treats `\x1c` to `\x1f` as whitespace while Rust does not.
7. **Constant order is user-visible.** Upstream joins `COMMIT_TYPES` into the
   "Type must be one of: ..." error, so the array order becomes text a user reads. Keep
   ported constant arrays in upstream order even when nothing prints them yet.
8. **Send every user-facing line through one output path.** Upstream routes everything
   through its console layer, which returns early when quiet is set, and catches expected
   failures. An error that escapes to `main` prints an anyhow chain that no quiet flag can
   suppress.
9. **Port an argparse type function into a clap `value_parser`.** Reproduce upstream's
   error text, and add `allow_negative_numbers = true`, or clap treats `-5` as a flag and
   the parser never runs.
10. **Pass lint options as arguments.** Keep `lint_commit_message` a function of its
    arguments, as upstream does with its `AppParams` dataclass. A process global makes the
    same input return different results depending on what ran before, and makes unit tests
    race.

## Testing rules

- Assert the exact message, not just the exit code. cocox and upstream both exit 2 for
  every bad argument, so an exit code alone proves nothing.
- Prefer an exact match over a substring. `contains("")` is true for every string.
- Never delete a test to make a change pass. Port it, or say in the pull request why the
  behavior it pinned is gone.
- Add the input that broke to `tests/cli.rs` when you fix a parity bug. Do not add it to
  `tests/upstream_parity.rs`, which is a verbatim mirror of upstream's table and asserts
  its own row count, so a local row makes it fail.
- `TestRepo` changes the working directory of the whole test process, so any test using it
  needs `#[serial]`. Do not remove that attribute to make tests faster.

## Known divergences from upstream

Open defects, not decisions. Each was reproduced against commitlint 2.0.0. Fix them in
separate pull requests. This list is what has been measured, not everything that exists.

1. A leading space fails here and passes upstream, because the direct message and
   `--file` contents are never stripped.
2. A CRLF message file fails here and passes upstream.
3. `Initial commit\x0cgarbage` is linted here and ignored upstream, per trap 5.
4. A trailing `\x1c` to `\x1f` survives `trim()` here and is stripped upstream.
5. `COMMIT_TYPES` lists `bump` second; upstream lists it last, per trap 7.
6. A missing `--file` prints a multi-line anyhow chain; upstream prints one line,
   `Error: file '<path>' not found`.

## Working rules

- **Keep scratch work out of the repository.** Put probe crates, harness scripts and
  clones of upstream in a temporary directory. Never commit one. To compare against main
  without disturbing the checkout:
  `mkdir -p /tmp/cocox-main && git archive main | tar -x -C /tmp/cocox-main`
- **Do not write unverified claims into the repository.** Check every factual sentence you
  add to a document against the code or a command. Do not write that a gate, a test or a
  feature exists until you have seen it run. Prefer "measured so far" over "complete".
- **Report what you actually ran.** If tests fail, say so and show the output. If you
  skipped a step, say which. Do not describe work as done until it is done.
- **One concern per pull request.** Do not add repository policy to a feature pull
  request. Propose policy separately so it can be discussed on its own.
