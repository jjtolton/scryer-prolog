# Scryer Prolog - Documentation

This site documents proposed enhancements, generative analyses, and changes to [Scryer Prolog](https://github.com/mthom/scryer-prolog).

## Generative Analysis

### [Garbage Collection: Technical Analysis](generative-analysis/garbage-collection.html)
Comprehensive technical analysis of memory management in Scryer Prolog, examining the consequences of missing GC and what it would take to implement it.

**Analysis Date**: October 25, 2025
**Codebase Version**: [`ace10bb1`](https://github.com/jjtolton/scryer-prolog/commit/ace10bb1d00edd44c3a9f35ee28334d7c90bf48c)

**Topics Covered**:
- Current memory management architecture (heap, arena, atom table)
- 7 categories of memory leaks with quantified impact
- Implementation roadmap (5-7 months MVP, 12-19 months production-ready)
- Rust-specific challenges and design decisions

**[View Full Analysis →](generative-analysis/garbage-collection.html)**

---

## Active Proposals

### [Non-blocking I/O: get_n_chars/4 with Timeout](get_n_chars_4.html)
Timeout support for `get_n_chars/4` enabling non-blocking character reading from TCP sockets and process pipes.

**Status**: Complete
**Implementation Date**: October 24, 2025
**Branch**: `discussion-3035`
**Commit**: [`82432685`](https://github.com/jjtolton/scryer-prolog/commit/82432685424ad4113e1b19ce39fe1e3d7f547cb2)

**[View Full Technical Presentation →](get_n_chars_4.html)**
