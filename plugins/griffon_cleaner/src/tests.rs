use super::*;
use serde_json::json;

#[test]
#[ignore = "requires mock filesystem"]
fn test_list_candidates_routing() {
    let router = build_router();

    // Simulate a request from the GUI using call_json
    let payload = json!({
        "dry_run": true,
        "file_types": ["system", "user"]
    });

    // Call the router directly
    let result = router.call_json("list_candidates", payload);

    // Verify it routed correctly and executed the business logic
    assert!(result.is_ok());

    let json_val = result.unwrap();
    assert_eq!(json_val["ok"], true);
    assert!(json_val.get("items").is_some());
}

#[test]
fn test_unknown_method() {
    let router = build_router();

    let err = router.call_json("does_not_exist", json!({})).unwrap_err();
    assert_eq!(err.code, -32601); // Method not found
}
