//use std::io::Cursor;

use args::Args;
use error::Result;
use serde::{serialize, Deserialize, Field, FieldReader, Serialize};
use session::Session;

pub mod actions;
pub mod args;
pub mod error;
pub mod serde;
pub mod session;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub name: String,
    field_b: u8,
    field_c: bool,
}

impl Entity {
    pub fn new(name: String) -> Self {
        Self {
            name,
            field_b: 0,
            field_c: false,
        }
    }
}

impl Serialize for Entity {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = vec![];
        serialize(&mut bytes, Field::Str(&self.name));
        serialize(&mut bytes, Field::Byte(self.field_b));
        serialize(&mut bytes, Field::Bool(self.field_c));
        bytes
    }
}

impl Deserialize for Entity {
    fn deserialize(reader: &mut FieldReader<'_>) -> Result<Self>
    where
        Self: Sized,
    {
        let entity = Self {
            name: reader.read_field()?,
            field_b: reader.read_field()?,
            field_c: reader.read_field()?,
        };

        Ok(entity)
    }
}

fn print_help() {
    println!("HELP!");
    println!("-----");
    println!("  -h, --help        | Show this help");
    println!("  new <name>        | Create a new session");
    println!("  load <name>       | Load a session");
    println!("  action <action>   | Act upon a session");
}

fn main() -> Result<()> {
    let args = Args::parse()?;

    //let session = Session::load().unwrap();
    match args {
        Args::Help => print_help(),
        Args::Action(kind, target) => {
            eprintln!("args are {kind:?} and {target}");
            let session = Session::load()?;
            eprintln!("Session comprises of: {session:?}");
        }
        Args::New(name) => {
            let entity = Entity::new(name);
            let session = Session::new(entity)?;
            session.save()?;
            println!("session saved");
        }
        Args::Load(name) => {
            eprintln!("name is {name:?}");
            let entity = Session::load()?;
            eprintln!("{entity:?}");
        }
    }

    Ok(())
}
