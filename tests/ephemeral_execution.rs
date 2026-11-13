use abapc_studio::api::handlers::AutoCleanFile;
use abapc_studio::compiler::policy::LanguageMode;
use abapc_studio::compiler::{compile_source, CompileOptions};
use abapc_studio::db::IpRateLimiter;
use std::fs;

#[test]
fn test_autoclean_file_deletion_on_drop() {
    let temp_path = std::env::temp_dir().join("abap_test_autoclean_drop.abap");
    fs::write(&temp_path, "REPORT ztest.\nWRITE 'AUTO_CLEAN'._").unwrap();
    assert!(temp_path.exists(), "Temp file must exist before drop");

    {
        let _guard = AutoCleanFile::new(temp_path.clone());
        assert!(temp_path.exists(), "Temp file must exist while guard is held");
    } // Guard drops here

    assert!(!temp_path.exists(), "Temp file must be unlinked the instant guard is dropped");
}

#[tokio::test]
async fn test_rate_limiter_allows_three_then_blocks_fourth() {
    let limiter = IpRateLimiter::new(None);
    let test_ip = "192.168.1.100";

    // 1st request -> allowed
    assert!(limiter.check_rate_limit(test_ip).await.is_ok());

    // 2nd request -> allowed
    assert!(limiter.check_rate_limit(test_ip).await.is_ok());

    // 3rd request -> allowed
    assert!(limiter.check_rate_limit(test_ip).await.is_ok());

    // 4th request -> blocked with retry_after > 0
    let fourth = limiter.check_rate_limit(test_ip).await;
    assert!(fourth.is_err());
    let retry_after = fourth.unwrap_err();
    assert!(retry_after > 0 && retry_after <= 60);

    // Another IP should not be affected
    let other_ip = "192.168.1.101";
    assert!(limiter.check_rate_limit(other_ip).await.is_ok());
}

#[test]
fn test_ephemeral_source_execution_with_oo_and_open_sql() {
    let source = r#"
REPORT z_ephemeral_test.

TYPES: BEGIN OF ty_flight,
         carrid TYPE string,
         connid TYPE i,
         price  TYPE i,
       END OF ty_flight.

DATA: lt_flights TYPE TABLE OF ty_flight,
      ls_flight  TYPE ty_flight.

SELECT carrid connid price
  FROM sflight
  WHERE price > 450
  INTO TABLE @lt_flights.

WRITE: / 'Total flights:', sy-dbcnt.
LOOP AT lt_flights INTO ls_flight.
  WRITE: / ls_flight-carrid, ls_flight-connid, ls_flight-price.
ENDLOOP.
"#;

    let mut options = CompileOptions::default();
    options.run = true;
    options.mode = Some(LanguageMode::Standard);

    let res = compile_source(source, options);
    assert!(res.ok, "Compilation failed: {:?}", res.diagnostics);
    let vm = res.vm.expect("VM must be present");
    assert!(vm.output.iter().any(|line| line.contains("Total flights: 2")));
    assert!(vm.output.iter().any(|line| line.contains("LH 400 520")));
    assert!(vm.output.iter().any(|line| line.contains("LH 401 610")));
}
