:- module(bar_syntax_tests, []).
:- use_module(test_framework).
:- use_module(library(charsio)).

% ISO/IEC 13211-1 Technical Corrigendum 2, Section C2
% Bar Syntax Tests
%
% Per TC2 C2:
% - "A bar (6.4) shall be equivalent to the atom '|' when '|' is an operator."
% - "Bar is also a solo character (6.5.3), and a token (6.4) but not an atom."
%
% Therefore:
% - (|) when | is NOT an operator -> syntax_error (bar is "not an atom")
% - (|) when | IS an operator -> valid, parses as atom '|'
%
% See: https://www.complang.tuwien.ac.at/ulrich/iso-prolog/dtc2#C2

% ============================================================================
% Section 1: (|) when | is NOT an operator (default state)
% Expected: syntax_error because bar is "not an atom" per TC2 C2
% ============================================================================

test("bar_not_operator_parens_should_error", (
    catch(
        (read_from_chars("(|).", _), false),
        error(syntax_error(_), _),
        true
    )
)).

test("bar_not_operator_in_term_should_error", (
    catch(
        (read_from_chars("foo((|)).", _), false),
        error(syntax_error(_), _),
        true
    )
)).

test("bar_not_operator_in_list_should_error", (
    catch(
        (read_from_chars("[(|)].", _), false),
        error(syntax_error(_), _),
        true
    )
)).

% ============================================================================
% Section 2: (|) when | IS an operator (after op/3 declaration)
% Expected: valid, parses as atom '|' per TC2 C2
% "A bar shall be equivalent to the atom '|' when '|' is an operator"
% ============================================================================

test("bar_is_operator_parens_should_succeed", (
    op(1105, xfy, '|'),
    catch(
        read_from_chars("(|).", T),
        _,
        false
    ),
    op(0, xfy, '|'),
    T == '|'
)).

test("bar_is_operator_equals_quoted", (
    op(1105, xfy, '|'),
    catch(
        read_from_chars("(|).", T),
        _,
        false
    ),
    op(0, xfy, '|'),
    read_from_chars("'|'.", Q),
    T == Q
)).

test("bar_is_operator_in_term_should_succeed", (
    op(1105, xfy, '|'),
    catch(
        read_from_chars("foo((|)).", T),
        _,
        false
    ),
    op(0, xfy, '|'),
    T == foo('|')
)).

test("bar_is_operator_in_list_should_succeed", (
    op(1105, xfy, '|'),
    catch(
        read_from_chars("[(|)].", T),
        _,
        false
    ),
    op(0, xfy, '|'),
    T == ['|']
)).

% ============================================================================
% Section 3: (|) after removing operator status
% Expected: syntax_error (back to "not an atom" state)
% ============================================================================

test("bar_after_op_removal_should_error", (
    op(1105, xfy, '|'),
    op(0, xfy, '|'),
    catch(
        (read_from_chars("(|).", _), false),
        error(syntax_error(_), _),
        true
    )
)).

% ============================================================================
% Section 4: Quoted '|' always valid (per ISO 6.4.2)
% Quoted atoms are always atoms regardless of operator status
% ============================================================================

test("quoted_bar_always_valid_not_operator", (
    read_from_chars("'|'.", T),
    T == '|'
)).

test("quoted_bar_in_parens_always_valid", (
    read_from_chars("('|').", T),
    T == '|'
)).

test("quoted_bar_always_valid_with_operator", (
    op(1105, xfy, '|'),
    read_from_chars("'|'.", T),
    op(0, xfy, '|'),
    T == '|'
)).

% ============================================================================
% Section 5: op/3 restrictions for '|' (per TC2 C2)
% - '|' can only be infix (xfx, xfy, yfx)
% - Priority must be >= 1001
% ============================================================================

test("op_bar_priority_below_1001_should_error", (
    catch(
        op(1000, xfy, '|'),
        error(permission_error(create, operator, '|'), _),
        true
    )
)).

test("op_bar_priority_1001_should_succeed", (
    op(1001, xfy, '|'),
    op(0, xfy, '|')
)).

test("op_bar_priority_1105_should_succeed", (
    op(1105, xfy, '|'),
    op(0, xfy, '|')
)).

test("op_bar_prefix_should_error", (
    catch(
        op(1105, fx, '|'),
        error(permission_error(create, operator, '|'), _),
        true
    )
)).

test("op_bar_postfix_should_error", (
    catch(
        op(1105, xf, '|'),
        error(permission_error(create, operator, '|'), _),
        true
    )
)).

% ============================================================================
% Section 6: {} and [] operator restrictions (per ISO 8.14.3)
% ============================================================================

test("op_empty_curly_should_error", (
    catch(
        op(500, xfy, {}),
        error(permission_error(create, operator, {}), _),
        true
    )
)).

test("op_empty_list_should_error", (
    catch(
        op(500, xfy, []),
        error(permission_error(create, operator, []), _),
        true
    )
)).
