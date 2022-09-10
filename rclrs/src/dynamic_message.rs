//! Functionality for working with messages whose type is not statically known.
//!
//! This is useful for writing generic tools such as introspection tools, bridges to
//! other communication systems, or nodes that manipulate messages à la `topic_tools`.
//!
//! The central type of this module is [`DynamicMessage`].

use crate::rcl_bindings::*;
use rosidl_runtime_rs::RmwMessage;
use rosidl_typesupport_introspection_c__MessageMembers_s as rosidl_message_members_t;
use std::fmt::{self, Display};
use std::path::PathBuf;
use std::sync::Arc;
use std::ops::Deref;

mod dynamic_subscription;
mod error;
mod field_access;
mod message_structure;
#[cfg(feature = "serde")]
mod ser;
pub use dynamic_subscription::*;
pub use error::*;
pub use field_access::*;
pub use message_structure::*;
#[cfg(feature = "serde")]
pub use ser::*;

/// Factory for constructing messages in a certain package dynamically.
///
/// This is the result of loading the type support library (which is a per-package operation),
/// whereas [`DynamicMessageMetadata`] is the result of loading the data related to the message
/// from the library.
//
// Theoretically it could be beneficial to make this struct public so users can "cache"
// the library loading, but unless a compelling use case comes up, I don't think it's
// worth the complexity.
//
// Under the hood, this is an `Arc<libloading::Library>`, so if this struct and the
// [`DynamicMessageMetadata`] and [`DynamicMessage`] structs created from it are dropped,
// the library will be unloaded. This shared ownership ensures that the type_support_ptr
// is always valid.
pub struct DynamicMessagePackage {
    type_support_library: Arc<libloading::Library>,
    package: String,
}

/// A parsed/validated message type name.
#[derive(Clone, Debug)]
struct MessageTypeIdentifier {
    package: String,
    type_name: String,
}

/// A runtime representation of the message "class".
///
/// This is not an instance of a message itself, but it
/// can be used as a factory to create message instances.
#[derive(Clone)]
pub struct DynamicMessageMetadata {
    message_type: MessageTypeIdentifier,
    // The library needs to be kept loaded in order to keep the type_support_ptr valid
    #[allow(dead_code)]
    type_support_library: Arc<libloading::Library>,
    type_support_ptr: *const rosidl_message_type_support_t,
    structure: MessageStructure,
    fini_function: unsafe extern "C" fn(*mut libc::c_void),
}

/// A message whose type is not known at compile-time.
///
/// This type allows inspecting the structure of the message as well as the
/// values contained in it.
/// It also allows _modifying_ the values, but not the structure, because
/// even a dynamic message must always correspond to a specific message type.
// There is no clone function yet, we need to add that in rosidl.
pub struct DynamicMessage {
    metadata: DynamicMessageMetadata,
    // This is aligned to the maximum possible alignment of a message (8)
    // by the use of a special allocation function.
    storage: Box<[u8]>,
    // This type allows moving the message contents out into another message,
    // in which case the drop impl is not responsible for calling fini anymore
    needs_fini: bool,
}

// ========================= impl for DynamicMessagePackage =========================

/// This is an analogue of rclcpp::get_typesupport_library.
pub(crate) fn get_type_support_library(
    package_name: &str,
    type_support_identifier: &str,
) -> Result<Arc<libloading::Library>, DynamicMessageError> {
    use DynamicMessageError::RequiredPrefixNotSourced;
    // Creating this is pretty cheap, it just parses an env var
    let ament = ament_rs::Ament::new().map_err(|_| RequiredPrefixNotSourced {
        package: package_name.to_owned(),
    })?;
    let prefix = PathBuf::from(ament.find_package(&package_name).ok_or(
        RequiredPrefixNotSourced {
            package: package_name.to_owned(),
        },
    )?);
    #[cfg(target_os = "windows")]
    let library_path = prefix.join("bin").join(format!(
        "{}__{}.dll",
        &package_name, type_support_identifier
    ));
    #[cfg(target_os = "macos")]
    let library_path = prefix.join("lib").join(format!(
        "lib{}__{}.dylib",
        &package_name, type_support_identifier
    ));
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let library_path = prefix.join("lib").join(format!(
        "lib{}__{}.so",
        &package_name, type_support_identifier
    ));
    Ok({
        // SAFETY: This function is unsafe because it may execute initialization/termination routines
        // contained in the library. A type support library should not cause problems there.
        let lib = unsafe { libloading::Library::new(library_path) };
        let lib = lib.map_err(DynamicMessageError::LibraryLoadingError)?;
        Arc::new(lib)
    })
}

