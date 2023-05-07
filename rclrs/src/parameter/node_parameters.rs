use std::collections::BTreeMap;

use super::{Constraints, ParameterOverrideMap, ParameterValue};

pub(crate) enum DeclaredParameter {
    Bool(DeclarablibabilaParameter<bool>),
    Integer(DeclarablibabilaParameter<i64>),
    Double(DeclarablibabilaParameter<f64>),
    String(DeclarablibabilaParameter<String>),
    ByteArray(DeclarablibabilaParameter<Vec<u8>>),
    BoolArray(DeclarablibabilaParameter<Vec<bool>>),
    IntegerArray(DeclarablibabilaParameter<Vec<i64>>),
    DoubleArray(DeclarablibabilaParameter<Vec<f64>>),
    StringArray(DeclarablibabilaParameter<Vec<String>>),
}

// An opaque wrapper, because the inner type has invariants we want to uphold.
pub struct DeclarablibabilaParameter<T>(ParameterWithConstraints<T>);

// Invariant: The value always fulfills the constraints.
// Invariant: Read-only values cannot change.
enum ParameterWithConstraints<T> {
    ReadOnly(T, Constraints<T>), // TODO: Is this an Option too?
    ReadWrite(Option<T>, Constraints<T>),
}

impl DeclaredParameter {
    fn from_override(param: &ParameterValue, is_readonly: bool) -> Self {
        macro_rules! convert {
            ($value:expr) => {
                DeclarablibabilaParameter(if is_readonly {
                    ParameterWithConstraints::ReadOnly($value.clone(), vec![])
                } else {
                    ParameterWithConstraints::ReadWrite(Some($value.clone()), vec![])
                })
            };
        }
        match param {
            ParameterValue::Bool(x) => DeclaredParameter::Bool(convert!(x)),
            ParameterValue::Integer(x) => DeclaredParameter::Integer(convert!(x)),
            ParameterValue::Double(x) => DeclaredParameter::Double(convert!(x)),
            ParameterValue::String(x) => DeclaredParameter::String(convert!(x)),
            ParameterValue::ByteArray(x) => DeclaredParameter::ByteArray(convert!(x)),
            ParameterValue::BoolArray(x) => DeclaredParameter::BoolArray(convert!(x)),
            ParameterValue::IntegerArray(x) => DeclaredParameter::IntegerArray(convert!(x)),
            ParameterValue::DoubleArray(x) => DeclaredParameter::DoubleArray(convert!(x)),
            ParameterValue::StringArray(x) => DeclaredParameter::StringArray(convert!(x)),
        }
    }
}

pub enum ParameterError {
    TypeMismatch,
    InvalidName,
}

// Would the user be able to

// RCLCPP_PUBLIC
// virtual
// const rclcpp::ParameterValue &
// declare_parameter(
// const std::string & name,
// const rclcpp::ParameterValue & default_value,
// const rcl_interfaces::msg::ParameterDescriptor & parameter_descriptor =
// rcl_interfaces::msg::ParameterDescriptor(),
// bool ignore_override = false) = 0;

// It is a read-only parameter if

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
    parameters: BTreeMap<String, DeclaredParameter>,
    allow_set_parameters_service_to_declare_parameters: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutomaticallyDeclareParametersFromOverrides {
    No,
    ReadOnly,
    ReadWrite,
}

impl NodeParameters {
    pub fn new(
        overrides: ParameterOverrideMap,
        automatically_declare_parameters_from_overrides: AutomaticallyDeclareParametersFromOverrides,
        allow_set_parameters_service_to_declare_parameters: bool,
    ) -> Self {
        use AutomaticallyDeclareParametersFromOverrides::*;
        let parameters = match automatically_declare_parameters_from_overrides {
            No => BTreeMap::new(),
            mutability @ (ReadOnly | ReadWrite) => overrides
                .iter()
                .map(|(name, value)| {
                    let param = DeclaredParameter::from_override(value, mutability == ReadOnly);
                    (name.clone(), param)
                })
                .collect(),
        };
        Self {
            overrides,
            parameters,
            allow_set_parameters_service_to_declare_parameters,
        }
    }

    // pub fn get_unused_overrides()

    // pub fn get(&self, name: &str) -> Option<&ParameterValue> {
    //     match self.parameters.get(name)? {
    //         Parameter::InitialValue(_) => None,
    //         Parameter::ReadOnly(pv) => Some(pv),
    //         Parameter::ReadWrite(pv) => Some(pv),
    //     }
    // }

    // pub fn entry<T>(&self, name: &str) -> Option<ParameterEntry<T>> {
    //     let param = self.parameters.get_mut(name)?;
    // }

    // declare = entry::<String>("foo")?;
}
