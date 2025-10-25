# Digit Separators

## Overview

Support for digit separators (underscores) in numeric literals to improve readability of large numbers.

## Motivation

Large numeric literals can be difficult to read:
```prolog
% Hard to read
X = 1000000000.

% Much easier with separators
X = 1_000_000_000.
```

This is a common feature in modern programming languages (Python, Rust, Java, etc.) that significantly improves code readability.

## Proposed Syntax

```prolog
% Decimal numbers
Million = 1_000_000.
Billion = 1_000_000_000.

% Hexadecimal
Color = 0xFF_AA_BB.

% Binary
Flags = 0b1010_1010_1111_0000.

% Arbitrary grouping
Pi = 3_14159_26535.
```

## Rules

- Underscores are purely visual separators and ignored by the parser
- Cannot start or end a number with underscore
- Cannot have consecutive underscores
- Works with all numeric bases (decimal, hex, octal, binary)

## Status

**Current**: Proposed
**Branch**: `digit-separators-upstream`

## Implementation

The implementation extends the lexer to recognize and strip underscores from numeric literals during tokenization.

## Examples

```prolog
?- X = 1_000 + 999.
X = 1999.

?- Y is 0xFF_FF >> 8.
Y = 255.

?- Large = 9_223_372_036_854_775_807.
Large = 9223372036854775807.
```
