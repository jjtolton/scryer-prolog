# Scryer Prolog Combined Testing

[![CI](https://github.com/jjtolton/scryer-prolog/actions/workflows/ci.yml/badge.svg?branch=combined-testing)](https://github.com/jjtolton/scryer-prolog/actions/workflows/ci.yml)

Integration testing branch for [mthom/scryer-prolog](https://github.com/mthom/scryer-prolog).

## Included Features

| Feature | Description | Reference | PR |
|---------|-------------|-----------|-----|
| `||` operator | Partial string list continuation | [WG17 2025](https://www.complang.tuwien.ac.at/ulrich/iso-prolog/double_bar) | [#3135](https://github.com/mthom/scryer-prolog/pull/3135) |
| `(|)` syntax | Bar atom in parentheses (valid) | [ISO TC2 C2](https://www.complang.tuwien.ac.at/ulrich/iso-prolog/dtc2#C2) | [#3172](https://github.com/mthom/scryer-prolog/pull/3172) |
| Bracket errors | Reject incomplete `([`, `({` | ISO 6.3 | [#3139](https://github.com/mthom/scryer-prolog/pull/3139) |

## Tests Added Since upstream/master

| File | Tests | Category | Reference | PR |
|------|-------|----------|-----------|-----|
| `src/tests/double_bar.pl` | 56 | `\|\|` string continuation | WG17 2025 | [#3135](https://github.com/mthom/scryer-prolog/pull/3135) |
| `src/tests/digit_separators.pl` | 36 | Numeric underscore separators | ISO extension | [#3134](https://github.com/mthom/scryer-prolog/pull/3134) |
| `src/tests/iso_syntax_errors.pl` | 14 | Bar operator `(\|)` syntax | ISO TC2 C2 | [#3172](https://github.com/mthom/scryer-prolog/pull/3172) |
| `src/tests/double_bar_syntax_errors.pl` | 8 | Invalid `\|\|` rejection | WG17 2025 | [#3135](https://github.com/mthom/scryer-prolog/pull/3135) |
| `src/tests/syntax_errors.pl` | 3 | Bracket mismatch errors | ISO 6.3 | [#3139](https://github.com/mthom/scryer-prolog/pull/3139) |
| **Total** | **117** | | | |

### ISO Syntax Errors Tests

| Test | Input | Expected | Spec |
|------|-------|----------|------|
| `single_bar_in_parens_should_parse_to_atom` | `(\|).` | atom `\|` | ISO TC2 C2 |
| `op_create_empty_curly_should_error` | `op(500,xfy,{})` | permission_error | ISO 8.14.3 |
| `op_create_bar_priority_1000_should_error` | `op(1000,xfy,'\|')` | permission_error | ISO TC2 #72 |
| `op_create_bar_prefix_should_error` | `op(1150,fx,'\|')` | permission_error | ISO TC2 |
| `op_create_bar_priority_1105_should_succeed` | `op(1105,xfy,'\|')` | success | ISO TC2 |
| `bar_in_parens_with_operator_defined` | `op(1105...), "(\|)."` | atom `\|` | ISO TC2 |
| `bar_in_nested_parens_should_parse` | `((\|)).` | atom `\|` | ISO 6.3.4 |
| `bar_in_error_term_should_match` | catch with `(\|)` | matches | ISO TC2 |
| `bar_parens_equals_bar_quoted` | `(\|) == '\|'` | true | ISO 6.3.4 |

### Double-Bar Syntax Error Tests

| Test | Input | Expected | Spec |
|------|-------|----------|------|
| `[]\|\|X list before \|\|` | `[]\|\|X.` | syntax_error | WG17 6.3.7 |
| `[a,b]\|\|X non-empty list` | `[a,b]\|\|X.` | syntax_error | WG17 6.3.7 |
| `X\|\|[] variable before \|\|` | `X\|\|[].` | syntax_error | WG17 6.3.2 |
| `foo\|\|X atom before \|\|` | `foo\|\|X.` | syntax_error | WG17 6.3.1.3 |
| `123\|\|X number before \|\|` | `123\|\|X.` | syntax_error | WG17 6.3.1.1 |
| `{foo\|\|bar} in curly` | `{foo\|\|bar}.` | syntax_error | WG17 6.3.6 |

## Links

- [CI Status](https://github.com/jjtolton/scryer-prolog/actions)
- [Upstream](https://github.com/mthom/scryer-prolog)
- [ISO Conformity Testing](https://www.complang.tuwien.ac.at/ulrich/iso-prolog/conformity_testing)
- [WG17 Double Bar Spec](https://www.complang.tuwien.ac.at/ulrich/iso-prolog/double_bar)

## Related PRs

| PR | Title | Status |
|----|-------|--------|
| [#3135](https://github.com/mthom/scryer-prolog/pull/3135) | Implement double bar (\|\|) operator for partial string lists | Open |
| [#3172](https://github.com/mthom/scryer-prolog/pull/3172) | Fix #3170: Reject (\|) when \| declared as operator | Open |
| [#3139](https://github.com/mthom/scryer-prolog/pull/3139) | Fix parser to reject incomplete reductions like ([, ({, (( | Open |
| [#3134](https://github.com/mthom/scryer-prolog/pull/3134) | Add digit separator support for binary, octal, and hexadecimal numbers | Open |
