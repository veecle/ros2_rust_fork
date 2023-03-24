use std::collections::BTreeMap;

use super::{ParameterOverrideMap, ParameterValue};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Parameter {
    ReadOnly(ParameterValue),
    ReadWrite(ParameterValue),
}

pub enum ParameterError {

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


// What when a set_parameter service call arrives before a parameter is declared? => If allow_undeclared_parameters
// Do we want to replicate the behavior of get_parameter() to return an empty rclcpp::Parameter{name} if allow_undeclared?
// Would it be fair to say that the parameters = a map with defaults for some values (= overrides)?
// => How would the service handle this?

pub struct NodeParameters {
    overrides: ParameterOverrideMap,
    parameters: BTreeMap<String, Parameter>,
    allow_undeclared: bool,
}

impl NodeParameters {
    pub fn new(overrides: ParameterOverrideMap) -> Self {
        Self {
            overrides,
            parameters: BTreeMap::new(),
        }
    }

    pub fn ro_parameter(&self, name: &str) -> Option<&ParameterValue> {
        // Uhh, TODO
        match self.parameters.get(name)? {
            Parameter::ReadOnly(pv) => Some(pv),
            Parameter::ReadWrite(pv) => Some(pv),
        }
    }
}
