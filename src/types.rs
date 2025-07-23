#[derive(Debug)]
pub struct Field<'a> {
    pub type_str: &'a str,
    pub name_str: &'a str,
    pub is_nullable: bool,
}

#[derive(Debug)]
pub struct Class<'a> {
    pub name_str: &'a str,
    pub fields: Vec<Field<'a>>,
}