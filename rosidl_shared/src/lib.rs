
pub trait MiddlewareMessage {
    /// Type name of the message in the ROS middleware
    /// 
    /// This usually follows a pattern "<package_name>/<subfolder>/<type_name>".
    const TYPE_NAME: &'static str;
}