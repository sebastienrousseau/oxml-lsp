#!/usr/bin/env bash
#
# The command-line linter: files, standard input, and exit codes.
#
# Asserts rather than prints. A README full of example invocations goes
# stale the moment behaviour changes and still looks correct.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="${OXML_LSP:-oxml-lsp}"
DATA="$HERE/data"
failures=0

expect() {
  local description="$1" want_code="$2" want_out="$3"; shift 3
  local out code
  set +e
  out="$("$BIN" "$@" 2>&1)"; code=$?
  set -e
  if [[ "$code" != "$want_code" || "$out" != "$want_out" ]]; then
    echo "FAIL: $description"
    echo "  exit   : got $code, want $want_code"
    echo "  output : got ${out@Q}"
    echo "           want ${want_out@Q}"
    failures=$((failures + 1))
  else
    echo "ok: $description"
  fi
}

expect "a well-formed document produces nothing" 0 \
  "no diagnostics" "$DATA/valid.xml"

# One diagnostic, with a line and column counted in characters, and a
# stable code to match on.
expect "a mismatched tag is reported with its position" 1 \
  "3:13: error: at byte 39: </hostname> closes <port> [not-well-formed]" \
  "$DATA/mismatched.xml"

# Exit 2 is "I could not read that", distinct from exit 1 "the document
# has a problem". A script needs to tell those apart.
expect "a missing file is a different failure from a bad document" 2 \
  "oxml-lsp: cannot read $DATA/nope.xml: No such file or directory (os error 2)" \
  "$DATA/nope.xml"

# Reads standard input when given no file.
got="$(printf '<a>' | "$BIN" 2>&1)" || true
if [[ "$got" != *"input ended unexpectedly"* ]]; then
  echo "FAIL: reading from standard input: ${got@Q}"; failures=$((failures + 1))
else
  echo "ok: reading from standard input"
fi

# The column counts characters, not bytes. Two accented characters
# before the error make the two differ: `</b>` starts at character 6
# and at byte 8. One accent would not -- they coincide there, and the
# assertion would pass without testing anything.
got="$(printf '<a>\xc3\xa9\xc3\xa9</b>' | "$BIN" 2>&1)" || true
if [[ "$got" != *":6: error"* ]]; then
  echo "FAIL: the column counts characters, not bytes: ${got@Q}"
  failures=$((failures + 1))
else
  echo "ok: the column counts characters, not bytes"
fi

[[ "$failures" -eq 0 ]] || { echo "$failures failed"; exit 1; }
echo "all assertions passed"