impl DynamicMessagePackage {
    /// Creates a new `DynamicMessagePackage`.
    ///
    /// This dynamically loads a type support library for the specified package.
    pub fn new(package_name: impl Into<String>) -> Result<Self, DynamicMessageError> {
        let package_name = package_name.into();
        Ok(Self {
            type_support_library: get_type_support_library(
                &package_name,
                "rosidl_typesupport_introspection_c",
            )?,
            package: package_name,
        })
    }

    pub(crate) fn message_metadata(
        &self,
        type_name: impl Into<String>,
    ) -> Result<DynamicMessageMetadata, DynamicMessageError> {
        let type_name = type_name.into();
        let symbol_name = format!(
            "rosidl_typesupport_introspection_c__get_message_type_support_handle__{}__msg__{}",
            self.package, &type_name
        );

        // SAFETY: We know that the symbol has this type.
        let get_type_support_handle: libloading::Symbol<
            unsafe extern "C" fn() -> *const rosidl_message_type_support_t,
        > = unsafe {
            self.type_support_library
                .get(symbol_name.as_bytes())
                .map_err(|_| DynamicMessageError::InvalidMessageType)?
        };
        // SAFETY: The library is kept loaded while the type support ptr is used.
        let type_support_ptr = unsafe { get_type_support_handle() };
        // SAFETY: The pointer returned by get_type_support_handle() is always valid.
        let type_support = unsafe { &*type_support_ptr };
        debug_assert!(!type_support.data.is_null());
        let message_members: &rosidl_message_members_t =
            // SAFETY: The data pointer is supposed to be always valid.
            unsafe { &*(type_support.data as *const rosidl_message_members_t) };
        // SAFETY: The message members coming from a type support library will always be valid.
        let structure = unsafe { MessageStructure::from_rosidl_message_members(message_members) };
        // The fini function will always exist.
        let fini_function = message_members.fini_function.unwrap();
        let dyn_msg = DynamicMessageMetadata {
            message_type: MessageTypeIdentifier {
                package: self.package.clone(),
                type_name,
            },
            type_support_library: Arc::clone(&self.type_support_library),
            type_support_ptr,
            structure,
            fini_function,
        };
        Ok(dyn_msg)
    }
}

// ========================= impl for MessageTypeIdentifier =========================

impl TryFrom<&str> for MessageTypeIdentifier {
    type Error = DynamicMessageError;
    fn try_from(full_message_type: &str) -> Result<Self, Self::Error> {
        let mut parts = full_message_type.split('/');
        use DynamicMessageError::InvalidMessageTypeSyntax;
        let package = parts
            .next()
            .ok_or(InvalidMessageTypeSyntax {
                input: full_message_type.to_owned(),
            })?
            .to_owned();
        if Some("msg") != parts.next() {
            return Err(InvalidMessageTypeSyntax {
                input: full_message_type.to_owned(),
            });
        };
        let type_name = parts
            .next()
            .ok_or(InvalidMessageTypeSyntax {
                input: full_message_type.to_owned(),
            })?
            .to_owned();
        if parts.next().is_some() {
            return Err(InvalidMessageTypeSyntax {
                input: full_message_type.to_owned(),
            });
        }
        Ok(Self { package, type_name })
    }
}

impl Display for MessageTypeIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/msg/{}", &self.package, &self.type_name)
    }
}

// ========================= impl for DynamicMessageMetadata =========================

// SAFETY: The type_support_ptr member is the one that makes this type not implement Sync
// automatically, but it is not used for interior mutability.
unsafe impl Sync for DynamicMessageMetadata {}

// SAFETY: The functions accessing this type, including drop(), shouldn't care about the thread
// they are running in. Therefore, this type can be safely sent to another thread.
unsafe impl Send for DynamicMessageMetadata {}

