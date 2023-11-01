[package]
name = "@(package_name)"
version = "@(package_version)"
edition = "2021"

[dependencies]
rosidl_runtime_rs = { version = "0.3", optional = true }
rosidl_shared = { version = "*" }
serde = { version = "1", optional = true, features = ["derive"] }
serde-big-array = { version = "0.5.1", optional = true }
@[for dep in dependency_packages]@
@(dep) = "*"
@[end for]@

[features]
default = ["with_middleware"]
@{
serde_features = ["dep:serde", "dep:serde-big-array", "rosidl_runtime_rs?/serde"]
for dep in dependency_packages:
	serde_features.append("{}/serde".format(dep))
}@
serde = @(serde_features)
# Enable to prevent linking against the ROS middleware
@{
with_middleware_features = ["dep:rosidl_runtime_rs"]
for dep in dependency_packages:
	with_middleware_features.append("{}/with_middleware".format(dep))
}@
with_middleware = @(with_middleware_features)
