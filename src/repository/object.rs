pub trait Serializable {
    fn serialize(&self) -> &[u8];
    fn deserialize(input: &[u8]) -> Self;
}

#[derive(Copy, Clone, Debug)]
pub enum Type {
    Commit,
    Tree,
    Tag,
    Blob,
}

pub struct GitObject {
    pub object_type: Type,
    pub data: Vec<u8>
}

impl Serializable for GitObject {
    fn serialize(&self) -> &[u8]{
        &self.data
    }

    fn deserialize(_input: &[u8]) -> Self {
        unimplemented!()
    }
}

impl Serializable for Type {
    fn serialize(&self) -> &[u8] {
        match *self {
            Type::Commit => b"commit",
            Type::Tree => b"tree",
            Type::Tag => b"tag",
            Type::Blob => b"blob",
        }
    }

    fn deserialize(input: &[u8]) -> Type {
        match input {
            b"commit" => Type::Commit,
            b"tree" => Type::Tree,
            b"tag" => Type::Tag,
            b"blob" => Type::Blob,
            _ => Type::Commit
        }
    }
}

impl GitObject {
    pub fn new(object_type: Type, data: &[u8]) -> GitObject {
        GitObject {
            object_type,
            data: data.to_vec()
        }
    }
}