impl DynamicMessageMetadata {
    /// Loads the metadata for the given message type.
    ///
    /// See [`DynamicMessage::new()`] for the expected format of the `full_message_type`.
    pub fn new(full_message_type: &str) -> Result<Self, DynamicMessageError> {
        let MessageTypeIdentifier { package, type_name } = full_message_type.try_into()?;
        let factory = DynamicMessagePackage::new(package)?;
        factory.message_metadata(type_name)
    }

    /// Instantiates a new message.
    pub fn create(&self) -> Result<DynamicMessage, DynamicMessageError> {
        // Get an aligned boxed slice. This is inspired by the maligned crate.
        use std::alloc::Layout;
        // As mentioned in the struct definition, the maximum alignment required is 8.
        let layout = Layout::from_size_align(self.structure.size, 8).unwrap();
        let mut storage = unsafe {
            assert_ne!(self.structure.size, 0);
            // SAFETY: The layout has non-zero size.
            let ptr = std::alloc::alloc_zeroed(layout);
            // SAFETY: This is valid, memory in ptr has appropriate size and is initialized
            let slice = std::slice::from_raw_parts_mut(ptr, self.structure.size);
            // The mutable reference decays into a (fat) *mut [u8]
            Box::from_raw(slice)
        };
        // SAFETY: The pointer returned by get_type_support_handle() is always valid.
        let type_support = unsafe { &*self.type_support_ptr };
        let message_members: &rosidl_message_members_t =
            // SAFETY: The data pointer is supposed to be always valid.
            unsafe { &*(type_support.data as *const rosidl_message_members_t) };
        // SAFETY: The init function is passed zeroed memory of the correct alignment.
        unsafe {
            (message_members.init_function.unwrap())(
                storage.as_mut_ptr() as _,
                rosidl_runtime_c__message_initialization::ROSIDL_RUNTIME_C_MSG_INIT_ALL,
            );
        };
        let dyn_msg = DynamicMessage {
            metadata: self.clone(),
            storage,
            needs_fini: true,
        };
        Ok(dyn_msg)
    }

    /// Returns a description of the message structure.
    pub fn structure(&self) -> &MessageStructure {
        &self.structure
    }
}

// ========================= impl for DynamicMessage =========================

impl<'msg> Deref for DynamicMessage {
    type Target = MessageStructure;
    fn deref(&self) -> &Self::Target {
        &self.metadata.structure
    }
}


impl Drop for DynamicMessage {
    fn drop(&mut self) {
        if self.needs_fini {
            // SAFETY: The fini_function expects to be passed a pointer to the message
            unsafe { (self.metadata.fini_function)(self.storage.as_mut_ptr() as _) }
        }
    }
}

impl PartialEq for DynamicMessage {
    fn eq(&self, other: &Self) -> bool {
        self.metadata.type_support_ptr == other.metadata.type_support_ptr
            && self.storage == other.storage
    }
}

impl Eq for DynamicMessage {}

impl DynamicMessage {
    /// Dynamically loads a type support library for the specified type and creates a message instance.
    ///
    /// The full message type is of the form `<package>/msg/<type_name>`, e.g.
    /// `std_msgs/msg/String`.
    ///
    /// The message instance will contain the default values of the message type.
    pub fn new(full_message_type: &str) -> Result<Self, DynamicMessageError> {
        DynamicMessageMetadata::new(full_message_type)?.create()
    }

