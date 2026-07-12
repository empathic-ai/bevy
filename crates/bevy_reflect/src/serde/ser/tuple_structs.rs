use crate::serde::ser::error_utils::make_custom_error;
use crate::serde::{ReflectSerializer, SerializationData, TypedReflectSerializer};
use crate::{DynamicStruct, ReflectRef, TupleStruct, TypeRegistry};
use serde::ser::SerializeTupleStruct;
use serde::Serialize;

use super::ReflectSerializerProcessor;

/// A serializer for [`TupleStruct`] values.
pub(super) struct TupleStructSerializer<'a, P> {
    pub tuple_struct: &'a dyn TupleStruct,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<P: ReflectSerializerProcessor> Serialize for TupleStructSerializer<'_, P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let type_info = self
            .tuple_struct
            .get_represented_type_info()
            .ok_or_else(|| {
                make_custom_error(format_args!(
                    "cannot get type info for `{}`",
                    self.tuple_struct.reflect_type_path()
                ))
            })?;

        let tuple_struct_info = type_info.as_tuple_struct().map_err(make_custom_error)?;

        let serialization_data = self
            .registry
            .get(type_info.type_id())
            .and_then(|registration| registration.data::<SerializationData>());
        let ignored_len = serialization_data.map(SerializationData::len).unwrap_or(0);

        if self.tuple_struct.field_len() == 1 && serialization_data.is_none() {
            let field = self.tuple_struct.field(0).unwrap();
            let field_type = tuple_struct_info.field_at(0).unwrap().type_id();

            return if field_type == std::any::TypeId::of::<DynamicStruct>() {
                serializer.serialize_newtype_struct(
                    tuple_struct_info.type_path_table().ident().unwrap(),
                    &ReflectSerializer::new_internal(field, self.registry, self.processor),
                )
            } else {
                serializer.serialize_newtype_struct(
                    tuple_struct_info.type_path_table().ident().unwrap(),
                    &TypedReflectSerializer::new_internal(field, self.registry, self.processor),
                )
            };
        }

        let mut state = serializer.serialize_tuple_struct(
            tuple_struct_info.type_path_table().ident().unwrap(),
            self.tuple_struct.field_len() - ignored_len,
        )?;

        for (index, value) in self.tuple_struct.iter_fields().enumerate() {
            if serialization_data.is_some_and(|data| data.is_field_skipped(index)) {
                continue;
            }

            let field_type = tuple_struct_info.field_at(index).unwrap().type_id();

            if field_type == std::any::TypeId::of::<DynamicStruct>() {
                state.serialize_field(&ReflectSerializer::new_internal(
                    value,
                    self.registry,
                    self.processor,
                ))?;
            } else {
                state.serialize_field(&TypedReflectSerializer::new_internal(
                    value,
                    self.registry,
                    self.processor,
                ))?;
            }
        }
        state.end()
    }
}
