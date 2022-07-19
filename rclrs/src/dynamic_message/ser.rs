use serde::{ser::SerializeStruct, Serialize, Serializer};
use serde_value::Value;
use std::collections::BTreeMap;

use super::{DynamicMessageView, Field};

impl<'msg> DynamicMessageView<'msg> {
    pub fn to_serde_value(&self) -> Value {
        let field_names = self.structure.fields_inorder();
        let mut map = BTreeMap::new();
        for field_name in field_names {
            use Field::*;
            // unwrap() is ok here since the field name is guaranteed to exist
            let value = match self.field(&field_name).unwrap() {
                Float(x) => Value::F32(*x),
                Double(x) => Value::F64(*x),
                LongDouble(..) => panic!("LongDouble is not supported"),
                Char(x) => Value::U8(*x),
                WChar(x) => Value::U16(*x),
                Boolean(x) => Value::Bool(*x),
                Octet(x) => Value::U8(*x),
                Uint8(x) => Value::U8(*x),
                Int8(x) => Value::I8(*x),
                Uint16(x) => Value::U16(*x),
                Int16(x) => Value::I16(*x),
                Uint32(x) => Value::U32(*x),
                Int32(x) => Value::I32(*x),
                Uint64(x) => Value::U64(*x),
                Int64(x) => Value::I64(*x),
                String(x) => Value::String(x.to_string()),
                WString(x) => Value::String(x.to_string()),
                Message(x) => x.to_serde_value(),
                // ------------------------------------------------------------------------
                FloatArray(x) => Value::Seq(x.iter().copied().map(Value::F32).collect()),
                DoubleArray(x) => Value::Seq(x.iter().copied().map(Value::F64).collect()),
                LongDoubleArray(..) => panic!("LongDouble is not supported"),
                CharArray(x) => Value::Seq(x.iter().copied().map(Value::U8).collect()),
                WCharArray(x) => Value::Seq(x.iter().copied().map(Value::U16).collect()),
                BooleanArray(x) => Value::Seq(x.iter().copied().map(Value::Bool).collect()),
                OctetArray(x) => Value::Seq(x.iter().copied().map(Value::U8).collect()),
                Uint8Array(x) => Value::Seq(x.iter().copied().map(Value::U8).collect()),
                Int8Array(x) => Value::Seq(x.iter().copied().map(Value::I8).collect()),
                Uint16Array(x) => Value::Seq(x.iter().copied().map(Value::U16).collect()),
                Int16Array(x) => Value::Seq(x.iter().copied().map(Value::I16).collect()),
                Uint32Array(x) => Value::Seq(x.iter().copied().map(Value::U32).collect()),
                Int32Array(x) => Value::Seq(x.iter().copied().map(Value::I32).collect()),
                Uint64Array(x) => Value::Seq(x.iter().copied().map(Value::U64).collect()),
                Int64Array(x) => Value::Seq(x.iter().copied().map(Value::I64).collect()),
                StringArray(x) => Value::Seq(
                    x.iter()
                        .map(ToString::to_string)
                        .map(Value::String)
                        .collect(),
                ),
                WStringArray(x) => Value::Seq(
                    x.iter()
                        .map(ToString::to_string)
                        .map(Value::String)
                        .collect(),
                ),
                MessageArray(x) => {
                    Value::Seq(x.iter().map(DynamicMessageView::to_serde_value).collect())
                }
                // ------------------------------------------------------------------------
                FloatSequence(x) => Value::Seq(x.iter().copied().map(Value::F32).collect()),
                DoubleSequence(x) => Value::Seq(x.iter().copied().map(Value::F64).collect()),
                LongDoubleSequence(..) => panic!("LongDouble is not supported"),
                CharSequence(x) => Value::Seq(x.iter().copied().map(Value::U8).collect()),
                WCharSequence(x) => Value::Seq(x.iter().copied().map(Value::U16).collect()),
                BooleanSequence(x) => Value::Seq(x.iter().copied().map(Value::Bool).collect()),
                OctetSequence(x) => Value::Seq(x.iter().copied().map(Value::U8).collect()),
                Uint8Sequence(x) => Value::Seq(x.iter().copied().map(Value::U8).collect()),
                Int8Sequence(x) => Value::Seq(x.iter().copied().map(Value::I8).collect()),
                Uint16Sequence(x) => Value::Seq(x.iter().copied().map(Value::U16).collect()),
                Int16Sequence(x) => Value::Seq(x.iter().copied().map(Value::I16).collect()),
                Uint32Sequence(x) => Value::Seq(x.iter().copied().map(Value::U32).collect()),
                Int32Sequence(x) => Value::Seq(x.iter().copied().map(Value::I32).collect()),
                Uint64Sequence(x) => Value::Seq(x.iter().copied().map(Value::U64).collect()),
                Int64Sequence(x) => Value::Seq(x.iter().copied().map(Value::I64).collect()),
                StringSequence(x) => Value::Seq(
                    x.iter()
                        .map(ToString::to_string)
                        .map(Value::String)
                        .collect(),
                ),
                WStringSequence(x) => Value::Seq(
                    x.iter()
                        .map(ToString::to_string)
                        .map(Value::String)
                        .collect(),
                ),
                MessageSequence(x) => {
                    Value::Seq(x.iter().map(DynamicMessageView::to_serde_value).collect())
                }
                // // ------------------------------------------------------------------------
                FloatBoundedSequence(x) => Value::Seq(x.iter().copied().map(Value::F32).collect()),
                DoubleBoundedSequence(x) => Value::Seq(x.iter().copied().map(Value::F64).collect()),
                LongDoubleBoundedSequence(..) => panic!("LongDouble is not supported"),
                CharBoundedSequence(x) => Value::Seq(x.iter().copied().map(Value::U8).collect()),
                WCharBoundedSequence(x) => Value::Seq(x.iter().copied().map(Value::U16).collect()),
                BooleanBoundedSequence(x) => {
                    Value::Seq(x.iter().copied().map(Value::Bool).collect())
                }
                OctetBoundedSequence(x) => Value::Seq(x.iter().copied().map(Value::U8).collect()),
                Uint8BoundedSequence(x) => Value::Seq(x.iter().copied().map(Value::U8).collect()),
                Int8BoundedSequence(x) => Value::Seq(x.iter().copied().map(Value::I8).collect()),
                Uint16BoundedSequence(x) => Value::Seq(x.iter().copied().map(Value::U16).collect()),
                Int16BoundedSequence(x) => Value::Seq(x.iter().copied().map(Value::I16).collect()),
                Uint32BoundedSequence(x) => Value::Seq(x.iter().copied().map(Value::U32).collect()),
                Int32BoundedSequence(x) => Value::Seq(x.iter().copied().map(Value::I32).collect()),
                Uint64BoundedSequence(x) => Value::Seq(x.iter().copied().map(Value::U64).collect()),
                Int64BoundedSequence(x) => Value::Seq(x.iter().copied().map(Value::I64).collect()),
                StringBoundedSequence(x) => Value::Seq(
                    x.iter()
                        .map(ToString::to_string)
                        .map(Value::String)
                        .collect(),
                ),
                WStringBoundedSequence(x) => Value::Seq(
                    x.iter()
                        .map(ToString::to_string)
                        .map(Value::String)
                        .collect(),
                ),
                MessageBoundedSequence(x) => {
                    Value::Seq(x.iter().map(DynamicMessageView::to_serde_value).collect())
                }
            };
            map.insert(Value::String(field_name), value);
        }
        Value::Map(map)
    }
}

