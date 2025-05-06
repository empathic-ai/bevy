use crate::serde::ser::error_utils::make_custom_error;
use crate::serde::TypedReflectSerializer;
use crate::{Array, TypeRegistry};
use serde::ser::SerializeTuple;
use serde::Serialize;

use super::ReflectSerializerProcessor;

/// A serializer for [`Array`] values.
pub(super) struct ArraySerializer<'a, P> {
    pub array: &'a dyn Array,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<P: ReflectSerializerProcessor> Serialize for ArraySerializer<'_, P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let type_info = self.array.get_represented_type_info().ok_or_else(|| {
            make_custom_error(format_args!(
                "cannot get type info for `{}`",
                self.array.reflect_type_path()
            ))
        })?;

        let array_info = type_info.as_array().map_err(make_custom_error)?;
        let item_info = array_info.item_info();

        let mut state = serializer.serialize_tuple(self.array.len())?;
        for value in self.array.iter() {
            state.serialize_element(&TypedReflectSerializer::new_internal(
                value,
                self.registry,
                self.processor,
            ))?;
        }
        state.end()
    }
}
