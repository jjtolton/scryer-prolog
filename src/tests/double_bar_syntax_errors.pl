:- module(double_bar_syntax_errors_tests, []).
:- use_module(test_framework).
:- use_module(library(charsio)).

% WG17 2025 double bar syntax error tests
% Reference: https://www.complang.tuwien.ac.at/ulrich/iso-prolog/double_bar
% The || operator only valid directly after double-quoted string token
% term = double quoted list, bar, bar, term ;

% Invalid || usage - list before || violates WG17 grammar
% ISO 6.3.5: [] is list notation, not double quoted list (6.3.7)

test("[]||X list before || should error", (
    catch(
        (read_from_chars("[]||X.", _), false),
        error(syntax_error(_), _),
        true
    )
)).

test("[a,b]||X non-empty list before || should error", (
    catch(
        (read_from_chars("[a,b]||X.", _), false),
        error(syntax_error(_), _),
        true
    )
)).

test("X||[] variable before || should error", (
    % ISO 6.3.2: Variables have priority 0, are not double quoted lists
    catch(
        (read_from_chars("X||[].", _), false),
        error(syntax_error(_), _),
        true
    )
)).

test("foo||X atom before || should error", (
    % ISO 6.3.1.3: Atoms are atomic terms, not double quoted lists
    catch(
        (read_from_chars("foo||X.", _), false),
        error(syntax_error(_), _),
        true
    )
)).

test("123||X number before || should error", (
    % ISO 6.3.1.1: Numbers are atomic terms, not double quoted lists
    catch(
        (read_from_chars("123||X.", _), false),
        error(syntax_error(_), _),
        true
    )
)).

test("{foo||bar} atoms in curly brackets should error", (
    % ISO 6.3.6: {term} wraps term, but foo is atom not double-quoted string
    catch(
        (read_from_chars("{foo||bar}.", _), false),
        error(syntax_error(_), _),
        true
    )
)).

test("[[]||X] nested list context should error", (
    catch(
        (read_from_chars("[[]||X].", _), false),
        error(syntax_error(_), _),
        true
    )
)).

test("{[]||!} list before || in curly should error", (
    catch(
        (read_from_chars("{[]||!}.", _), false),
        error(syntax_error(_), _),
        true
    )
)).
