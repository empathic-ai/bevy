use crate::serde::ser::error_utils::make_custom_error;
use crate::serde::TypedReflectSerializer;
use crate::{List, TypeRegistry};
use serde::ser::SerializeSeq;
use serde::Serialize;

use super::ReflectSerializerProcessor;

/// A serializer for [`List`] values.
pub(super) struct ListSerializer<'a, P> {
    pub list: &'a dyn List,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<P: ReflectSerializerProcessor> Serialize for ListSerializer<'_, P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let type_info = self.list.get_represented_type_info().ok_or_else(|| {
            make_custom_error(format_args!(
                "cannot get type info for `{}`",
                self.list.reflect_type_path()
            ))
        })?;

        let list_info = type_info.as_list().map_err(make_custom_error)?;
        let item_info = list_info.item_info();

        let mut state = serializer.serialize_seq(Some(self.list.len()))?;
        for value in self.list.iter() {
            state.serialize_element(&TypedReflectSerializer::new_internal(
                value,
                self.registry,
                self.processor,
            ))?;
        }
        state.end()
    }
}
