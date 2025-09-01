// use kodo_core::Activity;
// use serde_json;

// #[test]
// fn activity_json_roundtrip() {
//     let a = Activity::new("coding", 30);

//     let json = serde_json::to_string(&a).unwrap();
//     println!("JSON: {}", json);

//     let back: Activity = serde_json::from_str(&json).unwrap();
//     assert_eq!(back.name(), "coding");
//     assert_eq!(back.duration_minutes(), 30);
// }