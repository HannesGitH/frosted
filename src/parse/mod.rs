use crate::types::{Class, Field};
use tree_sitter::{InputEdit, Language, Node, Parser, Point, Tree, TreeCursor};
use tree_sitter_dart::language;

pub fn get_tree(code: &str) -> Result<Tree, String> {
    let mut parser = Parser::new();
    parser.set_language(&language()).map_err(|e| e.to_string())?;
    let tree = parser.parse(code, None).ok_or("Error parsing Dart code")?;
    Ok(tree)
}

pub fn parse(code: &str, magic_token: &str) -> Result<Vec<Class>, String> {
    let tree = get_tree(code)?;
    let root_node = tree.root_node();
    let mut cursor = root_node.walk();
    
    // go into program
    cursor.goto_first_child();
    let mut classes_to_parse = Vec::new();
    let mut prev_node = root_node;
    loop {
        if cursor.node().kind() == "class_definition" {
            if prev_node.kind() != "comment" {continue};
            if !prev_node.utf8_text(&code.as_bytes()).unwrap().contains(magic_token) {continue};
            let class_name = cursor.node().child(1).unwrap().utf8_text(&code.as_bytes()).unwrap();
            let class_body = cursor.node().child(2).unwrap();
            classes_to_parse.push((class_name, class_body.walk()));
        }
        prev_node = cursor.node();
        if !cursor.goto_next_sibling() {
            break;
        }
    }
    // now get the class definitions
    println!("{:?}",classes_to_parse.iter().map(|(name, body)| name).collect::<Vec<_>>());
    Ok(Vec::new())
}   

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_get_tree_in1() {
        let code = include_str!("../../test/in1.dart");
        let tree = get_tree(code).unwrap();
        assert_eq!(tree.root_node().to_sexp(), "(program (comment) (import_or_export (library_export (configurable_uri (uri (string_literal))))) (comment) (class_definition name: (identifier) body: (class_body (declaration (final_builtin) (type_identifier) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (final_builtin) (type_identifier) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (final_builtin) (type_identifier) (nullable_type) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (constant_constructor_signature (const_builtin) (identifier) (formal_parameter_list (optional_formal_parameters (formal_parameter (constructor_param (this) (identifier))) (formal_parameter (constructor_param (this) (identifier))) (formal_parameter (constructor_param (this) (identifier))))))))) (comment) (class_definition name: (identifier) superclass: (superclass (type_identifier)) body: (class_body (declaration (final_builtin) (type_identifier) (nullable_type) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (final_builtin) (type_identifier) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (constant_constructor_signature (const_builtin) (identifier) (formal_parameter_list (optional_formal_parameters (formal_parameter (constructor_param (this) (identifier))) (formal_parameter (super_formal_parameter (super) (identifier))) (formal_parameter (super_formal_parameter (super) (identifier))) (formal_parameter (super_formal_parameter (super) (identifier))) (formal_parameter (constructor_param (this) (identifier))))))))))");
    }

    #[test]
    fn test_parse_in1() {
        let code = include_str!("../../test/in1.dart");
        let fields = parse(code, "+mk:").unwrap();
    }
}