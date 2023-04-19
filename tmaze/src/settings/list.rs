use crate::{renderer::Renderer, ui::menu};

use super::{editable::EditableField, ColorScheme, GenericUIError};

impl<T: EditableField + Default> EditableField for Vec<T> {
    fn print(&self) -> String {
        if self.len() != 1 {
            format!("{} items", self.len())
        } else {
            "1 item".to_string()
        }
    }

    fn edit(
        &mut self,
        renderer: &mut Renderer,
        color_scheme: ColorScheme,
    ) -> Result<bool, GenericUIError> {
        let mut selected = 0;

        loop {
            let items: Vec<_> = self
                .iter()
                .map(|t| t.print())
                .chain(["Add new item".into()])
                .collect();

            let res: Result<_, GenericUIError> = menu::menu(
                renderer,
                color_scheme.normals(),
                color_scheme.texts(),
                "Edit list",
                &items,
                Some(selected),
                false,
            )
            .map_err(|e| e.into());

            match res {
                Ok(i) if i < self.len() => {
                    selected = i;

                    match menu(
                        renderer,
                        color_scheme.normals(),
                        color_scheme.texts(),
                        "Edit item",
                        &["Edit", "Remove"],
                        None,
                        false,
                    ) {
                        Ok(0) => {
                            self[i].edit(renderer, color_scheme.clone())?;
                        }
                        Ok(1) => {
                            self.remove(i);
                        }
                        Ok(_) => unreachable!(),
                        Err(menu::MenuError::Exit) => {}
                        Err(e) => return Err(e.into()),
                    }
                }
                Ok(_) => {
                    let mut item = T::default();
                    item.edit(renderer, color_scheme.clone())?;
                    self.push(item);
                    selected = self.len() - 1;
                }
                Err(GenericUIError::Back) => return Ok(false),
                Err(e) => return Err(e),
            }
        }
    }
}
