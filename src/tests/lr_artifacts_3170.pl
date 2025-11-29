%% Issue #3170: LR artifacts
%% https://github.com/mthom/scryer-prolog/issues/3170
%%
%% ISO 6.3.4: Operators require operands per specifier
%% ISO 6.3.6: {term} requires valid term inside

:- module(lr_artifacts_3170_tests, []).
:- use_module(test_framework).
:- use_module(library(charsio)).

test("lr_artifacts_uwn_case_should_not_produce_garbage", (
    op(1105, xfy, '|'),
    catch(
        read_from_chars("{!*!(|)/}.", T),
        _,
        T = caught_error
    ),
    op(0, xfy, '|'),
    T \== '{}'('/')
)).
