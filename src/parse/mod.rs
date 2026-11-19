use crate::types::{Class, CopyWithClassType, Field};
use anyhow::Result;
use tree_sitter::{Node, Parser, Tree, TreeCursor};
use tree_sitter_dart::language;

pub fn get_tree(code: &str) -> Result<Tree> {
    let mut parser = Parser::new();
    parser.set_language(&language())?;
    let tree = parser
        .parse(code, None)
        .ok_or(anyhow::anyhow!("Error parsing Dart code"))?;
    Ok(tree)
}

/// Parse a complete type from a sequence of sibling nodes starting at the given index.
/// Returns the parsed type string and how many nodes were consumed.
/// 
/// This handles patterns like:
/// - `type_identifier` -> "String"
/// - `type_identifier` `type_arguments` -> "List<String>"
/// - `type_identifier` `nullable_type` -> "String?"
/// - `type_identifier` `type_arguments` `nullable_type` -> "List<String>?"
fn parse_type_from_siblings(siblings: &[Node], start_idx: usize, code: &str) -> Option<(String, usize)> {
    if start_idx >= siblings.len() {
        return None;
    }
    
    let mut idx = start_idx;
    let node = siblings[idx];
    
    // First, we need a type_identifier
    if node.kind() != "type_identifier" {
        return None;
    }
    
    let mut type_str = node.utf8_text(code.as_bytes()).unwrap().to_string();
    idx += 1;
    
    // Check if next sibling is type_arguments
    if idx < siblings.len() && siblings[idx].kind() == "type_arguments" {
        let type_args_node = siblings[idx];
        let type_args_str = parse_type_arguments(type_args_node, code);
        type_str = format!("{type_str}<{type_args_str}>");
        idx += 1;
    }
    
    // Check if next sibling is nullable_type (applies to this type)
    if idx < siblings.len() && siblings[idx].kind() == "nullable_type" {
        type_str = format!("{type_str}?");
        idx += 1;
    }
    
    Some((type_str, idx - start_idx))
}

/// Parse the contents of a type_arguments node into a comma-separated string.
/// Handles multiple type arguments and nested generics.
fn parse_type_arguments(type_args_node: Node, code: &str) -> String {
    let mut cursor = type_args_node.walk();
    let children: Vec<Node> = type_args_node.named_children(&mut cursor).collect();
    
    let mut type_strings: Vec<String> = Vec::new();
    let mut idx = 0;
    
    while idx < children.len() {
        if let Some((type_str, consumed)) = parse_type_from_siblings(&children, idx, code) {
            type_strings.push(type_str);
            idx += consumed;
        } else {
            // Skip unknown nodes
            idx += 1;
        }
    }
    
    type_strings.join(", ")
}

fn check_and_handle_class_definition<'a, 'b>(cursor: &'b mut TreeCursor<'a>, prev_node: tree_sitter::Node<'a>, code: &'a str, magic_token: &'a str) ->  Result<Option<(&'a str, tree_sitter::Node<'a>, CopyWithClassType)>> {
    // returns (class name, class_definition node, copy with type)
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
        return Ok(Some((class_name, cursor.node(), copy_with_class_type)));
    }
    Ok(None)
}

/// Parse the type of a parameter / declaration from its (named) children:
/// either the flat `type_identifier` `type_arguments`? `nullable_type`? sequence,
/// or a single `function_type` / `record_type` node (taken verbatim).
/// Returns (type without trailing `?`, is_nullable).
fn parse_param_type(children: &[Node], code: &str) -> Option<(String, bool)> {
    for (idx, child) in children.iter().enumerate() {
        match child.kind() {
            "type_identifier" => {
                let (type_str, _) = parse_type_from_siblings(children, idx, code)?;
                return Some(split_nullable(type_str));
            }
            "function_type" | "record_type" => {
                let mut type_str = child.utf8_text(code.as_bytes()).unwrap().to_string();
                if children.get(idx + 1).is_some_and(|n| n.kind() == "nullable_type") {
                    type_str.push('?');
                }
                return Some(split_nullable(type_str));
            }
            _ => {}
        }
    }
    None
}

fn split_nullable(type_str: String) -> (String, bool) {
    match type_str.strip_suffix('?') {
        Some(t) => (t.to_string(), true),
        None => (type_str, false),
    }
}

