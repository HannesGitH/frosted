use crate::types::{Class, CopyWithClassType};
use anyhow::Result;
use handlebars::Handlebars;
use serde_json::json;

pub struct Generator<'a> {
    handlebars: Handlebars<'a>,
}

impl<'a> Generator<'a> {
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string(
            "template",
            include_str!("../../templates/main.dart.handlebars"),
        )?;
        Ok(Self { handlebars })
    }

    pub fn generate(&self, classes: &[Class], file_name: &str) -> Result<String> {
        let rendered = self.handlebars.render("template", &json!({
            "file_name": file_name,
            "classes": classes.iter().map(|c| json!({
                "class_name": c.name_str,
                "is_mixin": c.copy_with_class_type == CopyWithClassType::Mixin,
                "fields": c.fields.iter().map(|f| json!({
                    "name_str": f.name_str,
                    "type_str": f.type_str,
                    "type_str_nullable": format!("{}{}", f.type_str, if f.is_nullable { "?" } else { "" }),
                    "type_str_wrapped_nullable_value": if f.is_nullable { format!("NullableValue<{}?>?", f.type_str) } else { format!("{}?", f.type_str) },
                    "is_nullable": f.is_nullable,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        }))?;
        Ok(rendered)
    }
}
