use crate::rcl_bindings::rosidl_typesupport_introspection_c__MessageMember_s as rosidl_message_member_t;
use crate::rcl_bindings::rosidl_typesupport_introspection_c__MessageMembers_s as rosidl_message_members_t;
use crate::rcl_bindings::*;

use std::collections::HashMap;
use std::ffi::CStr;
use std::ops::Index;

/// A description of the structure of a message.
///
/// Namely, the list of fields and their types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageStructure {
    /// The set of fields in the message, indexed by name.
    pub fields: HashMap<String, MessageFieldType>,
    /// The size of this structure in bytes.
    pub size: usize,
    /// The name of this type.
    pub name: String,
}

impl Index<&str> for MessageStructure {
    type Output = MessageFieldType;

    fn index(&self, index: &str) -> &Self::Output {
        self.fields.index(index)
    }
}

impl MessageStructure {
    /// Parses the C struct containing a list of fields.
    // That function must be unsafe, since it is possible to safely create a garbage non-null
    // pointer and store it in a rosidl_message_members_t.
    pub(crate) unsafe fn from_rosidl_message_members(
        message_members: &rosidl_message_members_t,
    ) -> Self {
        debug_assert!(!message_members.members_.is_null());
        let num_fields: usize = usize::try_from(message_members.member_count_).unwrap();
        let fields = (0..num_fields)
            .map(|i| {
                // SAFETY: This is an array as per the documentation
                let rosidl_message_member: &rosidl_message_member_t =
                    /*unsafe*/ { &*message_members.members_.add(i) };
                debug_assert!(!rosidl_message_member.name_.is_null());
                // SAFETY: This is a valid string pointer
                let name = /*unsafe*/ { CStr::from_ptr(rosidl_message_member.name_) }
                    .to_string_lossy()
                    .into_owned();
                (name, MessageFieldType::from(rosidl_message_member))
            })
            .collect();
        // SAFETY: Immediate conversion into owned string.
        let name = /*unsafe*/ {
            CStr::from_ptr(message_members.message_name_)
                .to_string_lossy()
                .into_owned()
        };
        Self {
            fields,
            size: message_members.size_of_,
            name,
        }
    }

    /// Returns the field names in the order they appear in the message.
    pub fn fields_inorder(&self) -> Vec<String> {
        let mut fields_by_offset: Vec<_> = self
            .fields
            .keys()
            .map(String::from)
            .collect();
        fields_by_offset.sort_by_cached_key(|field_name| self.fields[field_name].offset);
        fields_by_offset
    }
}

/// Information on whether a field is a single value or a list of some kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueKind {
    /// This field is a single value, which includes string values.
    Simple,
    /// This field is an array of values.
    Array {
        /// The array length.
        length: usize,
    },
    /// This field is a [`Sequence`][1] of values.
    ///
    /// [1]: rosidl_runtime_rs::Sequence
    Sequence,
    /// This field is a [`BoundedSequence`][1] of values.
    ///
    /// [1]: rosidl_runtime_rs::BoundedSequence
    BoundedSequence {
        /// The maximum sequence length.
        upper_bound: usize,
    },
}

/// A description of a single field in a [`DynamicMessage`][1].
///
/// The concrete type of a field is the combination of its [`BaseType`] with its [`ValueKind`].
/// That is, the base types exist as single values, arrays, bounded sequences and unbounded sequences.
///
/// [1]: crate::dynamic_message::DynamicMessage
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageFieldType {
    /// The base type – number, string, etc.
    pub base_type: BaseType,
    pub(crate) is_array: bool,
    pub(crate) is_upper_bound: bool,
    pub(crate) array_size: usize,
    pub(crate) string_upper_bound: usize,
    pub(crate) resize_function:
        Option<unsafe extern "C" fn(arg1: *mut libc::c_void, size: usize) -> bool>,
    pub(crate) offset: usize,
}

impl MessageFieldType {
    // That function must be unsafe, since it is possible to safely create a garbage non-null
    // pointer and store it in a rosidl_message_member_t.
    unsafe fn from(rosidl_message_member: &rosidl_message_member_t) -> Self {
        Self {
            base_type: BaseType::from_type_id(
                rosidl_message_member.type_id_,
                rosidl_message_member.members_,
            ),
            is_array: rosidl_message_member.is_array_,
            is_upper_bound: rosidl_message_member.is_upper_bound_,
            array_size: rosidl_message_member.array_size_,
            string_upper_bound: rosidl_message_member.string_upper_bound_,
            resize_function: rosidl_message_member.resize_function,
            offset: usize::try_from(rosidl_message_member.offset_).unwrap(),
        }
    }
}

impl MessageFieldType {
    /// Returns whether the field is a simple value, an array, or a (bounded) sequence.
    pub fn value_kind(&self) -> ValueKind {
        match (
            self.is_array,
            self.resize_function.is_some(),
            self.is_upper_bound,
        ) {
            (false, _, _) => ValueKind::Simple,
            (true, false, _) => ValueKind::Array {
                length: self.array_size,
            },
            (true, true, false) => ValueKind::Sequence,
            (true, true, true) => ValueKind::BoundedSequence {
                upper_bound: self.array_size,
            },
        }
    }
}

/// Possible base types for fields in a message.
// The field variants are self-explaining, no need to add redundant documentation.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BaseType {
    /// AKA `float32` in ROS .msg files.
    Float,
    /// AKA `float64` in ROS .msg files.
    Double,
    LongDouble,
    Char,
    WChar,
    Boolean,
    /// AKA `byte` in ROS .msg files.
    Octet,
    Uint8,
    Int8,
    Uint16,
    Int16,
    Uint32,
    Int32,
    Uint64,
    Int64,
    String,
    WString,
    Message(Box<MessageStructure>),
}

impl BaseType {
    // The inner message type support will be nullptr except for the case of a nested message.
    // That function must be unsafe, since it is possible to safely create a garbage non-null
    // pointer.
    unsafe fn from_type_id(type_id: u8, inner: *const rosidl_message_type_support_t) -> Self {
        use rosidl_typesupport_introspection_c_field_types::*;
        match u32::from(type_id) {
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_FLOAT as u32 => Self::Float,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_DOUBLE as u32 => Self::Double,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_LONG_DOUBLE as u32 => {
                Self::LongDouble
            }
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_CHAR as u32 => Self::Char,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_WCHAR as u32 => Self::WChar,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_BOOLEAN as u32 => Self::Boolean,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_OCTET as u32 => Self::Octet,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_UINT8 as u32 => Self::Uint8,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_INT8 as u32 => Self::Int8,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_UINT16 as u32 => Self::Uint16,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_INT16 as u32 => Self::Int16,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_UINT32 as u32 => Self::Uint32,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_INT32 as u32 => Self::Int32,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_UINT64 as u32 => Self::Uint64,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_INT64 as u32 => Self::Int64,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_STRING as u32 => Self::String,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_WSTRING as u32 => Self::WString,
            x if x == rosidl_typesupport_introspection_c__ROS_TYPE_MESSAGE as u32 => {
                assert!(!inner.is_null());
                let type_support: &rosidl_message_type_support_t = &*inner;
                let message_members: &rosidl_message_members_t =
                    // SAFETY: The data pointer is supposed to be always valid.
                    &*(type_support.data as *const rosidl_message_members_t);
                let structure = MessageStructure::from_rosidl_message_members(message_members);
                Self::Message(Box::new(structure))
            }
            _ => panic!("Invalid field type"),
        }
    }
}
