use anyhow::Result;
use handlebars::Handlebars;
use serde_json::json;
use crate::types::Class;

pub fn generate(classes: &[Class], file_name: &str) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_file("template", "template.dart.handlebars")?;
    let rendered = handlebars.render("template", &json!({
        "file_name": file_name,
        "classes": classes.iter().map(|c| json!({
            "class_name": c.name_str,
            "fields": c.fields.iter().map(|f| json!({
                "name_str": f.name_str,
                "type_str_nullable": format!("{}{}", f.type_str, if f.is_nullable { "?" } else { "" }),
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    }))?;
    Ok(rendered)
}