// impl<'msg> Serialize for DynamicMessageView<'msg> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let field_names = self.structure.fields_inorder();
//         let mut msg = serializer.serialize_struct(&self.structure.name, field_names.len())?;
//         for field_name in field_names.iter() {
//             use Field::*;
//             // unwrap() is ok here since the field name is guaranteed to exist
//             match self.field(field_name).unwrap() {
//                 Float(f32_value) => msg.serialize_field(field_name, f32_value),
//                 Double(f64_value) => msg.serialize_field(field_name, f64_value),
//                 LongDouble(..) => msg.skip_field(field_name),
//                 Char(u8_value) => msg.serialize_field(field_name, u8_value),
//                 WChar(u16_value) => msg.serialize_field(field_name, u16_value),
//                 Boolean(bool_value) => msg.serialize_field(field_name, bool_value),
//                 Octet(u8_value) => msg.serialize_field(field_name, u8_value),
//                 Uint8(u8_value) => msg.serialize_field(field_name, u8_value),
//                 Int8(i8_value) => msg.serialize_field(field_name, i8_value),
//                 Uint16(u16_value) => msg.serialize_field(field_name, u16_value),
//                 Int16(i16_value) => msg.serialize_field(field_name, i16_value),
//                 Uint32(u32_value) => msg.serialize_field(field_name, u32_value),
//                 Int32(i32_value) => msg.serialize_field(field_name, i32_value),
//                 Uint64(u64_value) => msg.serialize_field(field_name, u64_value),
//                 Int64(i64_value) => msg.serialize_field(field_name, i64_value),
//                 String(s) => msg.serialize_field(field_name, s),
//                 WString(ws) => msg.serialize_field(field_name, ws),
//                 Message(inner) => msg.serialize_field(field_name, &inner),
//                 // ------------------------------------------------------------------------
//                 FloatArray(f32_slice) => msg.serialize_field(field_name, f32_slice),
//                 DoubleArray(f64_slice) => msg.serialize_field(field_name, f64_slice),
//                 LongDoubleArray(..) => msg.skip_field(field_name),
//                 CharArray(u8_slice) => msg.serialize_field(field_name, u8_slice),
//                 WCharArray(u16_slice) => msg.serialize_field(field_name, u16_slice),
//                 BooleanArray(bool_slice) => msg.serialize_field(field_name, bool_slice),
//                 OctetArray(u8_slice) => msg.serialize_field(field_name, u8_slice),
//                 Uint8Array(u8_slice) => msg.serialize_field(field_name, u8_slice),
//                 Int8Array(i8_slice) => msg.serialize_field(field_name, i8_slice),
//                 Uint16Array(u16_slice) => msg.serialize_field(field_name, u16_slice),
//                 Int16Array(i16_slice) => msg.serialize_field(field_name, i16_slice),
//                 Uint32Array(u32_slice) => msg.serialize_field(field_name, u32_slice),
//                 Int32Array(i32_slice) => msg.serialize_field(field_name, i32_slice),
//                 Uint64Array(u64_slice) => msg.serialize_field(field_name, u64_slice),
//                 Int64Array(i64_slice) => msg.serialize_field(field_name, i64_slice),
//                 StringArray(string_slice) => msg.serialize_field(field_name, string_slice),
//                 WStringArray(wstring_slice) => msg.serialize_field(field_name, wstring_slice),
//                 MessageArray(inner) => msg.serialize_field(field_name, &*inner),
//                 // ------------------------------------------------------------------------
//                 FloatSequence(f32_seq) => msg.serialize_field(field_name, f32_seq),
//                 DoubleSequence(f64_seq) => msg.serialize_field(field_name, f64_seq),
//                 LongDoubleSequence(..) => msg.skip_field(field_name),
//                 CharSequence(u8_seq) => msg.serialize_field(field_name, u8_seq),
//                 WCharSequence(u16_seq) => msg.serialize_field(field_name, u16_seq),
//                 BooleanSequence(bool_seq) => msg.serialize_field(field_name, bool_seq),
//                 OctetSequence(u8_seq) => msg.serialize_field(field_name, u8_seq),
//                 Uint8Sequence(u8_seq) => msg.serialize_field(field_name, u8_seq),
//                 Int8Sequence(i8_seq) => msg.serialize_field(field_name, i8_seq),
//                 Uint16Sequence(u16_seq) => msg.serialize_field(field_name, u16_seq),
//                 Int16Sequence(i16_seq) => msg.serialize_field(field_name, i16_seq),
//                 Uint32Sequence(u32_seq) => msg.serialize_field(field_name, u32_seq),
//                 Int32Sequence(i32_seq) => msg.serialize_field(field_name, i32_seq),
//                 Uint64Sequence(u64_seq) => msg.serialize_field(field_name, u64_seq),
//                 Int64Sequence(i64_seq) => msg.serialize_field(field_name, i64_seq),
//                 StringSequence(string_seq) => msg.serialize_field(field_name, string_seq),
//                 WStringSequence(wstring_seq) => msg.serialize_field(field_name, wstring_seq),
//                 MessageSequence(inner) => msg.serialize_field(field_name, &*inner),
//                 // ------------------------------------------------------------------------
//                 FloatBoundedSequence(f32_bounded_seq) => msg.serialize_field(field_name, f32_bounded_seq.as_slice()),
//                 DoubleBoundedSequence(f64_bounded_seq) => msg.serialize_field(field_name, f64_bounded_seq.as_slice()),
//                 LongDoubleBoundedSequence(..) => msg.skip_field(field_name),
//                 CharBoundedSequence(u8_bounded_seq) => msg.serialize_field(field_name, u8_bounded_seq.as_slice()),
//                 WCharBoundedSequence(u16_bounded_seq) => msg.serialize_field(field_name, u16_bounded_seq.as_slice()),
//                 BooleanBoundedSequence(bool_bounded_seq) => msg.serialize_field(field_name, bool_bounded_seq.as_slice()),
//                 OctetBoundedSequence(u8_bounded_seq) => msg.serialize_field(field_name, u8_bounded_seq.as_slice()),
//                 Uint8BoundedSequence(u8_bounded_seq) => msg.serialize_field(field_name, u8_bounded_seq.as_slice()),
//                 Int8BoundedSequence(i8_bounded_seq) => msg.serialize_field(field_name, i8_bounded_seq.as_slice()),
//                 Uint16BoundedSequence(u16_bounded_seq) => msg.serialize_field(field_name, u16_bounded_seq.as_slice()),
//                 Int16BoundedSequence(i16_bounded_seq) => msg.serialize_field(field_name, i16_bounded_seq.as_slice()),
//                 Uint32BoundedSequence(u32_bounded_seq) => msg.serialize_field(field_name, u32_bounded_seq.as_slice()),
//                 Int32BoundedSequence(i32_bounded_seq) => msg.serialize_field(field_name, i32_bounded_seq.as_slice()),
//                 Uint64BoundedSequence(u64_bounded_seq) => msg.serialize_field(field_name, u64_bounded_seq.as_slice()),
//                 Int64BoundedSequence(i64_bounded_seq) => msg.serialize_field(field_name, i64_bounded_seq.as_slice()),
//                 StringBoundedSequence(string_bounded_seq) => msg.serialize_field(field_name, string_bounded_seq.as_slice()),
//                 WStringBoundedSequence(wstring_bounded_seq) => msg.serialize_field(field_name, wstring_bounded_seq.as_slice()),
//                 MessageBoundedSequence(inner) => msg.serialize_field(field_name, &*inner),
//             }?;
//         }
//         msg.end()
//     }
// }
