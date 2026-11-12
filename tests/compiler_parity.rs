use abapc_studio::compiler::policy::LanguageMode;
use abapc_studio::compiler::{compile_source, CompileOptions};
use std::collections::HashMap;

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
