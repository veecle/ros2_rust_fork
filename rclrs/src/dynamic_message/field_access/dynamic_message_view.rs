use super::super::MessageStructure;
use super::{DynamicSequenceElementMut, Proxy, ProxyMut, ProxySequence, Value, ValueMut};
use std::fmt::{self, Debug};

/// A view of a single message. Used for nested messages.
///
/// This allows reading the fields of the message, but not modifying them.
#[derive(PartialEq)]
pub struct DynamicMessageView<'msg> {
    pub(crate) structure: &'msg MessageStructure,
    pub(crate) storage: &'msg [u8],
}

/// A mutable view of a single message. Used for nested messages.
///
/// This allows reading and modifying the fields of the message.
#[derive(PartialEq)]
pub struct DynamicMessageViewMut<'msg> {
    pub(crate) structure: &'msg MessageStructure,
    pub(crate) storage: &'msg mut [u8],
}

// ========================= impl for a single message =========================

impl<'msg> Debug for DynamicMessageView<'msg> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let mut struct_ = f.debug_struct(&self.structure().name);
        for field in self.structure().fields_inorder() {
            let value = self.get(&field).unwrap();
            struct_.field(&field, &value as &dyn Debug);
        }
        struct_.finish()
    }
}

unsafe impl<'msg> Proxy<'msg> for DynamicMessageView<'msg> {
    type Metadata = &'msg MessageStructure;

    fn size_in_memory(structure: Self::Metadata) -> usize {
        structure.size
    }

    unsafe fn new(bytes: &'msg [u8], structure: Self::Metadata) -> Self {
        DynamicMessageView {
            structure,
            storage: bytes,
        }
    }
}

impl<'msg> DynamicMessageView<'msg> {
    /// Tries to access a field in the message.
    ///
    /// If no field of that name exists, `None` is returned.
    pub fn get(&self, field_name: &str) -> Option<Value<'msg>> {
        unsafe { Value::new(self.storage, self.structure, field_name) }
    }

    /// Returns a description of the message structure.
    pub fn structure(&self) -> &MessageStructure {
        self.structure
    }

    /// Iterate over all fields in declaration order.
    pub fn iter_inorder(&self) -> impl Iterator<Item = (String, Value<'msg>)> + '_ {
        let fields = self.structure.fields_inorder();
        fields.into_iter().map(|field| {
            let value = self.get(&field).unwrap();
            (field, value)
        })
    }
}

impl<'msg> Debug for DynamicMessageViewMut<'msg> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        DynamicMessageView {
            structure: self.structure,
            storage: &*self.storage
        }.fmt(f)
    }
}

impl<'msg> DynamicSequenceElementMut<'msg> for DynamicMessageViewMut<'msg> {
    type InnerSequence = ProxySequence<'msg, Self>;
}

unsafe impl<'msg> ProxyMut<'msg> for DynamicMessageViewMut<'msg> {
    type Metadata = &'msg MessageStructure;

    fn size_in_memory(structure: Self::Metadata) -> usize {
        structure.size
    }

    unsafe fn new(bytes: &'msg mut [u8], structure: Self::Metadata) -> Self {
        DynamicMessageViewMut {
            structure,
            storage: bytes,
        }
    }
}

impl<'msg> DynamicMessageViewMut<'msg> {
    /// Tries to access a field in the message.
    ///
    /// If no field of that name exists, `None` is returned.
    pub fn get(&self, field_name: &str) -> Option<Value<'_>> {
        unsafe { Value::new(self.storage, self.structure, field_name) }
    }

    /// Tries to mutably access a field in the message.
    ///
    /// If no field of that name exists, `None` is returned.
    pub fn get_mut(&mut self, field_name: &str) -> Option<ValueMut<'_>> {
        unsafe { ValueMut::new(self.storage, self.structure, field_name) }
    }

    /// Returns a description of the message structure.
    pub fn structure(&self) -> &MessageStructure {
        self.structure
    }

    /// Iterate over all fields in declaration order.
    pub fn iter_inorder(&self) -> impl Iterator<Item = (String, Value<'_>)> + '_ {
        let fields = self.structure.fields_inorder();
        fields.into_iter().map(|field| {
            let value = self.get(&field).unwrap();
            (field, value)
        })
    }

    /// Iterate over all fields in declaration order (mutable version).
    pub fn iter_mut_inorder(&mut self) -> impl Iterator<Item = (String, ValueMut<'_>)> + '_ {
        let fields = self.structure.fields_inorder();
        fields.into_iter().map(|field| {
            let value = self.get_mut(&field).unwrap();
            (field, value)
        })
    }

}
