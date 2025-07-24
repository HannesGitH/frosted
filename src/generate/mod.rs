use anyhow::Result;
use handlebars::Handlebars;
use serde_json::json;
use crate::types::{Class, CopyWithClassType};

pub fn generate(classes: &[Class], file_name: &str) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("template", include_str!("../../templates/main.dart.handlebars"))?;
    let rendered = handlebars.render("template", &json!({
        "file_name": file_name,
        "classes": classes.iter().map(|c| json!({
            "class_name": c.name_str,
            "is_mixin": c.copy_with_class_type == CopyWithClassType::Mixin,
            "fields": c.fields.iter().map(|f| json!({
                "name_str": f.name_str,
                "type_str": f.type_str,
                "type_str_nullable": format!("{}{}", f.type_str, if f.is_nullable { "?" } else { "" }),
                "type_str_wrapped_nullable_value": format!("{}?", f.type_str),
                "is_nullable": f.is_nullable,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    }))?;
    Ok(rendered)
}