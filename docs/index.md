# Scryer Prolog - Proposed Changes

This site documents proposed enhancements and changes to [Scryer Prolog](https://github.com/mthom/scryer-prolog).

## Active Proposals

### [Shared Library Interface](proposed-changes/shared-library.md)
Work in progress to expose Scryer Prolog as a shared library with C API for embedding in other languages.

**Status**: In Development
**Branch**: `ISSUE-2464/scryer-prolog-shared-lib`

### [Improved Syntax Error Reporting](proposed-changes/syntax-error-reporting.md)
Enhanced error messages and reporting for syntax errors in Prolog code.

**Status**: In Development
**Branch**: `improve-syntax-error-reporting`

### [Double Bar Syntax](proposed-changes/double-bar.md)
Implementation of double bar (`||`) syntax support.

**Status**: Proposed
**Branch**: `double-bar`

### [Digit Separators](proposed-changes/digit-separators.md)
Support for digit separators in numeric literals for improved readability.

**Status**: Proposed
**Branch**: `digit-separators-upstream`

### [Non-blocking I/O: get_n_chars/4 with Timeout](get_n_chars_4.html)
Timeout support for `get_n_chars/4` enabling non-blocking character reading from TCP sockets and process pipes.

**Status**: Complete
**Branch**: `discussion-3035`
**[View Full Technical Presentation →](get_n_chars_4.html)**

## Contributing

For general contribution guidelines, see the [main repository](https://github.com/mthom/scryer-prolog).

## About This Fork

This is [@jjtolton's fork](https://github.com/jjtolton/scryer-prolog) of Scryer Prolog, focused on experimental features and enhancements.
