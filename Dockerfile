ARG ROS_DISTRO=humble
FROM ros:$ROS_DISTRO as base
ARG DEBIAN_FRONTEND=noninteractive

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    libclang-dev \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install Rust and the cargo-ament-build plugin
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal --default-toolchain 1.74.0 -y --component rustfmt
ENV PATH=/root/.cargo/bin:$PATH
RUN cargo install cargo-ament-build

# Install the colcon-cargo and colcon-ros-cargo plugins
RUN pip install git+https://github.com/colcon/colcon-cargo.git git+https://github.com/colcon/colcon-ros-cargo.git

RUN mkdir -p /workspace && echo "Did you forget to mount the repository into the Docker container?" > /workspace/HELLO.txt
RUN mkdir /r2r
COPY . /r2r/ros2_rust
RUN cd /r2r && vcs import . < ros2_rust/ros2_rust_humble.repos

RUN . /opt/ros/humble/setup.sh \
  && cd /r2r \
  && colcon build --packages-skip-regex 'rclrs_tests' 'rclrs_example_msgs' 'example.*' 'test.*' \
  && rm -r /r2r/build /r2r/log

RUN mv /r2r/ros2_rust/docker_entry.sh /
ENTRYPOINT ["/bin/bash", "/docker_entry.sh"]
WORKDIR /workspace
