use crate::il2cpp::types::Il2CppImage;

pub mod Query;
mod PreparedQuery;
pub mod Connection;

pub fn init(image: *const Il2CppImage) {
    Query::init(image);
    PreparedQuery::init(image);
    Connection::init(image);
}