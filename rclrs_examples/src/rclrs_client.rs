use rclrs;
use std_msgs;
use example_interfaces;

fn main() -> rclrs::RclResult {
    let context = rclrs::Context::default();

    let mut node = context.create_node("minimal_client")?;

    let client = node.create_client::<example_interfaces::srv::AddTwoInts, _>("add_two_ints")?;

    let mut request = AddTwoInts::Request::default();
    request.a = 41;
    request.b = 1;

    while context.ok() {
        let result_future = client.async_send_request(&request)?;
    }

    rclrs::spin_until_future_complete(&node, &result_future)?;

    let result = result_future.get();
    println!("Publishing: [{}]", message.data);
    println!("result of {} + {} = {}", request.a, request.b, result.sum);

    rclrs::shutdown();
}
