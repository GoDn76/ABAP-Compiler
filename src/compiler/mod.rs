pub mod ast;
pub mod class_model;
pub mod diagnostics;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod policy;
pub mod semantic;
pub mod tokens;
pub mod vm;

use ast::TypeRef;
use diagnostics::{Diagnostic, DiagnosticBag};
use ir::{ProgramIR, RequestedEvent};
use lexer::Lexer;
use parser::Parser;
use policy::{validate_policy, CompilerPolicy, LanguageMode, ReleasedApiManifest};
use semantic::SemanticAnalyzer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use vm::{VMResult, VM};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileStats {
    pub tokens: usize,
    pub diagnostics: usize,
    pub errors: usize,
    pub warnings: usize,
    #[serde(rename = "elapsedMs")]
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    pub ok: bool,
    pub diagnostics: Vec<Diagnostic>,
    #[serde(rename = "diagnosticText", default)]
    pub diagnostic_text: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ir: Option<ProgramIR>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vm: Option<VMResult>,
    pub stats: CompileStats,
    #[serde(rename = "manifestCounts", default)]
    pub manifest_counts: HashMap<String, usize>,
}

#[derive(Debug, Clone, Default)]
pub struct CompileOptions {
    pub mode: Option<LanguageMode>,
    pub released_apis: Option<ReleasedApiManifest>,
    pub run: bool,
    pub max_steps: usize,
    pub parameter_values: HashMap<String, serde_json::Value>,
    pub preview_selection: bool,
    pub selection_event: Option<RequestedEvent>,
    pub external_types: HashMap<String, TypeRef>,
    pub filename: Option<String>,
    pub database: Option<HashMap<String, Vec<serde_json::Value>>>,
}

fn extract_program_stem(filename: &str) -> String {
    let path = std::path::Path::new(filename);
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or(filename);
    let without_abap = if let Some(s) = file_name.strip_suffix(".abap") {
        s
    } else if let Some(s) = file_name.strip_suffix(".prog.abap") {
        s
    } else {
        file_name
    };
    without_abap.to_string()
}

fn is_ephemeral_or_generic(stem: &str) -> bool {
    let lower = stem.to_ascii_lowercase();
    lower.starts_with("abap_ephemeral_")
        || lower.ends_with("_standard_program")
        || lower == "standard_program"
        || lower == "temp"
        || lower == "program"
        || lower == "z_db_ping"
}

