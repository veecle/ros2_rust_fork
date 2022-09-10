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
        for field in &self.structure().fields {
            let value = self.get(&field.name).unwrap();
            struct_.field(&field.name, &value as &dyn Debug);
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
        self.structure.fields.iter().map(|field_info| {
            let value = self.get(&field_info.name).unwrap();
            (field_info.name.clone(), value)
        })
    }
}

impl<'msg> Debug for DynamicMessageViewMut<'msg> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        DynamicMessageView {
            structure: self.structure,
            storage: &*self.storage,
        }
        .fmt(f)
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
        let field_info = self.structure.get(field_name)?;
        // The size is None for LongDouble, which has platform-dependent size.
        // It's fine to pass in 1 here – the length of the slice isn't strictly needed
        // by this function, especially not for a LongDouble value.
        let size = field_info.size().unwrap_or(1);
        let bytes = &mut self.storage[field_info.offset..field_info.offset + size];
        Some(unsafe { ValueMut::new(bytes, field_info) })
    }

    /// Returns a description of the message structure.
    pub fn structure(&self) -> &MessageStructure {
        self.structure
    }

    /// Iterate over all fields in declaration order.
    pub fn iter_inorder(&self) -> impl Iterator<Item = (String, Value<'_>)> + '_ {
        self.structure.fields.iter().map(|field_info| {
            let value = self.get(&field_info.name).unwrap();
            (field_info.name.clone(), value)
        })
    }

    /// Iterate over all fields in declaration order (mutable version).
    pub fn into_iter_inorder(self) -> impl Iterator<Item = (String, ValueMut<'msg>)> + 'msg {
        self.structure
            .fields
            .iter()
            .rev()
            .scan(
                self.storage,
                |remainder: &mut &'msg mut [u8], field_info| {
                    // remainder is of type &'closure mut &'a mut [i32],
                    // and calling remainder.split_at_mut would move out of
                    // the outer reference, so it's forbidden
                    let rem = std::mem::take(remainder);
                    let (init, tail) = rem.split_at_mut(field_info.offset);
                    *remainder = init;
                    Some((field_info, tail))
                },
            )
            .map(|(field_info, value_bytes)| {
                (field_info.name.clone(), unsafe {
                    ValueMut::new(value_bytes, field_info)
                })
            })
    }

    /// Iterate over all fields in declaration order (mutable version).
    pub fn iter_mut_inorder(&mut self) -> impl Iterator<Item = (String, ValueMut<'_>)> + '_ {
        self.structure
            .fields
            .iter()
            .rev()
            .scan(
                &mut *self.storage,
                |remainder: &mut &'msg mut [u8], field_info| {
                    // remainder is of type &'closure mut &'a mut [i32],
                    // and calling remainder.split_at_mut would move out of
                    // the outer reference, so it's forbidden
                    let rem = std::mem::take(remainder);
                    let (init, tail) = rem.split_at_mut(field_info.offset);
                    *remainder = init;
                    Some((field_info, tail))
                },
            )
            .map(|(field_info, value_bytes)| {
                (field_info.name.clone(), unsafe {
                    ValueMut::new(value_bytes, field_info)
                })
            })
    }
}
