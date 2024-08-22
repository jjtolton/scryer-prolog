#[cfg(test)]
mod dll_tests {
    use std::ffi::{CStr, CString};

    use serde_json::{json, Value};

    // uncomment if we can figure out why this isn't working
    // use crate::lib::dll::{machine_free};
    use scryer_prolog::machine::shared_library::shared_library::dll::{consult_module_string, free_c_string, machine_free, machine_new, query_state_free, run_query, run_query_iter, run_query_next};
    use scryer_prolog::machine::Machine;

    #[test]
    fn test_scryer_run_multiple_queries_greedy_evaluation() {
        let queries = vec![
            CString::new("true.").unwrap(),
            CString::new("false.").unwrap(),
            CString::new("X=2.").unwrap(),
            CString::new("member(a, [a, b, c]).").unwrap(),
            CString::new(r#"member(A, [a, b, c, "a", "b", "c", f(a), "f(a)"])."#).unwrap(),
        ];

        let expected_results = vec![
            json!({"status": "ok", "result": true}),
            json!({"status": "ok", "result": false}),
            json!({"status": "ok", "result": [{"X":2}]}),
            json!({"status": "ok", "result": true}),
            json!({"status": "ok", "result": [{"A": "a"}, {"A": "b"}, {"A": "c"},
            {"A": "\"a\""}, {"A": "\"b\""}, {"A": "\"c\""}, {"A": "f(a)"}, {"A": "\"f(a)\""}]}
            ),
        ];

        let machine_ptr: *mut Machine = machine_new();
        let module_name = CString::new("tests").unwrap();
        let program = CString::new(":- use_module(library(lists)).").unwrap();
        unsafe {
            consult_module_string(&mut *machine_ptr, module_name.as_ptr(), program.as_ptr());
        }

        for (query, expected) in queries.iter().zip(expected_results.iter()) {
            let result = unsafe { run_query(&mut *machine_ptr, query.as_ptr()) };
            let cstr = unsafe { CStr::from_ptr(result) };
            let output_string = cstr.to_str().unwrap().to_owned();
            let deserialized: Value = serde_json::from_str(&output_string).unwrap();
            assert_eq!(expected, &deserialized);
            unsafe { free_c_string(result) };
        }

        // breaks if uncommented
        unsafe { machine_free(machine_ptr); }
    }

    #[test]
    fn test_scryer_run_query_equal_variables() {
        let program =
            CString::new(":- use_module(library(lists)). :- use_module(library(dif)).").unwrap();
        let module_name = CString::new("facts").unwrap();
        let query = CString::new("X=Y.").unwrap();
        let machine_ptr: *mut Machine = machine_new();
        // Check if pointer is not null
        assert!(!machine_ptr.is_null());
        unsafe {
            consult_module_string(&mut *machine_ptr, module_name.as_ptr(), program.as_ptr());
        }
        let query_state = unsafe { run_query_iter(&mut *machine_ptr, query.as_ptr()) };

        // should be X=Y not Y=X, see https://github.com/mthom/scryer-prolog/pull/2465#issuecomment-2294961856
        // expected fix with https://github.com/mthom/scryer-prolog/pull/2475
        let expected_results = [
            r#"{"result":[{"Y":"X"}],"status":"ok"}"#, // should be:
                                                       // "{\"result\":[{\"X\":\"Y\"}],\"status\":\"ok\"}"
        ];

        let query_state_ref = unsafe { &mut *query_state };
        for expected in &expected_results {
            let result_ptr = run_query_next(&mut *query_state_ref);
            let result_char = unsafe { CStr::from_ptr(result_ptr) };
            let result_s = result_char.to_str().unwrap();
            let result_obj = serde_json::from_str::<serde_json::Value>(result_s).expect("Bad JSON");
            let expected_obj =
                serde_json::from_str::<serde_json::Value>(expected).expect("Bad JSON");
            println!("{result_s:?}");
            assert_eq!(result_obj, expected_obj);
            unsafe {
                free_c_string(result_ptr);
            }
        }

        unsafe { query_state_free(query_state) }
        unsafe { machine_free(machine_ptr); }
    }

    #[test]
    fn test_scryer_run_query_true_members() {
        let program =
            CString::new(":- use_module(library(lists)). :- use_module(library(dif)).").unwrap();
        let module_name = CString::new("facts").unwrap();
        let query = CString::new(r#"member(X, [a,"a",f(a),"f(a)", true, "true", false, "false"])."#).unwrap();
        let machine_ptr: *mut Machine = machine_new();
        unsafe {
            consult_module_string(&mut *machine_ptr, module_name.as_ptr(), program.as_ptr());
        }
        let query_state = unsafe { run_query_iter(&mut *machine_ptr, query.as_ptr()) };

        let expected_results = [
            r#"{"result":[{"X":"a"}],"status":"ok"}"#,
            r#"{"result":[{"X":"\"a\""}],"status":"ok"}"#,
            r#"{"result":[{"X":"f(a)"}],"status":"ok"}"#,
            r#"{"result":[{"X":"\"f(a)\""}],"status":"ok"}"#,
            r#"{"result":[{"X": true}],"status":"ok"}"#,
            r#"{"result":[{"X":"\"true\""}],"status":"ok"}"#,
            r#"{"result":[{"X": false}],"status":"ok"}"#,
            r#"{"result":[{"X":"\"false\""}],"status":"ok"}"#,
        ];

        let query_state_ref = unsafe { &mut *query_state };
        for expected in &expected_results {
            let result_ptr = run_query_next(query_state_ref);
            let result_char = unsafe { CStr::from_ptr(result_ptr) };
            let result_s = result_char.to_str().unwrap();
            let result_obj =
                serde_json::from_str::<serde_json::Value>(&result_s).expect("Bad JSON");
            let expected_obj =
                serde_json::from_str::<serde_json::Value>(&expected).expect("Bad JSON");
            assert_eq!(result_obj, expected_obj);
            unsafe {
                free_c_string(result_ptr);
            }
        }

        unsafe { query_state_free(query_state) }
        unsafe { machine_free(machine_ptr) };
    }
}
