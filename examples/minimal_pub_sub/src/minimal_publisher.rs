use anyhow::{Error, Result};
use std::env;

fn main() -> Result<(), Error> {
    let context = rclrs::Context::new(env::args())?;

    let node = rclrs::create_node(&context, "minimal_publisher")?;

    let publisher =
        node.create_publisher::<rclrs_example_msgs::msg::VariousTypes>("topic", rclrs::QOS_PROFILE_DEFAULT)?;

    let mut message = rclrs_example_msgs::msg::VariousTypes::default();

    let mut publish_count: u32 = 1;

    while context.ok() {
        println!("Publishing");
        publisher.publish(&message)?;
        publish_count += 1;
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    Ok(())
}
