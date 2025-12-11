#[derive(Debug)]
pub struct Field {
    pub type_str: String,
    pub name_str: String,
    pub is_nullable: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CopyWithClassType {
    Extension,
    Mixin,
    ExtensionForcingNullableValue,
}

#[derive(Debug)]
pub struct Class {
    pub name_str: String,
    pub fields: Vec<Field>,
    pub copy_with_class_type: CopyWithClassType,
}