pub fn compile_source(source: &str, options: CompileOptions) -> CompileResult {
    let started = Instant::now();
    let mut diagnostics = DiagnosticBag::new();

    let (tokens, lex_diags) = Lexer::new(source).tokenize();
    diagnostics.extend(lex_diags);

    let token_count = tokens.len();

    let mode = options.mode.unwrap_or(LanguageMode::Standard);
    let mode_str = match mode {
        LanguageMode::Standard => "standard",
        LanguageMode::Cloud => "cloud",
    };

    let manifest = options.released_apis.clone().unwrap_or_default();
    let mut manifest_counts = HashMap::new();
    manifest_counts.insert("functions".to_string(), manifest.functions.len());
    manifest_counts.insert("classes".to_string(), manifest.classes.len());
    manifest_counts.insert("interfaces".to_string(), manifest.interfaces.len());
    manifest_counts.insert("tables".to_string(), manifest.tables.len());

    if !diagnostics.has_errors() {
        let (program, parse_diags) = Parser::new(tokens).parse();
        diagnostics.extend(parse_diags);

        if !diagnostics.has_errors() {
            // Validate Report Name vs Program/File Name (Standard ABAP rule)
            if let Some(ref rep_name) = program.report_name {
                if let Some(ref fname) = options.filename {
                    let clean_stem = extract_program_stem(fname);
                    if !is_ephemeral_or_generic(&clean_stem) {
                        if rep_name.to_ascii_uppercase() != clean_stem.to_ascii_uppercase() {
                            let span = program.report_span.unwrap_or((
                                diagnostics::Position { line: 1, column: 1, offset: 0 },
                                diagnostics::Position { line: 1, column: 1, offset: 0 },
                            ));
                            diagnostics.error(
                                "REP001",
                                format!(
                                    "Report name '{}' does not match program name '{}'.",
                                    rep_name.to_ascii_uppercase(),
                                    clean_stem.to_ascii_uppercase()
                                ),
                                span.0,
                                span.1,
                            );
                        }
                    }
                }
            }

            let policy = CompilerPolicy {
                mode,
                released_apis: manifest,
                strict_cloud_syntax: true,
            };
            validate_policy(&program, &policy, &mut diagnostics);

            let sem_diags = SemanticAnalyzer::new().analyze(&program, options.external_types.clone());
            diagnostics.extend(sem_diags);

            if !diagnostics.has_errors() {
                let ir = ir::Lowerer::new().lower(
                    &program,
                    mode_str,
                    options.selection_event.clone(),
                    options.external_types,
                );

                if !options.run && !options.preview_selection {
                    let errs = diagnostics.items.iter().filter(|d| d.severity == diagnostics::Severity::Error).count();
                    let warns = diagnostics.items.iter().filter(|d| d.severity == diagnostics::Severity::Warning).count();
                    let d_text = diagnostics.items.iter().map(diagnostics::format_diagnostic).collect();

                    return CompileResult {
                        ok: errs == 0,
                        diagnostics: diagnostics.items.clone(),
                        diagnostic_text: d_text,
                        ir: Some(ir),
                        vm: None,
                        stats: CompileStats {
                            tokens: token_count,
                            diagnostics: diagnostics.items.len(),
                            errors: errs,
                            warnings: warns,
                            elapsed_ms: started.elapsed().as_millis() as u64,
                        },
                        manifest_counts,
                    };
                }

                let max_steps = if options.max_steps > 0 { options.max_steps } else { 2_000_000 };
                let mut vm = if let Some(custom_db) = options.database {
                    VM::with_database(custom_db)
                } else {
                    VM::new()
                };
                let vm_res = vm.execute(
                    &ir,
                    max_steps,
                    &options.parameter_values,
                    options.preview_selection,
                    options.selection_event.is_some(),
                    None,
                    None,
                );

                match vm_res {
                    Ok(res) => {
                        let errs = diagnostics.items.iter().filter(|d| d.severity == diagnostics::Severity::Error).count();
                        let warns = diagnostics.items.iter().filter(|d| d.severity == diagnostics::Severity::Warning).count();
                        let d_text = diagnostics.items.iter().map(diagnostics::format_diagnostic).collect();

                        return CompileResult {
                            ok: errs == 0,
                            diagnostics: diagnostics.items.clone(),
                            diagnostic_text: d_text,
                            ir: Some(ir),
                            vm: Some(res),
                            stats: CompileStats {
                                tokens: token_count,
                                diagnostics: diagnostics.items.len(),
                                errors: errs,
                                warnings: warns,
                                elapsed_ms: started.elapsed().as_millis() as u64,
                            },
                            manifest_counts,
                        };
                    }
                    Err(e) => {
                        diagnostics.error("RUN001", e, program.start, program.end);
                        let errs = diagnostics.items.iter().filter(|d| d.severity == diagnostics::Severity::Error).count();
                        let warns = diagnostics.items.iter().filter(|d| d.severity == diagnostics::Severity::Warning).count();
                        let d_text = diagnostics.items.iter().map(diagnostics::format_diagnostic).collect();

                        return CompileResult {
                            ok: false,
                            diagnostics: diagnostics.items.clone(),
                            diagnostic_text: d_text,
                            ir: Some(ir),
                            vm: None,
                            stats: CompileStats {
                                tokens: token_count,
                                diagnostics: diagnostics.items.len(),
                                errors: errs,
                                warnings: warns,
                                elapsed_ms: started.elapsed().as_millis() as u64,
                            },
                            manifest_counts,
                        };
                    }
                }
            }
        }
    }

    let errs = diagnostics.items.iter().filter(|d| d.severity == diagnostics::Severity::Error).count();
    let warns = diagnostics.items.iter().filter(|d| d.severity == diagnostics::Severity::Warning).count();
    let diag_count = diagnostics.items.len();
    let d_text = diagnostics.items.iter().map(diagnostics::format_diagnostic).collect();

    CompileResult {
        ok: false,
        diagnostics: diagnostics.items,
        diagnostic_text: d_text,
        ir: None,
        vm: None,
        stats: CompileStats {
            tokens: token_count,
            diagnostics: diag_count,
            errors: errs,
            warnings: warns,
            elapsed_ms: started.elapsed().as_millis() as u64,
        },
        manifest_counts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_unchain_write_colon() {
        let source = r#"
REPORT zunchain.
DATA: lv_a TYPE i VALUE 10, lv_b TYPE i VALUE 20.
WRITE: / lv_a, / lv_b.
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "Unchain test failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(vm.output, vec!["10", "20"]);
    }

    #[test]
    fn test_read_table_sy_subrc() {
        let source = r#"
REPORT zread_tab.
DATA lt_items TYPE TABLE OF string.
APPEND 'ALPHA' TO lt_items.
APPEND 'BETA' TO lt_items.
READ TABLE lt_items INDEX 1 INTO DATA(lv_item).
WRITE: / sy-subrc.
READ TABLE lt_items INDEX 99 INTO DATA(lv_miss).
WRITE: / sy-subrc.
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "Read table sy-subrc test failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(vm.output, vec!["0", "4"]);
    }

    #[test]
    fn test_switch_constructor_expression() {
        let source = r#"
REPORT zswitch_test.
DATA: lv_status TYPE string VALUE 'E'.
DATA(lv_message) = SWITCH string( lv_status WHEN 'S' THEN 'Success' WHEN 'W' THEN 'Warning' WHEN 'E' THEN 'Error' ELSE 'Unknown' ).
WRITE: / 'Status Message:', lv_message.
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "Compilation failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(vm.output, vec!["Status Message: Error"]);
        assert_eq!(
            vm.variables.get("LV_MESSAGE").and_then(|v| v.as_str()),
            Some("Error")
        );
    }

    #[test]
    fn test_parameter_injection() {
        let source = r#"
REPORT zparam_test.
PARAMETERS p_total TYPE i.
WRITE: p_total.
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Standard);
        let mut params = HashMap::new();
        params.insert("P_TOTAL".to_string(), serde_json::json!("42"));
        options.parameter_values = params;

        let res = compile_source(source, options);
        assert!(res.ok, "Parameter test failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(vm.output, vec!["42"]);
    }

    #[test]
    fn test_cloud_classrun_out_write() {
        let source = r#"
CLASS zcl_cloud_test DEFINITION PUBLIC FINAL CREATE PUBLIC.
  PUBLIC SECTION.
    INTERFACES if_oo_adt_classrun.
ENDCLASS.
CLASS zcl_cloud_test IMPLEMENTATION.
  METHOD if_oo_adt_classrun~main.
    DATA lv_res TYPE i.
    lv_res = 12 + 2.
    out->write( lv_res ).
  ENDMETHOD.
ENDCLASS.
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Cloud);

        let res = compile_source(source, options);
        assert!(res.ok, "Cloud classrun failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(vm.output, vec!["14"]);
    }

    #[test]
    fn test_oo_calculator_methods() {
        let source = r#"
REPORT zcalc_test.
CLASS lcl_calculator DEFINITION.
  PUBLIC SECTION.
    METHODS add_numbers IMPORTING iv_a TYPE i iv_b TYPE i RETURNING VALUE(rv_res) TYPE i.
ENDCLASS.
CLASS lcl_calculator IMPLEMENTATION.
  METHOD add_numbers.
    rv_res = iv_a + iv_b.
  ENDMETHOD.
ENDCLASS.
START-OF-SELECTION.
  DATA(lo_calc) = NEW lcl_calculator( ).
  DATA(lv_sum) = lo_calc->add_numbers( iv_a = 25 iv_b = 75 ).
  WRITE: / 'Class Method Output:', lv_sum.
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "OO calculator failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(vm.output, vec!["Class Method Output: 100"]);
        assert_eq!(
            vm.variables.get("LV_SUM").and_then(|v| v.as_i64()),
            Some(100)
        );
    }

    #[test]
    fn test_select_inline_data() {
        let source = r#"
REPORT zsel_test.
SELECT carrid, connid, fldate, price FROM sflight INTO TABLE @DATA(lt_live_flights).
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "Select failed: {:?}", res.diagnostics);
    }

    #[test]
    fn test_classic_strings_translate_and_split() {
        let source = r#"
REPORT zstr_test.
DATA text TYPE string.
DATA parts TYPE TABLE OF string.
text = 'hello,cloud'.
TRANSLATE text TO UPPER CASE.
SPLIT text AT ',' INTO TABLE parts.
LOOP AT parts INTO DATA(part).
  WRITE part.
ENDLOOP.
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "Strings test failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(vm.output, vec!["HELLO", "CLOUD"]);
    }

    #[test]
    fn test_types_begin_of_struct_and_loop_sy_tabix() {
        let source = r#"
REPORT zstruct_loop_test.
TYPES: BEGIN OF ty_flight,
         carrid TYPE c,
         connid TYPE n,
       END OF ty_flight.
DATA lt_flights TYPE TABLE OF ty_flight.
DATA ls_row TYPE ty_flight.
ls_row-carrid = 'LH'.
ls_row-connid = 400.
APPEND ls_row TO lt_flights.
ls_row-carrid = 'SQ'.
ls_row-connid = 88.
APPEND ls_row TO lt_flights.

LOOP AT lt_flights INTO DATA(ls_cur).
  WRITE: / 'Index:', sy-tabix, ls_cur-carrid.
ENDLOOP.
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "Types struct and loop test failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(vm.output, vec!["Index: 1 LH", "Index: 2 SQ"]);
    }

    #[test]
    fn test_string_template_interpolation_with_lines_and_select() {
        let source = r#"
REPORT zflight_report.
TYPES: BEGIN OF ty_flight,
         carrid TYPE c,
         connid TYPE n,
         fldate TYPE d,
         price  TYPE p,
       END OF ty_flight.

DATA lt_live_flights TYPE TABLE OF ty_flight.

SELECT carrid, connid, fldate, price FROM sflight INTO TABLE @DATA(lt_live_flights).

WRITE |Successfully fetched { lines( lt_live_flights ) } rows|.

LOOP AT lt_live_flights INTO DATA(ls_flight).
  WRITE: / 'Flight:', ls_flight-carrid, ls_flight-connid, 'Tabix:', sy-tabix.
ENDLOOP.
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "Flight report test failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(vm.output[0], "Successfully fetched 3 rows");
        assert_eq!(vm.output[1], "Flight: LH 400 Tabix: 1");
        assert_eq!(vm.output[2], "Flight: LH 401 Tabix: 2");
        assert_eq!(vm.output[3], "Flight: SQ 88 Tabix: 3");
    }

    #[test]
    fn test_open_sql_select_where_clause_with_inline() {
        let source = r#"
REPORT zsql_where.
SELECT * FROM sflight INTO TABLE @DATA(lt_lh) WHERE carrid = 'LH'.
WRITE |LH flights count: { lines( lt_lh ) }|.
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "Select where test failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(vm.output, vec!["LH flights count: 2"]);
    }

    #[test]
    fn test_hyphen_identifiers_string_bleed_and_write_unchain() {
        let source = r#"
REPORT zaudit_test.
DATA: BEGIN OF ls_audit,
        step_id TYPE string,
      END OF ls_audit.
ls_audit-step_id = 'S1'.
WRITE: / 'Step ->', ls_audit-step_id.
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "Audit test failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(vm.output, vec!["Step -> S1"]);
    }

    #[test]
    fn test_assignment_string_template_and_flexible_write() {
        let source = r#"
REPORT zaudit_pass.
DATA: BEGIN OF ls_audit,
        step_id TYPE string,
        result TYPE string,
      END OF ls_audit.
ls_audit-step_id = 'S1'.
ls_audit-result = |PASSED at { sy-tabix }|.
WRITE / 'Step'.
WRITE ls_audit-step_id.
WRITE / ls_audit-result.
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "Audit pass test failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(vm.output, vec!["Step S1", "PASSED at 0"]);
    }

    #[test]
    fn test_write_statement_flexible_multiple_items_and_slashes() {
        let source = r#"
REPORT zwrite_flex.
DATA: lv_a TYPE string VALUE 'Alpha',
      lv_b TYPE string VALUE 'Beta'.
WRITE / 'Item 1' lv_a.
WRITE: / 'Item 2', lv_b.
WRITE / 'Line 3' / 'Line 4'.
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "Write flex test failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(vm.output, vec!["Item 1 Alpha", "Item 2 Beta", "Line 3", "Line 4"]);
    }

    #[test]
    fn test_crazy_oo_pointer_test() {
        let source = r#"
REPORT z_crazy_oo_pointer_test.

CLASS lcl_math DEFINITION.
  PUBLIC SECTION.
    CLASS-METHODS: divide IMPORTING iv_num1 TYPE i iv_num2 TYPE i
                          RETURNING VALUE(rv_result) TYPE i.
ENDCLASS.

CLASS lcl_math IMPLEMENTATION.
  METHOD divide.
    TRY.
        rv_result = iv_num1 / iv_num2.
      CATCH cx_sy_zerodivide.
        rv_result = -1.
    ENDTRY.
  ENDMETHOD.
ENDCLASS.

DATA: lv_dynamic_var TYPE string VALUE 'RUST_IS_FAST'.
FIELD-SYMBOLS: <fs_value> TYPE any.

ASSIGN lv_dynamic_var TO <fs_value>.
IF <fs_value> IS ASSIGNED.
  <fs_value> = |{ <fs_value> }_AND_SAFE|. 
ENDIF.

TYPES: BEGIN OF ty_player,
         id    TYPE i,
         name  TYPE string,
         score TYPE i,
       END OF ty_player.

DATA: lt_players TYPE TABLE OF ty_player,
      ls_player  TYPE ty_player.

ls_player-id = 1. ls_player-name = 'Gaurav'. ls_player-score = 150. APPEND ls_player TO lt_players.
ls_player-id = 2. ls_player-name = 'RustVM'. ls_player-score = 999. APPEND ls_player TO lt_players.
ls_player-id = 3. ls_player-name = 'BugHunter'. ls_player-score = 50. APPEND ls_player TO lt_players.

FIELD-SYMBOLS: <fs_player> TYPE ty_player.

READ TABLE lt_players ASSIGNING <fs_player> WITH KEY name = 'RustVM'.
IF <fs_player> IS ASSIGNED.
  <fs_player>-score = 0.
ENDIF.

DATA: lv_total_score TYPE i VALUE 0.

LOOP AT lt_players INTO ls_player WHERE score > 0.
  lv_total_score = lv_total_score + ls_player-score.
  WRITE: / 'Player:', ls_player-name, 'Score:', ls_player-score.
ENDLOOP.

DATA(lv_div_valid) = lcl_math=>divide( iv_num1 = 10 iv_num2 = 2 ).
DATA(lv_div_zero)  = lcl_math=>divide( iv_num1 = 10 iv_num2 = 0 ).

WRITE: / '--- TEST RESULTS ---'.
WRITE: / 'Dynamic Pointer Result:', lv_dynamic_var.
WRITE: / 'Total Score (Expected 200):', lv_total_score.
WRITE: / 'Valid Division (Expected 5):', lv_div_valid.
WRITE: / 'Zero Division Handled (Expected -1):', lv_div_zero.
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "Crazy OO pointer test compilation failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(
            vm.output,
            vec![
                "Player: Gaurav Score: 150",
                "Player: BugHunter Score: 50",
                "--- TEST RESULTS ---",
                "Dynamic Pointer Result: RUST_IS_FAST_AND_SAFE",
                "Total Score (Expected 200): 200",
                "Valid Division (Expected 5): 5",
                "Zero Division Handled (Expected -1): -1",
            ]
        );
    }

    #[test]
    fn test_report_name_mismatch_error() {
        let source = "REPORT z_wrong_name.\nWRITE: / 'Test'.";
        let mut options = CompileOptions::default();
        options.filename = Some("z_stateless_main.abap".to_string());
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(!res.ok, "Should fail due to report name mismatch");
        assert!(res.diagnostics.iter().any(|d| d.code == "REP001"));
        let diag = res.diagnostics.iter().find(|d| d.code == "REP001").unwrap();
        assert!(diag.message.contains("Z_WRONG_NAME"));
        assert!(diag.message.contains("Z_STATELESS_MAIN"));
    }

    #[test]
    fn test_report_name_match_success() {
        let source = "REPORT z_stateless_main.\nWRITE: / 'Test'.";
        let mut options = CompileOptions::default();
        options.filename = Some("z_stateless_main.abap".to_string());
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "Should succeed when report name matches program filename");
        assert!(!res.diagnostics.iter().any(|d| d.code == "REP001"));
    }

    #[test]
    fn test_value_constructor_expression() {
        let source = r#"
REPORT z_value_test.
TYPES: BEGIN OF ty_item,
         id   TYPE i,
         name TYPE string,
       END OF ty_item.
DATA: ls_row TYPE ty_item.
ls_row = VALUE #( id = 42 name = 'Rust' ).
WRITE: / 'ID:', ls_row-id, 'Name:', ls_row-name.
"#;
        let mut options = CompileOptions::default();
        options.filename = Some("z_value_test.abap".to_string());
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "VALUE constructor test failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(vm.output, vec!["ID: 42 Name: Rust"]);
    }

    #[test]
    fn test_stateless_architecture_test_full() {
        let source = r#"
REPORT z_stateless_architecture_test.

WRITE: / '--- PHASE 1: BYODB SQL INTERCEPTOR ---'.

SELECT client, uname 
  FROM usr02 
  INTO TABLE @DATA(lt_db_users) 
  UP TO 5 ROWS.

IF sy-subrc = 0.
  WRITE: / |DB SUCCESS! Fetched records via Postgres/Mock Interceptor.|.
ELSE.
  WRITE: / |DB WARNING: Query executed but no records found (sy-subrc = 4).|.
ENDIF.

WRITE: / '--- PHASE 2: MEMORY POINTER MUTATION ---'.

TYPES: BEGIN OF ty_memory,
         address TYPE string,
         value   TYPE i,
       END OF ty_memory.

DATA: lt_heap TYPE TABLE OF ty_memory,
      ls_heap TYPE ty_memory.

ls_heap-address = '0x001'. ls_heap-value = 100. APPEND ls_heap TO lt_heap.
ls_heap-address = '0x002'. ls_heap-value = 200. APPEND ls_heap TO lt_heap.
ls_heap-address = '0x003'. ls_heap-value = 300. APPEND ls_heap TO lt_heap.

LOOP AT lt_heap ASSIGNING FIELD-SYMBOL(<fs_node>) WHERE value > 150.
  <fs_node>-value = <fs_node>-value * 2. 
ENDLOOP.

WRITE: / '--- PHASE 3: EXCEPTION STACK ---'.

CLASS lcl_memory_manager DEFINITION.
  PUBLIC SECTION.
    CLASS-METHODS read_memory IMPORTING iv_addr TYPE string
                              EXPORTING ev_val TYPE i.
ENDCLASS.

CLASS lcl_memory_manager IMPLEMENTATION.
  METHOD read_memory.
    READ TABLE lt_heap ASSIGNING FIELD-SYMBOL(<fs_read>) WITH KEY address = iv_addr.
    IF sy-subrc <> 0.
      RAISE EXCEPTION TYPE cx_sy_itab_line_not_found.
    ENDIF.
    ev_val = <fs_read>-value.
  ENDMETHOD.
ENDCLASS.

DATA: lv_val TYPE i.

TRY.
    lcl_memory_manager=>read_memory( EXPORTING iv_addr = '0x002' IMPORTING ev_val = lv_val ).
    WRITE: / |Pointer 0x002 Value (Should be 400): { lv_val }|.
    
    lcl_memory_manager=>read_memory( EXPORTING iv_addr = '0x999' IMPORTING ev_val = lv_val ).
    
    WRITE: / 'FAIL: The VM ignored the exception! This line should not print.'.
    
  CATCH cx_sy_itab_line_not_found.
    WRITE: / 'SUCCESS: Exception caught! Memory address 0x999 not found.'.
ENDTRY.

WRITE: / '--- VM EXECUTION COMPLETE ---'.
"#;
        let mut options = CompileOptions::default();
        options.filename = Some("z_stateless_architecture_test.abap".to_string());
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "Stateless architecture test failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        assert_eq!(
            vm.output,
            vec![
                "--- PHASE 1: BYODB SQL INTERCEPTOR ---",
                "DB SUCCESS! Fetched records via Postgres/Mock Interceptor.",
                "--- PHASE 2: MEMORY POINTER MUTATION ---",
                "--- PHASE 3: EXCEPTION STACK ---",
                "Pointer 0x002 Value (Should be 400): 400",
                "SUCCESS: Exception caught! Memory address 0x999 not found.",
                "--- VM EXECUTION COMPLETE ---",
            ]
        );
    }

    #[test]
    fn test_memory_hump_stress_test() {
        let source = r#"
REPORT z_memory_hump_stress_test.

WRITE: / '--- INITIALIZING MEMORY HUMP TEST ---'.

TYPES: BEGIN OF ty_node,
         id      TYPE i,
         counter TYPE i,
         status  TYPE string,
       END OF ty_node.

DATA: lt_data  TYPE TABLE OF ty_node,
      ls_data  TYPE ty_node,
      lv_index TYPE i VALUE 1.

WHILE lv_index <= 500.
  ls_data-id      = lv_index.
  ls_data-counter = 0.
  ls_data-status  = 'INIT'.
  APPEND ls_data TO lt_data.
  lv_index = lv_index + 1.
ENDWHILE.

WRITE: / 'Memory allocated. Starting pointer alias mutation...'.

LOOP AT lt_data ASSIGNING FIELD-SYMBOL(<fs_node>).
  <fs_node>-counter = <fs_node>-id * 10.
  
  IF <fs_node>-counter > 2500.
    <fs_node>-status = 'HIGH_LOAD'.
  ELSE.
    <fs_node>-status = 'SAFE_LOAD'.
  ENDIF.
ENDLOOP.

READ TABLE lt_data ASSIGNING FIELD-SYMBOL(<fs_target>) WITH KEY id = 404.
IF sy-subrc = 0.
  <fs_target>-status = 'CORRUPTED_BY_POINTER'.
ENDIF.

DATA: lv_math_res TYPE i,
      lv_zero     TYPE i VALUE 0.
TRY.
    lv_math_res = 100 / lv_zero.
  CATCH cx_sy_zerodivide.
    lv_math_res = -999.
ENDTRY.

WRITE: / '--- VALIDATION RESULTS ---'.

READ TABLE lt_data INTO ls_data WITH KEY id = 500.
WRITE: / |Record 500 Status (Expected: HIGH_LOAD): { ls_data-status }|.
WRITE: / |Record 500 Counter (Expected: 5000): { ls_data-counter }|.

READ TABLE lt_data INTO ls_data WITH KEY id = 100.
WRITE: / |Record 100 Status (Expected: SAFE_LOAD): { ls_data-status }|.
WRITE: / |Record 100 Counter (Expected: 1000): { ls_data-counter }|.

READ TABLE lt_data INTO ls_data WITH KEY id = 404.
WRITE: / |Record 404 Status (Expected: CORRUPTED_BY_POINTER): { ls_data-status }|.

WRITE: / |Exception Engine Check (Expected: -999): { lv_math_res }|.
"#;
        let mut options = CompileOptions::default();
        options.filename = Some("z_memory_hump_stress_test.abap".to_string());
        options.run = true;
        options.mode = Some(LanguageMode::Standard);

        let res = compile_source(source, options);
        assert!(res.ok, "Memory hump test compile failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM result should be present");
        for line in &vm.output {
            println!("{}", line);
        }
        assert_eq!(
            vm.output,
            vec![
                "--- INITIALIZING MEMORY HUMP TEST ---",
                "Memory allocated. Starting pointer alias mutation...",
                "--- VALIDATION RESULTS ---",
                "Record 500 Status (Expected: HIGH_LOAD): HIGH_LOAD",
                "Record 500 Counter (Expected: 5000): 5000",
                "Record 100 Status (Expected: SAFE_LOAD): SAFE_LOAD",
                "Record 100 Counter (Expected: 1000): 1000",
                "Record 404 Status (Expected: CORRUPTED_BY_POINTER): CORRUPTED_BY_POINTER",
                "Exception Engine Check (Expected: -999): -999",
            ]
        );
    }

    #[test]
    fn test_oo_abap_test() {
        let source = r#"
REPORT z_oo_abap_test.

WRITE: / '--- INITIALIZING OO ABAP TEST ---'.

INTERFACE lif_engine.
  METHODS: start_engine RETURNING VALUE(rv_status) TYPE string.
ENDINTERFACE.

CLASS lcl_vehicle DEFINITION.
  PUBLIC SECTION.
    INTERFACES lif_engine.
    CLASS-METHODS: get_total_vehicles RETURNING VALUE(rv_count) TYPE i.
    METHODS: constructor IMPORTING iv_make TYPE string,
             display_info.
  PRIVATE SECTION.
    CLASS-DATA: gv_total_vehicles TYPE i VALUE 0.
    DATA: mv_make TYPE string.
ENDCLASS.

CLASS lcl_vehicle IMPLEMENTATION.
  METHOD constructor.
    mv_make = iv_make.
    gv_total_vehicles = gv_total_vehicles + 1.
  ENDMETHOD.

  METHOD lif_engine~start_engine.
    rv_status = |The engine of the { mv_make } is roaring!|.
  ENDMETHOD.

  METHOD display_info.
    WRITE: / |Vehicle Make: { mv_make }|.
  ENDMETHOD.

  METHOD get_total_vehicles.
    rv_count = gv_total_vehicles.
  ENDMETHOD.
ENDCLASS.

WRITE: / '--- VALIDATION RESULTS ---'.

DATA: lo_car   TYPE REF TO lcl_vehicle,
      lo_truck TYPE REF TO lcl_vehicle,
      lv_msg   TYPE string.

CREATE OBJECT lo_car EXPORTING iv_make = 'Porsche 911'.
CREATE OBJECT lo_truck EXPORTING iv_make = 'Volvo F16'.

lo_car->display_info( ).
lo_truck->display_info( ).

lv_msg = lo_car->lif_engine~start_engine( ).
WRITE: / |Interface Output: { lv_msg }|.

DATA(lv_total) = lcl_vehicle=>get_total_vehicles( ).
WRITE: / |Total Vehicles in Memory (Expected 2): { lv_total }|.
"#;
        let mut options = CompileOptions::default();
        options.run = true;
        options.mode = Some(LanguageMode::Standard);
        let res = compile_source(source, options);
        assert!(res.ok, "OO ABAP test failed: {:?}", res.diagnostics);
        let vm = res.vm.expect("VM should run");
        for line in &vm.output {
            println!("{}", line);
        }
    }
}
