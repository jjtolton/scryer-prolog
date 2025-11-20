# Issue #3156: Bracketed terms should not be usable as functors

Tests that patterns like `((>)(1))` and `((a)(b))` throw syntax errors,
while valid patterns like `a(b)`, `(a)`, and `((>))` parse correctly.

```trycmd
$ scryer-prolog -f --no-add-history issue_3156_test.pl -g test -g halt
Testing ((>)(1) throws syntax error... PASS
Testing ((a)(b) throws syntax error... PASS
Testing (a) (b) throws syntax error... PASS
Testing a(b) parses correctly... PASS
Testing (a) parses correctly... PASS
Testing ((>)) parses correctly... PASS
All tests passed!

```
