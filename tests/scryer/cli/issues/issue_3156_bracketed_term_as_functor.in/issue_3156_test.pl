:- use_module(library(charsio)).
:- use_module(library(format)).

test_syntax_error(Input) :-
    catch(
        read_from_chars(Input, _),
        error(syntax_error(_), _),
        true
    ).

test_parses_to(Input, Expected) :-
    read_from_chars(Input, Term),
    Term == Expected.

test :-
    format("Testing ((>)(1) throws syntax error... ", []),
    ( test_syntax_error("((>)(1).") -> format("PASS~n", []) ; format("FAIL~n", []), halt(1) ),

    format("Testing ((a)(b) throws syntax error... ", []),
    ( test_syntax_error("((a)(b).") -> format("PASS~n", []) ; format("FAIL~n", []), halt(1) ),

    format("Testing (a) (b) throws syntax error... ", []),
    ( test_syntax_error("(a) (b).") -> format("PASS~n", []) ; format("FAIL~n", []), halt(1) ),

    format("Testing a(b) parses correctly... ", []),
    ( test_parses_to("a(b).", a(b)) -> format("PASS~n", []) ; format("FAIL~n", []), halt(1) ),

    format("Testing (a) parses correctly... ", []),
    ( test_parses_to("(a).", a) -> format("PASS~n", []) ; format("FAIL~n", []), halt(1) ),

    format("Testing ((>)) parses correctly... ", []),
    ( test_parses_to("((>)).", (>)) -> format("PASS~n", []) ; format("FAIL~n", []), halt(1) ),

    format("All tests passed!~n", []).