/// Type parameters of a class, e.g. for `class Gen<T extends Comparable<T>, @a U>`:
/// (`<T extends Comparable<T>, U>`, `<T, U>`). Both empty for non generic classes.
fn parse_type_parameters(class_node: Node, code: &str) -> (String, String) {
    let Some(type_parameters) = class_node.child_by_field_name("type_parameters") else {
        return (String::new(), String::new());
    };
    let mut declarations = Vec::new();
    let mut names = Vec::new();
    let mut cursor = type_parameters.walk();
    for type_parameter in type_parameters
        .named_children(&mut cursor)
        .filter(|n| n.kind() == "type_parameter")
    {
        let mut cursor = type_parameter.walk();
        // skip annotations: the declaration starts at the type parameter's name
        let Some(name) = type_parameter
            .named_children(&mut cursor)
            .find(|n| n.kind() == "type_identifier")
        else {
            continue;
        };
        let declaration = &code[name.start_byte()..type_parameter.end_byte()];
        declarations.push(declaration.split_whitespace().collect::<Vec<_>>().join(" "));
        names.push(name.utf8_text(code.as_bytes()).unwrap());
    }
    (
        format!("<{}>", declarations.join(", ")),
        format!("<{}>", names.join(", ")),
    )
}

fn last_identifier(n: Node) -> Option<Node> {
    let mut cursor = n.walk();
    n.named_children(&mut cursor)
        .filter(|c| c.kind() == "identifier")
        .last()
}

