# Shared Library Interface

## Overview

This proposal aims to expose Scryer Prolog as a shared library (`libscryer_prolog.so`) with a C API, enabling embedding Scryer Prolog in other programming languages and applications.

## Motivation

- Enable Scryer Prolog to be used from languages like Python, C, C++, Rust, etc.
- Provide a stable C ABI for foreign function interfaces
- Support embedding Prolog logic in larger applications
- Facilitate building language bindings and MCP servers

## Implementation

### Branches
- `ISSUE-2464/scryer-prolog-shared-lib` - Main shared library implementation
- `ISSUE-2464/scryer-prolog-shared-lib-eval-code-c` - C code evaluation interface

### API Design

The shared library exposes functions for:
- Initializing and shutting down the Prolog runtime
- Loading Prolog code from strings or files
- Querying the Prolog database
- Retrieving results and bindings
- Memory management

### Example Usage

```c
// Initialize Prolog runtime
scryer_prolog_init();

// Load Prolog code
scryer_prolog_consult_string("parent(tom, bob). parent(tom, liz).");

// Query
scryer_prolog_query("parent(tom, X)");

// Get results
while (scryer_prolog_has_solution()) {
    char* binding = scryer_prolog_get_binding("X");
    printf("X = %s\n", binding);
    scryer_prolog_next_solution();
}

// Cleanup
scryer_prolog_cleanup();
```

## Status

**Current**: In active development

## Related Projects

- [scryer-prolog-c-api](https://github.com/jjtolton/scryer-prolog-c-api) - C API bindings
- [libscryer-clj](https://github.com/jjtolton/libscryer-clj) - Clojure bindings

## Discussion

See [Issue #2464](https://github.com/mthom/scryer-prolog/issues/2464) for upstream discussion.
