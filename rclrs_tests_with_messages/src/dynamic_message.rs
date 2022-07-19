use rclrs::{DynamicMessage, DynamicMessageError, Field};

#[test]
fn it_works() -> Result<(), DynamicMessageError> {
    let floats = rclrs_example_msgs::msg::rmw::ThreeFloats::default();
    let arr = rclrs_example_msgs::msg::rmw::Array::default();
    assert_eq!(std::mem::size_of::<rclrs_example_msgs::msg::rmw::ThreeFloats>(), 24);
    assert_eq!(std::mem::size_of::<rclrs_example_msgs::msg::rmw::Array>(), 24*5);
    let floats_dyn = DynamicMessage::try_from(floats)?;
    let arr_dyn = DynamicMessage::try_from(arr)?;
    assert_eq!(floats_dyn.structure().size, 24);
    assert_eq!(arr_dyn.structure().size, 24*5);
    Ok(())
}

#[test]
fn various_types() -> Result<(), DynamicMessageError> {
    use rclrs_example_msgs::msg::rmw::VariousTypes as VariousTypesMsg;
    let vt = VariousTypesMsg::default();
    let mut dyn_msg = DynamicMessage::try_from(vt)?;
    assert_eq!(dyn_msg.structure().size, std::mem::size_of::<VariousTypesMsg>());
    if let Some(Field::Boolean(bool_val)) = dyn_msg.field("bool_member") {
        assert_eq!(*bool_val, true);
    } else {
        panic!("Oh no!")
    }

    Ok(())
}
