pub struct Field {
    /// includes ? at the end if null-able
    typeStr: String,
    nameStr: String,
}

pub struct Class {
    name: String,
    fields: Vec<Field>,
}