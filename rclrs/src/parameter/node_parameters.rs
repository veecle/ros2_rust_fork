use std::collections::BTreeMap;

use super::{ParameterOverrideMap, ParameterValue};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Parameter {
    ReadOnly(ParameterValue),
    ReadWrite(ParameterValue),
    // Uninitialized? https://github.com/ros2/rclcpp/pull/1673
}




pub struct ConstrainedParameter {

}

pub enum ParameterError {
    TypeMismatch,
    InvalidName,
}

// RCLCPP_PUBLIC
// virtual
// const rclcpp::ParameterValue &
// declare_parameter(
// const std::string & name,
// const rclcpp::ParameterValue & default_value,
// const rcl_interfaces::msg::ParameterDescriptor & parameter_descriptor =
// rcl_interfaces::msg::ParameterDescriptor(),
// bool ignore_override = false) = 0;

It is a read-only parameter if 

// Parameter states:
// - Undeclared (although there might be a "override"/"initial" value for it )
// - 



// What happens when you declare a parameter again?
// => Exception

// What when a set_parameter service call arrives before a parameter is declared? => If allow_undeclared_parameters
// Do we want to replicate the behavior of get_parameter() to return an empty rclcpp::Parameter{name} if allow_undeclared?
// Would it be fair to say that the parameters = a map with defaults for some values (= overrides)?
// => How would the service handle this?


/// The declare() function is responsible for specifying the intended type of the parameter

/// The node can receive parameters from the "outside", i.e. given
/// directly on the command line or in yaml files, in the node builder,
/// or via parameter services.
/// These parameters can then be validated by the node.
pub struct NodeParameters {
    // This is entirely immutable.
    overrides: ParameterOverrideMap,
    parameters: BTreeMap<String, Parameter>
    allow_set_parameters_service_to_declare_parameters: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ParameterEntry<T> {
    InitialValue(T),
    ReadOnly(T),
    ReadWrite(T),
}

impl NodeParameters {
    pub fn new(overrides: ParameterOverrideMap) -> Self {
        Self {
            overrides,
            parameters: BTreeMap::new(),
        }
    }

    // pub fn get_unused_overrides()

    pub fn get(&self, name: &str) -> Option<&ParameterValue> {
        match self.parameters.get(name)? {
            Parameter::InitialValue(_) => None,
            Parameter::ReadOnly(pv) => Some(pv),
            Parameter::ReadWrite(pv) => Some(pv),
        }
    }

    pub fn entry<T>(&self, name: &str) -> Option<ParameterEntry<T>> {
        let param = self.parameters.get_mut(name)?;

    }

    // declare = entry::<String>("foo")?;
}
