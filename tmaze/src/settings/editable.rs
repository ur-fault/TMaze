use crate::{
    renderer::Renderer,
    ui::{self, GenericUIError},
};

use super::ColorScheme;

pub trait EditableField {
    fn print(&self) -> String;

    fn edit(
        &mut self,
        renderer: &mut Renderer,
        color_scheme: ColorScheme,
    ) -> Result<bool, GenericUIError> {
        ui::popup(
            renderer,
            color_scheme.normals(),
            color_scheme.texts(),
            &format!("Current value: {}", self.print()),
            &["Cannot edit this field"],
        )
        .map(|_| false)
        .map_err(|e| e.into())
    }
}

impl EditableField for i32 {
    fn print(&self) -> String {
        self.to_string()
    }
}

impl<T: EditableField> EditableField for Option<T> {
    fn print(&self) -> String {
        match self {
            Some(value) => value.print(),
            None => "None".to_string(),
        }
    }
}
