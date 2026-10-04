<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Testing

```bash
./scripts/gate.sh
```

That runs everything CI runs. The individual pieces are below.

## Unit and integration tests

34 tests over `analyse()`, every `Diagnostic` field, positions and
exit codes, and the LSP transport added at 0.0.8 -- including that
`Content-Length` counts bytes rather than characters, which a client
counting characters gets wrong on its first message containing one.

Positions are the part worth testing carefully. A diagnostic reports a
line and a *character* offset, not a byte offset, because that is what
an editor renders — see [POSITIONS.md](POSITIONS.md). A document
containing an emoji shifts the two apart, and getting it wrong puts
the squiggle in the wrong place rather than producing an error anyone
would notice.

## The examples assert their own output

```bash
OXML_LSP="$PWD/target/release/oxml-lsp" ./examples/lint.sh
```

`examples/lint.sh` compares both exit code and exact output, so the
invocations in the README fail CI when they stop being true. Until
0.0.7 nothing ran them, which is a claim the README was making on its
own behalf.

## Fuzzing

```bash
cargo +nightly fuzz run analyse
```

An editor calls `analyse()` on every keystroke, so it sees text that
is *not yet valid* far more often than text that is: half-typed tags,
unbalanced quotes, a document mid-paste. A panic there is a crashed
editor plugin.

The target asserts more than the absence of a panic — every diagnostic
must point into the document, and must not end before it starts.

That assertion was wrong on its first draft, and the fuzzer found it
within sixty seconds. On `<a id="=` followed by a newline it reported
a diagnostic on line 1 of what `str::lines()` calls a one-line
document. `str::lines()` is not the LSP line model: a trailing newline
opens a further, empty line. The diagnostic was right and the
assertion was counting wrong. That input is kept as a seed.

4,381,086 executions since the correction, no crashes. CI runs the
target for 300 seconds on every pull request.

## Coverage

Line coverage is gated in CI at a 95% floor. **Branch coverage is
92.3%**, gated at 80.

Branch coverage needs a nightly toolchain: `cargo llvm-cov --branch`
does not build on the version this project pins.

## Performance

`analyse()` is linear in document size. It was not always: a quadratic
pass over attributes cost 1,088 ms on a document that now takes 9.5,
and it was the benchmark that found it, not a reading of the code.

```bash
cargo bench --bench analyse
```

## What is not tested here

The parser and its conformance suite belong to `oxml` and are tested
there — 2,557 of 2,557 decided W3C tests, zero panics, six fuzz
targets, Miri and property tests. See
<https://github.com/sebastienrousseau/oxml/blob/main/doc/TESTING.md>.
