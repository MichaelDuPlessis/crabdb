// Resolves Links between objects

use object::Key;

/// Resolves the links of an object and returns a new Object with the Links resolved
/// It requires a reference to the storage to resolve the links
pub fn resolve_links<T: storage::Store>(
    object: object::Object,
    storage: &T,
) -> storage::StoreResult {
    match object.kind() {
        object::ObjectKind::List => resolve_list_links(object.data(), storage),
        object::ObjectKind::Map => resolve_map_links(object.data(), storage),
        object::ObjectKind::Link => resolve_link(object.data(), storage),
        // Int, Text, Null cannot contain links
        _ => Ok(object),
    }
}

/// Resolve links for a List
fn resolve_list_links<T: storage::Store>(object_data: &[u8], storage: &T) -> storage::StoreResult {
    todo!()
}

/// Resolve links for a Map
fn resolve_map_links<T: storage::Store>(object_data: &[u8], storage: &T) -> storage::StoreResult {
    todo!()
}

/// Resolve links for a Link
fn resolve_link<T: storage::Store>(object_data: &[u8], storage: &T) -> storage::StoreResult {
    // This is safe since the only way a key could get stored is if it is valid
    // if something happend that made it not valid then we have a bigger problem
    let key = unsafe { Key::new_unchecked(object_data) };

    let object = storage.retrieve(key)?;
    // the object retrieve may also have links that need to be resolved
    resolve_links(object, storage)
}
