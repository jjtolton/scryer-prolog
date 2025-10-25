# Scryer Prolog - Documentation

This site documents proposed enhancements, generative analyses, and changes to [Scryer Prolog](https://github.com/mthom/scryer-prolog).

## Generative Analysis

### [Garbage Collection: Technical Analysis](generative-analysis/garbage-collection.html)
Comprehensive technical analysis of memory management in Scryer Prolog, examining the consequences of missing GC and what it would take to implement it.

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
**Branch**: `discussion-3035`

**[View Full Technical Presentation →](get_n_chars_4.html)**
