use std::u128;

use crate::actions::{Action, ActionKind};
use crate::error::{Error, Result};
use crate::session::Session;
use crate::Entity;

pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

pub trait Deserialize {
    fn deserialize(field_reader: &mut FieldReader<'_>) -> Result<Self>
    where
        Self: Sized;
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FieldType {
    Str = 1,
    U128,
    Byte,
    Bool,
    Action,
    ActionKind,
    Entity,
    Session,
}

pub enum Field<'a> {
    Str(&'a str),
    Byte(u8),
    Bool(bool),
    U128(u128),
    Action(Action),
    ActionKind(ActionKind),
    Entity(Entity),
    Session(Session),
}

macro_rules! impl_try_from {
    ($type:ty, $field_type:path) => {
        impl TryFrom<Field<'_>> for $type {
            type Error = Error;

            fn try_from(value: Field<'_>) -> Result<Self> {
                match value {
                    $field_type(val) => Ok(val.into()),
                    _ => Err(Error::InvalidFieldType),
                }
            }
        }
    };
}

impl_try_from!(u128, Field::U128);
impl_try_from!(u8, Field::Byte);
impl_try_from!(bool, Field::Bool);
impl_try_from!(String, Field::Str);
impl_try_from!(Action, Field::Action);
impl_try_from!(ActionKind, Field::ActionKind);
impl_try_from!(Entity, Field::Entity);

fn write_len(buf: &mut Vec<u8>, len: usize) {
    let len = len as u16;
    buf.extend(len.to_be_bytes());
}

pub fn serialize(buf: &mut Vec<u8>, field: Field<'_>) {
    match field {
        Field::Str(s) => {
            buf.push(FieldType::Str as u8);
            write_len(buf, s.len());
            buf.extend_from_slice(s.as_bytes());
        }
        Field::U128(b) => {
            buf.push(FieldType::U128 as u8);
            write_len(buf, 16);
            buf.extend(b.to_be_bytes());
        }
        Field::Byte(b) => {
            buf.push(FieldType::Byte as u8);
            write_len(buf, 1);
            buf.push(b);
        }
        Field::Bool(b) => {
            buf.push(FieldType::Bool as u8);
            write_len(buf, 1);
            buf.push(b as u8);
        }
        Field::Action(action) => {
            buf.push(FieldType::Action as u8);
            let bytes = action.serialize();
            write_len(buf, bytes.len());
            buf.extend(bytes);
        }
        Field::Entity(entity) => {
            buf.push(FieldType::Entity as u8);
            let bytes = entity.serialize();
            write_len(buf, bytes.len());
            buf.extend(bytes);
        }
        Field::Session(session) => {
            buf.push(FieldType::Session as u8);
            let bytes = session.serialize();
            write_len(buf, bytes.len());
            buf.extend(bytes);
        }
        Field::ActionKind(action_kind) => {
            buf.push(FieldType::ActionKind as u8);
            write_len(buf, 1);
            buf.push(action_kind as u8);
        }
    }
}

pub struct FieldReader<'a> {
    buffer: &'a [u8],
}

impl<'a> FieldReader<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer }
    }

    fn field_type(&mut self) -> Result<FieldType> {
        if self.buffer.is_empty() {
            return Err(Error::MissingFieldType);
        }
        let byte = self.buffer[0];
        self.buffer = &self.buffer[1..];

        eprintln!("Field Type: {:?}", byte);
        match byte {
            1 => Ok(FieldType::Str),
            2 => Ok(FieldType::U128),
            3 => Ok(FieldType::Byte),
            4 => Ok(FieldType::Bool),
            5 => Ok(FieldType::Action),
            6 => Ok(FieldType::ActionKind),
            7 => Ok(FieldType::Entity),
            8 => Ok(FieldType::Session),
            _ => Err(Error::InvalidFieldType),
        }
    }

    fn len(&mut self) -> Result<usize> {
        if self.buffer.len() < 2 {
            return Err(Error::MissingFieldLen);
        }
        let bytes = &self.buffer[..2];
        self.buffer = &self.buffer[2..];
        let len = u16::from_be_bytes([bytes[0], bytes[1]]);
        Ok(len as usize)
    }

    fn read_be_u128(input: &[u8]) -> u128 {
        let (int_bytes, _) = input.split_at(std::mem::size_of::<u128>());
        u128::from_be_bytes(int_bytes.try_into().unwrap())
    }

    pub fn read_field<T>(&mut self) -> Result<T>
    where
        T: TryFrom<Field<'a>, Error = Error>,
    {
        let field_type = self.field_type()?;
        let len = self.len()?;
        eprintln!("Field Type parsed: {field_type:?} with length: {len}");
        let bytes = &self.buffer[..len];
        self.buffer = &self.buffer[len..];

        eprintln!("Entity bytes: {bytes:?}");
        eprintln!("Remaining buffer: {:?}", self.buffer);
        let field = match field_type {
            FieldType::Str => Field::Str(std::str::from_utf8(bytes)?),
            FieldType::Bool => Field::Bool(bytes[0] == 1),
            FieldType::Byte => Field::Byte(bytes[0]),
            FieldType::Action => {
                let mut new_reader = FieldReader::new(bytes);
                Field::Action(Action::deserialize(&mut new_reader)?)
            }
            FieldType::Entity => {
                let mut new_reader = FieldReader::new(bytes);
                Field::Entity(Entity::deserialize(&mut new_reader)?)
            }
            FieldType::Session => {
                let mut new_reader = FieldReader::new(bytes);
                Field::Session(Session::deserialize(&mut new_reader)?)
            }
            FieldType::U128 => Field::U128(Self::read_be_u128(bytes)),
            FieldType::ActionKind => Field::ActionKind(unsafe { std::mem::transmute(bytes[0]) }),
        };

        field.try_into()
    }
}
