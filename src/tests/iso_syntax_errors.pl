:- module(iso_syntax_errors_tests, []).
:- use_module(test_framework).
:- use_module(library(charsio)).

% ISO/IEC 13211-1 Technical Corrigendum 2, Section C2
% See https://www.complang.tuwien.ac.at/ulrich/iso-prolog/dtc2#C2

% Note: (|) is VALID syntax - it's the atom '|' in parentheses
% The parentheses bracket the atom to give it priority 0
% This is NOT an operator application (which would require operands)
test("single_bar_in_parens_should_parse_to_atom", (
    read_from_chars("(|).", T),
    T == '|'
)).

test("op_create_empty_curly_should_error", (
    catch(
        op(500, xfy, {}),
        error(permission_error(create, operator, {}), _),
        true
    )
)).

test("op_create_empty_curly_in_list_should_error", (
    catch(
        op(500, xfy, [{}]),
        error(permission_error(create, operator, {}), _),
        true
    )
)).

test("op_create_bar_priority_1000_should_error", (
    catch(
        op(1000, xfy, '|'),
        error(permission_error(create, operator, '|'), _),
        true
    )
)).

test("op_create_bar_in_list_priority_1000_should_error", (
    catch(
        op(1000, xfy, ['|']),
        error(permission_error(create, operator, '|'), _),
        true
    )
)).

test("op_create_bar_prefix_should_error", (
    catch(
        op(1150, fx, '|'),
        error(permission_error(create, operator, '|'), _),
        true
    )
)).

test("op_create_bar_priority_1105_should_succeed", (
    op(1105, xfy, '|'),
    % Clean up
    op(0, xfy, '|')
)).

test("op_remove_bar_should_succeed", (
    op(1105, xfy, '|'),
    op(0, xfy, '|')
)).

% Regression tests for (|) parsing - must never break
% These ensure the atom '|' can always be written as (|) in parentheses

test("bar_in_parens_with_operator_defined_should_still_parse", (
    op(1105, xfy, '|'),
    read_from_chars("(|).", T),
    op(0, xfy, '|'),
    T == '|'
)).

test("bar_in_nested_parens_should_parse", (
    read_from_chars("((|)).", T),
    T == '|'
)).

test("bar_in_term_context_should_parse", (
    read_from_chars("foo((|)).", T),
    T == foo('|')
)).

test("bar_in_list_should_parse", (
    read_from_chars("[(|)].", T),
    T == ['|']
)).

% Test that (|) in error terms works correctly
% This verifies builtins.pl using (|) in permission_error term parses correctly
test("bar_in_error_term_should_match", (
    catch(
        op(999, xfy, '|'),
        error(permission_error(create, operator, (|)), _),
        true
    )
)).

% Verify (|) and '|' are the same atom
test("bar_parens_equals_bar_quoted", (
    (|) == '|'
)).
