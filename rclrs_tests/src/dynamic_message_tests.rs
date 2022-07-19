use rclrs::dynamic_message::*;
use test_msgs::msg;

// #[test]
// fn conversion_roundtrip() {
// 	let msg =
// }

#[test]
fn max_alignment_is_8() {
	// The DynamicMessage type makes sure that its storage is aligned to 8
	let alignments = [
		std::mem::align_of::<msg::Builtins>(),
		std::mem::align_of::<msg::Arrays>(),
		std::mem::align_of::<msg::Empty>(),
		std::mem::align_of::<msg::Strings>(),
		std::mem::align_of::<msg::BoundedSequences>(),
		std::mem::align_of::<msg::Nested>(),
		std::mem::align_of::<msg::MultiNested>(),
		std::mem::align_of::<msg::UnboundedSequences>(),
		std::mem::align_of::<msg::WStrings>(),
		std::mem::align_of::<msg::Constants>(),
		std::mem::align_of::<msg::BasicTypes>(),
		std::mem::align_of::<msg::Defaults>(),
	];
	assert_eq!(alignments.into_iter().max().unwrap(), 8);
}

#[test]
fn message_structure_is_accurate() {
    let dyn_msg = DynamicMessage::new("test_msgs/msg/MultiNested").unwrap();
    assert_eq!(
        dyn_msg.structure().fields_inorder(),
        vec![
            "array_of_arrays",
            "array_of_bounded_sequences",
            "array_of_unbounded_sequences",
            "bounded_sequence_of_arrays",
            "bounded_sequence_of_bounded_sequences",
            "bounded_sequence_of_unbounded_sequences",
            "unbounded_sequence_of_arrays",
            "unbounded_sequence_of_bounded_sequences",
            "unbounded_sequence_of_unbounded_sequences"
        ].into_iter().map(String::from).collect::<Vec<_>>()
    );

    let arrays_of_arrays = dyn_msg.structure()["array_of_arrays"];

}

#[test]
fn dynamic_message_has_defaults() {
    let dyn_msg = DynamicMessage::new("test_msgs/msg/Defaults").unwrap();
    assert_eq!(
        dyn_msg.get("bool_value"),
        Some(Value::Simple(SimpleValue::Boolean(&true)))
    );
    assert_eq!(
        dyn_msg.get("byte_value"),
        Some(Value::Simple(SimpleValue::Octet(&50u8)))
    );
    // assert_eq!(dyn_msg.get("char_value"), Some(Value::Simple(SimpleValue::Char(&100u8))));
    assert_eq!(
        dyn_msg.get("float32_value"),
        Some(Value::Simple(SimpleValue::Float(&1.125f32)))
    );
    assert_eq!(
        dyn_msg.get("float64_value"),
        Some(Value::Simple(SimpleValue::Double(&1.125f64)))
    );
    assert_eq!(
        dyn_msg.get("int8_value"),
        Some(Value::Simple(SimpleValue::Int8(&-50i8)))
    );
    assert_eq!(
        dyn_msg.get("uint8_value"),
        Some(Value::Simple(SimpleValue::Uint8(&200u8)))
    );
    assert_eq!(
        dyn_msg.get("int16_value"),
        Some(Value::Simple(SimpleValue::Int16(&-1000i16)))
    );
    assert_eq!(
        dyn_msg.get("uint16_value"),
        Some(Value::Simple(SimpleValue::Uint16(&2000u16)))
    );
    assert_eq!(
        dyn_msg.get("int32_value"),
        Some(Value::Simple(SimpleValue::Int32(&-30000i32)))
    );
    assert_eq!(
        dyn_msg.get("uint32_value"),
        Some(Value::Simple(SimpleValue::Uint32(&60000u32)))
    );
    assert_eq!(
        dyn_msg.get("int64_value"),
        Some(Value::Simple(SimpleValue::Int64(&-40000000i64)))
    );
    assert_eq!(
        dyn_msg.get("uint64_value"),
        Some(Value::Simple(SimpleValue::Uint64(&50000000u64)))
    );

    let dyn_msg = DynamicMessage::new("test_msgs/msg/Arrays").unwrap();
    let dyn_msg = DynamicMessage::new("test_msgs/msg/UnboundedSequences").unwrap();
    let dyn_msg = DynamicMessage::new("test_msgs/msg/BoundedSequences").unwrap();
}

// #[test]
// fn test_mut_value_same_as_value() {

// }

// #[test]
// fn test_setting_value() {

// }
