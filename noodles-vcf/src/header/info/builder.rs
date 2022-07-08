use indexmap::IndexMap;

use super::{tag, Info, Key, Type};
use crate::header::Number;

#[derive(Default)]
pub struct Builder {
    id: Option<Key>,
    number: Option<Number>,
    ty: Option<Type>,
    description: Option<String>,
    idx: Option<usize>,
    other_fields: IndexMap<tag::Other, String>,
}

impl Builder {
    pub fn set_id(mut self, id: Key) -> Self {
        self.id = Some(id);
        self
    }

    pub fn set_number(mut self, number: Number) -> Self {
        self.number = Some(number);
        self
    }

    pub fn set_type(mut self, ty: Type) -> Self {
        self.ty = Some(ty);
        self
    }

    pub fn set_description<D>(mut self, description: D) -> Self
    where
        D: Into<String>,
    {
        self.description = Some(description.into());
        self
    }

    pub fn set_idx(mut self, idx: usize) -> Self {
        self.idx = Some(idx);
        self
    }

    pub fn insert(mut self, key: tag::Other, value: String) -> Self {
        self.other_fields.insert(key, value);
        self
    }

    pub fn build(self) -> Result<Info, BuildError> {
        use super::{DESCRIPTION, ID, NUMBER, TYPE};

        Ok(Info {
            id: self.id.ok_or(BuildError::MissingField(ID))?,
            number: self.number.ok_or(BuildError::MissingField(NUMBER))?,
            ty: self.ty.ok_or(BuildError::MissingField(TYPE))?,
            description: self
                .description
                .ok_or(BuildError::MissingField(DESCRIPTION))?,
            idx: self.idx,
            fields: self.other_fields,
        })
    }
}

pub enum BuildError {
    MissingField(&'static str),
}