    /// See [`DynamicMessageView::get()`][1].
    ///
    /// [1]: crate::dynamic_message::DynamicMessageView::get
    pub fn get(&self, field_name: &str) -> Option<Value<'_>> {
        unsafe { Value::new(&self.storage, &self.metadata.structure, field_name) }
    }

    /// See [`DynamicMessageViewMut::get_mut()`][1].
    ///
    /// [1]: crate::dynamic_message::DynamicMessageViewMut::get_mut
    pub fn get_mut(&mut self, field_name: &str) -> Option<ValueMut<'_>> {
        let field_info = self.metadata.structure.get_field_info(field_name)?;
        // For the unwrap_or, see DynamicMessageViewMut::get_mut
        let size = field_info.size().unwrap_or(1);
        let bytes = &mut self.storage[field_info.offset..field_info.offset + size];
        Some(unsafe { ValueMut::new(bytes, field_info) })
    }

    /// Returns a description of the message structure.
    pub fn structure(&self) -> &MessageStructure {
        &self.metadata.structure
    }

    // /// Iterate over all fields in declaration order.
    // pub fn iter_inorder(&self) -> impl Iterator<Item = (String, Value<'msg>)> + '_ {
    //     let fields = self.structure.fields_inorder();
    //     fields.into_iter().map(|field| {
    //         let value = self.get(&field).unwrap();
    //         (field, value)
    //     })
    // }

    // /// Iterate over all fields in declaration order (mutable version).
    // pub fn iter_mut_inorder(&self) -> impl Iterator<Item = (String, ValueMut<'msg>)> + '_ {
    //     let fields = self.structure.fields_inorder();
    //     fields.into_iter().map(|field| {
    //         let value = self.get_mut(&field).unwrap();
    //         (field, value)
    //     })
    // }

    /// Returns a view object of this message.
    ///
    /// The purpose for this conversion is to allow uniform handling of this top-level message
    /// and nested messages contained in it through a [`DynamicMessageView`].
    pub fn view(&self) -> DynamicMessageView<'_> {
        DynamicMessageView {
            structure: &self.metadata.structure,
            storage: &self.storage,
        }
    }

    /// Returns a mutable view object of this message.
    ///
    /// The purpose for this conversion is to allow uniform handling of this top-level message
    /// and nested messages contained in it through a [`DynamicMessageViewMut`].
    pub fn view_mut(&mut self) -> DynamicMessageViewMut<'_> {
        DynamicMessageViewMut {
            structure: &self.metadata.structure,
            storage: &mut self.storage,
        }
    }

    /// Converts a statically typed RMW-native message into a `DynamicMessage`.
    pub fn convert_from_rmw_message<T>(mut msg: T) -> Result<Self, DynamicMessageError>
    where
        T: RmwMessage,
    {
        let mut dyn_msg = Self::new(<T as RmwMessage>::TYPE_NAME)?;
        let align = std::mem::align_of::<T>();
        assert_eq!(dyn_msg.storage.as_ptr().align_offset(align), 0);
        {
            // SAFETY: This transmutes the slice of bytes into a &mut T. This is fine, since
            // under the hood it *is* a T.
            // However, the resulting value is not seen as borrowing from dyn_msg by the borrow checker,
            // so we are careful to not create a second mutable reference before dropping this one,
            // since that would be UB.
            let dyn_msg_transmuted = unsafe { &mut *(dyn_msg.storage.as_mut_ptr() as *mut T) };
            // We cannot simply overwrite one message with the other, or we will get a memory leak/double-free.
            // Swapping is the solution.
            std::mem::swap(&mut msg, dyn_msg_transmuted);
        }
        Ok(dyn_msg)
    }

    /// Converts a `DynamicMessage` into a statically typed RMW-native message.
    ///
    /// If the RMW-native message type does not match the underlying message type of this `DynamicMessage`,
    /// it is not converted but instead returned unchanged.
    pub fn convert_into_rmw_message<T>(mut self) -> Result<T, Self>
    where
        T: RmwMessage,
    {
        if <T as RmwMessage>::TYPE_NAME == self.metadata.message_type.to_string() {
            // SAFETY: Even though a zero-initialized message might not match RMW expectations for
            // what a message should look like, it is safe to temporarily have a zero-initialized
            // value, i.e. it is not undefined behavior to do this since it's a C struct, and an
            // all-zeroes bit pattern is always a valid instance of any C struct.
            let mut dest = unsafe { std::mem::zeroed::<T>() };
            let dest_ptr = &mut dest as *mut T as *mut u8;
            // This reinterprets the struct as a slice of bytes.
            // The bytes copied into the dest slice are a valid value of T, as ensured by comparison
            // of the type support pointers.
            let dest_slice =
                unsafe { std::slice::from_raw_parts_mut(dest_ptr, std::mem::size_of::<T>()) };
            // This creates a shallow copy, with ownership of the "deep" (or inner) parts moving
            // into the destination.
            dest_slice.copy_from_slice(&*self.storage);
            // Don't run the fini function on the src data anymore, because the inner parts would be
            // double-freed by dst and src.
            self.needs_fini = false;
            Ok(dest)
        } else {
            Err(self)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn all_types_are_sync_and_send() {
        assert_send::<DynamicMessageMetadata>();
        assert_sync::<DynamicMessageMetadata>();
        assert_send::<DynamicMessage>();
        assert_sync::<DynamicMessage>();
    }
}
