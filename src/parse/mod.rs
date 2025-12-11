use crate::types::{Class, CopyWithClassType, Field};
use anyhow::Result;
use tree_sitter::{Parser, Tree, TreeCursor};
use tree_sitter_dart::language;

pub fn get_tree(code: &str) -> Result<Tree> {
    let mut parser = Parser::new();
    parser.set_language(&language())?;
    let tree = parser
        .parse(code, None)
        .ok_or(anyhow::anyhow!("Error parsing Dart code"))?;
    Ok(tree)
}

fn check_and_handle_class_definition<'a, 'b>(cursor: &'b mut TreeCursor<'a>, prev_node: tree_sitter::Node<'a>, code: &'a str, magic_token: &'a str) ->  Result<Option<(&'a str, tree_sitter::Node<'a>, CopyWithClassType)>> {
    if cursor.node().kind() == "class_definition" {
        if prev_node.kind() != "comment" {
            return Ok(None);
        };
        let comment = prev_node.utf8_text(&code.as_bytes()).unwrap();
        if !comment.contains(magic_token) {
            return Ok(None);
        };
        let copy_with_class_type = if comment.contains("+mk:copyWithMixin") {
            CopyWithClassType::Mixin
        } else if comment.contains("+mk:copyWithNullableValue") {
            CopyWithClassType::ExtensionForcingNullableValue
        } else if comment.contains("+mk:copyWith") {
            CopyWithClassType::Extension
        } else {
            anyhow::bail!("Unknown copy with class type: {}", comment);
        };
        let class_name = cursor
            .node()
            .child_by_field_name("name")
            .unwrap()
            .utf8_text(&code.as_bytes())
            .unwrap();
        let class_body = cursor.node().child_by_field_name("body").unwrap();
        return Ok(Some((class_name, class_body, copy_with_class_type)));
    }
    Ok(None)
}

pub fn parse(code: &str, magic_token: &str) -> Result<Vec<Class>> {
    let tree = get_tree(code)?;
    let root_node = tree.root_node();
    let mut cursor = root_node.walk();

    // go into program
    cursor.goto_first_child();

    // now get the class definitions
    let mut classes_to_parse = Vec::new();
    let mut prev_node = root_node;
    loop {
        let new_class = check_and_handle_class_definition(&mut cursor, prev_node, code, magic_token)?;
        if let Some(new_class) = new_class {
            classes_to_parse.push(new_class.clone());
        }
        prev_node = cursor.node();
        if !cursor.goto_next_sibling() {
            break;
        }
    }

    // now get the fields in these classes
    let mut classes_with_fields = Vec::new();
    for (class_name, class_body, copy_with_class_type) in classes_to_parse {
        let mut fields = Vec::new();
        let mut cursor = class_body.walk();
        let field_declarations = class_body
            .named_children(&mut cursor)
            .filter(|node| node.kind() == "declaration")
            .collect::<Vec<_>>();
        for field_declaration in field_declarations {
            let mut field_name = None;
            let mut field_type = None;
            let mut field_is_nullable = false;
            for child in field_declaration.named_children(&mut cursor) {
                if child.kind() == "type_identifier" {
                    field_type = Some(child.utf8_text(&code.as_bytes()).unwrap().to_string());
                } else if child.kind() == "type_arguments" {
                    let mut cursor = child.walk();
                    let mut inner_is_nullable = false;
                    let mut inner_type = None;
                    for inner_child in child.named_children(&mut cursor) {
                        if inner_child.kind() == "type_identifier" {
                            inner_type = Some(inner_child.utf8_text(&code.as_bytes()).unwrap());
                        } else if inner_child.kind() == "nullable_type" {
                            inner_is_nullable = true;
                        }
                    }
                    if let Some(inner_type) = inner_type {
                        field_type = Some(format!(
                            "{}<{}{}>",
                            field_type.unwrap(),
                            inner_type,
                            if inner_is_nullable { "?" } else { "" }
                        ));
                    }
                } else if child.kind() == "nullable_type" {
                    field_is_nullable = true;
                } else if child.kind() == "initialized_identifier_list" {
                    field_name = child
                        .named_child(0)
                        .unwrap()
                        .utf8_text(&code.as_bytes())
                        .ok();
                }
            }
            if let (Some(name_str), Some(type_str)) = (field_name, field_type) {
                fields.push(Field {
                    name_str: name_str.to_string(),
                    type_str,
                    is_nullable: field_is_nullable,
                });
            };
        }
        classes_with_fields.push(Class {
            name_str: class_name.to_string(),
            fields,
            copy_with_class_type,
        });
    }

    Ok(classes_with_fields)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_get_tree_in1() {
        let code = include_str!("../../test/in1.dart");
        let tree = get_tree(code).unwrap();
        assert_eq!(
            tree.root_node().to_sexp(),
            "(program (comment) (import_or_export (library_export (configurable_uri (uri (string_literal))))) (comment) (class_definition name: (identifier) body: (class_body (declaration (final_builtin) (type_identifier) (nullable_type) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (final_builtin) (type_identifier) (type_arguments (type_identifier) (nullable_type)) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (final_builtin) (type_identifier) (type_arguments (type_identifier)) (nullable_type) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (final_builtin) (type_identifier) (type_arguments (type_identifier)) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (constant_constructor_signature (const_builtin) (identifier) (formal_parameter_list (optional_formal_parameters (formal_parameter (constructor_param (this) (identifier))) (formal_parameter (constructor_param (this) (identifier))) (formal_parameter (constructor_param (this) (identifier))) (formal_parameter (constructor_param (this) (identifier))))))))) (comment) (class_definition name: (identifier) body: (class_body (declaration (final_builtin) (type_identifier) (nullable_type) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (final_builtin) (type_identifier) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (constant_constructor_signature (const_builtin) (identifier) (formal_parameter_list (optional_formal_parameters (formal_parameter (constructor_param (this) (identifier))) (formal_parameter (constructor_param (this) (identifier))))))))))"
        );
    }

    #[test]
    fn test_parse_in1() {
        let code = include_str!("../../test/in1.dart");
        let classes = parse(code, "+mk:").unwrap();
        println!("{:?}", classes);
    }
}