/// Fields of a primary constructor, one per parameter.
fn parse_primary_constructor_fields(
    primary_constructor: Node,
    class_name: &str,
    body_fields: &[Field],
    code: &str,
) -> Result<Vec<Field>> {
    let text = |n: Node| n.utf8_text(code.as_bytes()).unwrap();
    let Some(parameters) = primary_constructor.child_by_field_name("parameters") else {
        anyhow::bail!("Primary constructor of {class_name} has no parameter list");
    };

    // (formal_parameter, is_positional)
    let mut formal_parameters = Vec::new();
    let mut cursor = parameters.walk();
    for child in parameters.named_children(&mut cursor) {
        match child.kind() {
            "formal_parameter" => formal_parameters.push((child, true)),
            "optional_formal_parameters" => {
                let is_named = child.child(0).is_some_and(|c| c.kind() == "{");
                let mut cursor = child.walk();
                for param in child.named_children(&mut cursor) {
                    if param.kind() == "formal_parameter" {
                        formal_parameters.push((param, !is_named));
                    }
                }
            }
            "comment" | "annotation" => {}
            _ => anyhow::bail!(
                "Could not parse the primary constructor of {class_name}: `{}`",
                text(child)
            ),
        }
    }

    let mut fields = Vec::new();
    for (param, is_positional) in formal_parameters {
        let mut cursor = param.walk();
        let children: Vec<Node> = param.named_children(&mut cursor).collect();
        let field = |name_str: &str, (type_str, is_nullable): (String, bool)| Field {
            name_str: name_str.to_string(),
            type_str,
            is_nullable,
            is_positional,
        };

        match children.first().map(|n| n.kind()) {
            // `this.x` / `T this.x`: the field is declared in the body
            Some("constructor_param") => {
                let this_param = children[0];
                let name_str = text(last_identifier(this_param).unwrap());
                let mut cursor = this_param.walk();
                let this_children: Vec<Node> = this_param.named_children(&mut cursor).collect();
                if let Some(ty) = parse_param_type(&this_children, code) {
                    fields.push(field(name_str, ty));
                } else if let Some(f) = body_fields.iter().find(|f| f.name_str == name_str) {
                    fields.push(field(name_str, (f.type_str.clone(), f.is_nullable)));
                } else {
                    anyhow::bail!("Could not find the type of field `{name_str}` of {class_name}");
                }
            }
            // `T super.x`: the field lives in the superclass, so the type has to be explicit
            Some("super_formal_parameter") => {
                let super_param = children[0];
                let name_str = text(last_identifier(super_param).unwrap());
                let mut cursor = super_param.walk();
                let super_children: Vec<Node> = super_param.named_children(&mut cursor).collect();
                let Some(ty) = parse_param_type(&super_children, code) else {
                    anyhow::bail!(
                        "Parameter `super.{name_str}` of {class_name} needs an explicit type (e.g. `int super.{name_str}`)"
                    );
                };
                fields.push(field(name_str, ty));
            }
            _ => {
                let Some(name) = param.child_by_field_name("name") else {
                    anyhow::bail!(
                        "Unsupported parameter `{}` in the primary constructor of {class_name}",
                        text(param)
                    );
                };
                let name_str = text(name);
                // only declaring parameters (`final` / `var`) introduce a field
                if !children.iter().any(|n| matches!(n.kind(), "final_builtin" | "inferred_type")) {
                    anyhow::bail!(
                        "Parameter `{name_str}` of the primary constructor of {class_name} is not a field (add `final` or `var`), can't generate copyWith"
                    );
                }
                let Some(ty) = parse_param_type(&children, code) else {
                    anyhow::bail!(
                        "Parameter `{name_str}` of {class_name} needs an explicit type for copyWith generation"
                    );
                };
                fields.push(field(name_str, ty));
            }
        }
    }
    Ok(fields)
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
    for (class_name, class_node, copy_with_class_type) in classes_to_parse {
        let mut fields = Vec::new();
        let mut cursor = class_node.walk();
        // a class with a primary constructor may have a `;` body
        let field_declarations = class_node
            .child_by_field_name("body")
            .map(|class_body| {
                class_body
                    .named_children(&mut class_node.walk())
                    .filter(|node| node.kind() == "declaration")
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for field_declaration in field_declarations {
            let mut field_name = None;
            let mut field_type = None;
            let mut field_is_nullable = false;
            
            // Collect all named children to process them as siblings
            let children: Vec<Node> = field_declaration.named_children(&mut cursor).collect();
            let mut idx = 0;
            
            while idx < children.len() {
                let child = children[idx];
                
                if child.kind() == "type_identifier" {
                    // Use the new helper to parse the complete type from siblings
                    if let Some((type_str, consumed)) = parse_type_from_siblings(&children, idx, code) {
                        // Check if the parsed type ends with '?' to determine nullability
                        if type_str.ends_with('?') {
                            field_type = Some(type_str.trim_end_matches('?').to_string());
                            field_is_nullable = true;
                        } else {
                            field_type = Some(type_str);
                        }
                        idx += consumed;
                        continue;
                    }
                } else if child.kind() == "final_builtin" {
                    // Skip final keyword
                    idx += 1;
                    continue;
                } else if child.kind() == "initialized_identifier_list" {
                    field_name = child
                        .named_child(0)
                        .unwrap()
                        .utf8_text(code.as_bytes())
                        .ok();
                }
                
                idx += 1;
            }
            if let (Some(name_str), Some(type_str)) = (field_name, field_type) {
                fields.push(Field {
                    name_str: name_str.to_string(),
                    type_str,
                    is_nullable: field_is_nullable,
                    is_positional: false,
                });
            };
        }
        let (type_params, type_args) = parse_type_parameters(class_node, code);
        let mut constructor_str = format!("{class_name}{type_args}");
        // Dart 3.13 primary constructor: `class const Point._(final int x, {final int? y})`.
        // Its parameters are what copyWith has to pass, the body declarations are only
        // used to look up the types of `this.x` parameters.
        if let Some(primary_constructor) = class_node.child_by_field_name("primary_constructor") {
            if let Some(ctor) = primary_constructor.child_by_field_name("name") {
                let ctor = ctor.utf8_text(code.as_bytes()).unwrap();
                if ctor != "new" {
                    constructor_str = format!("{class_name}{type_args}.{ctor}");
                }
            }
            fields = parse_primary_constructor_fields(primary_constructor, class_name, &fields, code)?;
        }
        classes_with_fields.push(Class {
            name_str: class_name.to_string(),
            type_params,
            type_args,
            constructor_str,
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

    fn summary(classes: &[Class]) -> Vec<String> {
        classes
            .iter()
            .map(|c| {
                let fields: Vec<String> = c
                    .fields
                    .iter()
                    .map(|f| {
                        format!(
                            "{}{}{} {}",
                            if f.is_positional { "pos " } else { "" },
                            f.type_str,
                            if f.is_nullable { "?" } else { "" },
                            f.name_str
                        )
                    })
                    .collect();
                format!(
                    "{:?} {}{} via {}: {}",
                    c.copy_with_class_type,
                    c.name_str,
                    c.type_params,
                    c.constructor_str,
                    fields.join(", ")
                )
            })
            .collect()
    }

    #[test]
    fn test_get_tree_in1() {
        let code = include_str!("../../test/in1.dart");
        let tree = get_tree(code).unwrap();
        assert_eq!(
            tree.root_node().to_sexp(),
            "(program (comment) (part_directive (uri (string_literal))) (comment) (class_definition name: (identifier) body: (class_body (declaration (final_builtin) (type_identifier) (nullable_type) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (final_builtin) (type_identifier) (type_arguments (type_identifier) (nullable_type)) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (final_builtin) (type_identifier) (type_arguments (type_identifier)) (nullable_type) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (final_builtin) (type_identifier) (type_arguments (type_identifier)) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (constant_constructor_signature (const_builtin) name: (identifier) (formal_parameter_list (optional_formal_parameters (formal_parameter (constructor_param (this) (identifier))) (formal_parameter (constructor_param (this) (identifier))) (formal_parameter (constructor_param (this) (identifier))) (formal_parameter (constructor_param (this) (identifier))))))))) (comment) (class_definition name: (identifier) body: (class_body (declaration (final_builtin) (type_identifier) (nullable_type) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (final_builtin) (type_identifier) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (constant_constructor_signature (const_builtin) name: (identifier) (formal_parameter_list (optional_formal_parameters (formal_parameter (constructor_param (this) (identifier))) (formal_parameter (constructor_param (this) (identifier))))))))) (comment) (class_definition name: (identifier) body: (class_body (declaration (final_builtin) (type_identifier) (nullable_type) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (final_builtin) (type_identifier) (initialized_identifier_list (initialized_identifier (identifier)))) (declaration (constant_constructor_signature (const_builtin) name: (identifier) (formal_parameter_list (optional_formal_parameters (formal_parameter (constructor_param (this) (identifier))) (formal_parameter (constructor_param (this) (identifier))))))))))"
        );
    }

    #[test]
    fn test_parse_in1() {
        let code = include_str!("../../test/in1.dart");
        let classes = parse(code, "+mk:").unwrap();
        assert_eq!(
            summary(&classes),
            vec![
                "Mixin UiInsurancePreviewSimpleDataModel via UiInsurancePreviewSimpleDataModel: String? title, Set<String?> cta, List<String>? image, List<String> image2",
                "Extension UiInsurancePreviewDataModel via UiInsurancePreviewDataModel: String? avatar, String tagType",
                "ExtensionForcingNullableValue UiInsurancePreviewDataModel2 via UiInsurancePreviewDataModel2: String? avatar, String tagType",
            ]
        );
    }

    #[test]
    fn test_parse_in2_multi_type_args() {
        let code = include_str!("../../test/in2.model.dart");
        let classes = parse(code, "+mk:").unwrap();
        assert_eq!(
            summary(&classes),
            vec![
                "Mixin UiInsurancePreviewSimpleDataModel via UiInsurancePreviewSimpleDataModel: String? title, Set<String?> cta, List<String>? image, List<String> image2, Map<String, int> mapping, Map<String, List<int>> nestedMapping",
                "Extension UiInsurancePreviewDataModel via UiInsurancePreviewDataModel: String? avatar, String tagType",
            ]
        );
    }

    #[test]
    fn test_parse_in3_primary() {
        let code = include_str!("../../test/in3.primary.dart");
        assert!(!get_tree(code).unwrap().root_node().has_error());
        let classes = parse(code, "+mk:").unwrap();
        assert_eq!(
            summary(&classes),
            vec![
                "Extension Point via Point: pos int x, pos int? y",
                "Mixin Named via Named: String? title, Map<String, List<int>> mapping, List<String> tags",
                "Extension OldStyle via OldStyle: String? avatar, String tagType",
                "ExtensionForcingNullableValue Gen<T extends Object> via Gen<T>._: pos T value, pos List<T>? others",
                "Extension WithThis via WithThis: pos Set<String>? late, int id",
            ]
        );
    }

    #[test]
    fn test_parse_primary_function_type() {
        let code = "// +mk:copyWith\nclass A(final int Function(String)? f);\n";
        let classes = parse(code, "+mk:").unwrap();
        assert_eq!(summary(&classes), vec!["Extension A via A: pos int Function(String)? f"]);
    }

    #[test]
    fn test_parse_generic_classes() {
        let code = "// +mk:copyWith\nclass const Gen<T extends Comparable<T>, @a U>._(final T a, final U? b);\n\n// +mk:copyWith\nclass Old<T> {\n  final T v;\n  const Old({required this.v});\n}\n";
        let classes = parse(code, "+mk:").unwrap();
        assert_eq!(
            summary(&classes),
            vec![
                "Extension Gen<T extends Comparable<T>, U> via Gen<T, U>._: pos T a, pos U? b",
                "Extension Old<T> via Old<T>: T v",
            ]
        );
        assert_eq!(classes[0].type_args, "<T, U>");
    }

    #[test]
    fn test_primary_non_declaring_param_errors() {
        let code = "// +mk:copyWith\nclass A(final int x, int delta);\n";
        let err = parse(code, "+mk:").unwrap_err().to_string();
        assert!(err.contains("delta"), "{err}");
    }
}
