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
                let mut vm = VM::new();
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
}
