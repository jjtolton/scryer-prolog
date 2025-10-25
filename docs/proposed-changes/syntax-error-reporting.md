# Improved Syntax Error Reporting

## Overview

Enhancement of Scryer Prolog's syntax error messages to provide more helpful and detailed information when parsing fails.

## Motivation

Better error messages help developers:
- Quickly identify and fix syntax errors
- Understand what went wrong and why
- Learn Prolog syntax more effectively
- Debug complex code more efficiently

## Proposed Improvements

### More Contextual Information
- Show the exact location of the error with line and column numbers
- Display the surrounding code context
- Highlight the specific token or construct causing the issue

### Clearer Error Messages
- Explain what was expected vs. what was found
- Provide suggestions for common mistakes
- Use human-readable descriptions instead of parser internals

### Example

**Before:**
```
Error: syntax error
```

**After:**
```
Syntax Error at line 5, column 12:
  parent(tom, bob.
              ^
Expected: closing parenthesis ')'
Found: period '.'

Hint: Did you forget to close the parenthesis?
```

## Implementation Status

**Current**: In development on `improve-syntax-error-reporting` branch

## Testing

Test cases cover:
- Missing parentheses
- Incorrect operators
- Malformed terms
- Invalid clause syntax
