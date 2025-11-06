# 🔥 Bleeding Edge Features

This document tracks all pull requests that have been merged into the `hotness` branch but are not yet in `master`. These are experimental features undergoing testing and refinement before official release.

## Status Legend

- 🟢 **Ready** - Feature complete, thoroughly tested, ready for master
- 🟡 **Testing** - Feature complete, undergoing real-world testing
- 🔵 **WIP** - Work in progress, experimental
- 🟣 **Draft** - Early stage, subject to significant changes

---

## Parser & Syntax Enhancements

### 🟡 Digit Separators for Numeric Literals
**PR**: [#3134](https://github.com/mthom/scryer-prolog/pull/3134)
**Status**: Testing
**Branch**: `digit-separators-upstream`

Support for digit separators (`_`) in binary, octal, and hexadecimal numbers for improved readability.

**Examples**:
```prolog
?- X = 0b1111_0000_1010_1010.
X = 61610.

?- Y = 0xDEAD_BEEF.
Y = 3735928559.

?- Z = 0o777_666.
Z = 261046.
```

**Benefits**:
- Improved readability for large numbers
- Follows ISO Prolog extension proposals
- Common in modern programming languages

---

### 🟡 Double Bar Operator for Partial String Lists
**PR**: [#3135](https://github.com/mthom/scryer-prolog/pull/3135)
**Status**: Testing
**Branch**: `double-bar`

Implementation of the `||` operator for partial string lists following ISO Prolog standards.

**Examples**:
```prolog
?- S = "hello" || Rest.
S = [h,e,l,l,o|Rest].

?- append("world", X, "hello" || Y).
X = Y, Y = [].
```

**Benefits**:
- ISO Prolog compliance
- Cleaner syntax for partial strings
- Better pattern matching for string processing

**Reference**: [ISO Prolog Double Bar Specification](https://www.complang.tuwien.ac.at/ulrich/iso-prolog/double_bar)

---

### 🟡 Dyadic Quad Syntax
**PR**: [#3132](https://github.com/mthom/scryer-prolog/pull/3132)
**Status**: Testing
**Branch**: `binary-quad-syntax`

Added support for dyadic quad syntax.

**Examples**:
```prolog
% Binary quad operations
?- quad(A, B, C, D).
```

**Benefits**:
- Extended syntax support
- Better expressiveness for certain operations

---

### 🟢 Parser Rejection of Incomplete Reductions
**PR**: [#3139](https://github.com/mthom/scryer-prolog/pull/3139)
**Status**: Ready
**Branch**: `issue-3138`

Fixed parser to properly reject incomplete reductions like `([`, `({`, and `((` with appropriate syntax errors.

**Before**:
```prolog
?- X = ([.
% Could produce confusing errors or undefined behavior
```

**After**:
```prolog
?- X = ([.
error(syntax_error(incomplete_reduction), ...).
```

**Benefits**:
- Better error messages
- ISO Prolog compliance
- Prevents confusing parse errors

**Related Issue**: [#3138](https://github.com/mthom/scryer-prolog/issues/3138)

---

### 🟢 Single Bar Syntax Validation
**PR**: [#3141](https://github.com/mthom/scryer-prolog/pull/3141)
**Status**: Ready
**Branch**: `fix-single-bar-syntax-error`

Fixed parser to reject `(|)` as a syntax error per ISO specification.

**Examples**:
```prolog
?- X = (|).
error(syntax_error(incomplete_reduction), ...).
```

**Benefits**:
- ISO Prolog compliance
- Prevents invalid syntax from being accepted

**Reference**: [ISO Prolog Syntax Specification](https://www.complang.tuwien.ac.at/ulrich/iso-prolog/dtc2#C2)

---

## Error Handling & Diagnostics

### 🟢 Improved Syntax Error Reporting
**PR**: [#3133](https://github.com/mthom/scryer-prolog/pull/3133)
**Status**: Ready
**Branch**: `improve-syntax-error-reporting`

Enhanced error messages to include source file names for better debugging.

**Before**:
```
error(syntax_error(incomplete_reduction), read_term/3:8).
```

**After**:
```
error(syntax_error(incomplete_reduction), [file-'my_file.pl'|read_term/3:8]).
```

**Benefits**:
- Easier debugging of multi-file projects
- Better error context
- Follows established Prolog error reporting patterns

**Related Issue**: [#302](https://github.com/mthom/scryer-prolog/issues/302)

---

### 🟡 Script-Safe Execution Flags: `--halt-on-error` and `--always-halt`
**PR**: [#3147](https://github.com/mthom/scryer-prolog/pull/3147)
**Status**: Testing
**Branch**: `error-termination-flag`

Two complementary flags that make Scryer Prolog safe for scripting by preventing it from entering the REPL:

1. **`--halt-on-error`** - Terminate with exit code 1 when encountering errors
2. **`--always-halt`** - Always exit after execution completes (even without explicit `halt/0`)

**The Problem**:
Without these flags, Scryer drops into the REPL in two scenarios:
- When an error occurs during execution
- When a program completes without calling `halt/0`

This causes scripts to hang indefinitely waiting for user input.

**Usage**:
```bash
# Exit on errors only
scryer-prolog --halt-on-error my_file.pl

# Always exit after completion (no REPL even on success)
scryer-prolog --always-halt my_file.pl

# Best for scripting: combine both flags
scryer-prolog --always-halt --halt-on-error my_file.pl

# With goals
scryer-prolog --always-halt --halt-on-error -g "run_tests"
```

**Behavior**:

| Scenario | No flags | `--halt-on-error` | `--always-halt` | Both flags |
|----------|----------|-------------------|-----------------|------------|
| Success, no halt/0 | REPL | REPL | Exit 0 | Exit 0 |
| Success, with halt/0 | Exit 0 | Exit 0 | Exit 0 | Exit 0 |
| Error occurs | REPL | Exit 1 | REPL | Exit 1 |

**Use Cases**:
- **Shell scripts** that shouldn't hang on errors or after completion
- **CI/CD pipelines** requiring proper exit codes
- **Automated testing** where REPL interaction is impossible
- **Docker containers** that need to exit cleanly
- **Cron jobs** running unattended
- **Build systems** (Make, etc.)
- **Batch processing** scripts

**Examples**:

*CI/CD Pipeline*:
```bash
#!/bin/bash
set -e
scryer-prolog --always-halt --halt-on-error compile.pl
scryer-prolog --always-halt --halt-on-error -g "run_tests"
echo "All checks passed!"
```

*Makefile*:
```makefile
test:
	scryer-prolog --always-halt --halt-on-error -g "run_all_tests"
```

*One-shot data processing*:
```bash
scryer-prolog --always-halt --halt-on-error process_data.pl < input.txt > output.txt
```

**Benefits**:
- Prevents scripts from hanging indefinitely
- Proper exit codes (0 for success, 1 for errors)
- No REPL interaction in headless environments
- Works with files and goals
- Composable: use one or both flags as needed

**Related Issue**: [#3146](https://github.com/mthom/scryer-prolog/issues/3146)

---

## I/O & Streams

### 🟡 Character Streams for REBIS Pattern
**PR**: [#2968](https://github.com/mthom/scryer-prolog/pull/2968)
**Status**: Testing (Long-running PR since May 2025)
**Branch**: `memory-stream`

Added `chars_stream/1` and `chars_to_stream/{2,3}` for REBIS (REading By Incremental Scanning) pattern support.

**Predicates**:
```prolog
% Create a stream from a list of characters
chars_stream(+Chars).

% Convert characters to a stream
chars_to_stream(+Chars, -Stream).
chars_to_stream(+Chars, -Stream, +Options).
```

**Examples**:
```prolog
?- chars_stream("hello world").
true.

?- chars_to_stream("test data", Stream),
   get_char(Stream, C),
   close(Stream).
C = t.
```

**Benefits**:
- Efficient incremental parsing
- In-memory stream processing
- Useful for testing and text processing

---

### 🟢 Timeout and Non-blocking Support for `get_n_chars/4`
**PR**: [#3136](https://github.com/mthom/scryer-prolog/pull/3136)
**Status**: Ready
**Branch**: `discussion-3035`

Enhanced `get_n_chars/4` with timeout and non-blocking I/O support.

**Signature**:
```prolog
get_n_chars(+Stream, ?N, -Chars, +Timeout).
```

**Timeout Modes**:
- `0` or negative: No timeout (same as `get_n_chars/3`)
- Positive integer: Timeout in milliseconds
- `infinity`: No timeout (explicit)
- `nonblock`: Return immediately with available data

**Examples**:
```prolog
% Read up to 100 chars with 5 second timeout
?- get_n_chars(Stream, 100, Chars, 5000).

% Non-blocking read
?- get_n_chars(Stream, N, Chars, nonblock).

% Read all available data with timeout
?- get_n_chars(Stream, N, Chars, 2000).
```

**Features**:
- Proper UTF-8 boundary handling across timeouts
- Non-blocking mode for async I/O
- Preserves incomplete UTF-8 sequences

**Benefits**:
- Network I/O with timeouts
- Interactive applications
- Real-time data processing
- Prevents hanging on slow streams

**Related Discussion**: [#3035](https://github.com/mthom/scryer-prolog/discussions/3035)

---

## Term Expansion

### 🟢 Empty List Handling in `term_expansion/2`
**PR**: [#3137](https://github.com/mthom/scryer-prolog/pull/3137)
**Status**: Ready
**Branch**: `term-expansion-empty-list-clean`

Allows `term_expansion/2` to return empty list `[]` to silently remove terms without warnings.

**Before**:
```prolog
term_expansion(remove_me, []).
% Would generate warnings about removing terms
```

**After**:
```prolog
term_expansion(remove_me, []).
% Silently removes the term, no warnings
```

**Use Cases**:
- Conditional compilation
- Removing debugging code
- Platform-specific term filtering
- Clean macro expansion

**Benefits**:
- Cleaner output during compilation
- Better control over term transformation
- No spurious warnings

---

## Foreign Function Interface

### 🔵 RTLD_GLOBAL Support for FFI
**PR**: [#3144](https://github.com/mthom/scryer-prolog/pull/3144)
**Status**: WIP
**Branch**: `rtld-global-support`

Added RTLD_GLOBAL flag support to FFI for Python C extension compatibility.

**Context**:
Enables loading shared libraries with global symbol visibility, required for some Python C extensions and other foreign libraries that depend on symbol sharing between libraries.

**Use Case**:
```prolog
% Load library with global symbols
?- ffi_load_library('libpython3.so', [rtld_global(true)], Handle).
```

**Benefits**:
- Python C extension compatibility
- Better interoperability with foreign libraries
- Support for libraries with inter-dependencies

**Status Note**: This is still WIP as it requires more testing with various Python extensions and C libraries.

---

## Testing Status

All features in this branch have been:
- ✅ Locally tested in the hotness branch
- ✅ Compiled successfully
- ✅ Integration tested with other hotness features
- 🔄 Awaiting review and feedback before master merge

## Building the Hotness Branch

```bash
git clone https://github.com/jjtolton/scryer-prolog.git
cd scryer-prolog
git checkout hotness
cargo build --release
./target/release/scryer-prolog --help
```

## Contributing

If you'd like to test these features or provide feedback:

1. Check out the hotness branch
2. Test the features in your use case
3. Report issues or feedback on the relevant PR
4. Help us get these features ready for master!

---

**Last Updated**: 2025-11-05
**Total Features**: 11 PRs
**Ready for Master**: 4 features
**In Testing**: 6 features
**WIP**: 1 feature

---

*This branch represents the cutting edge of Scryer Prolog development. Features here are experimental and subject to change based on testing and feedback.*
