// Resolves Links between objects

use object::{Key, MapField, Object};
use server::LinkResolution;
use std::collections::HashSet;

/// Manages link resolution
pub struct LinkResolver<'a, T: storage::Store> {
    /// The max number of resolutions that can occur
    max_resolution_depth: u8,
    /// Links that have already been visited, used for cycle detection
    visited: HashSet<Key>,
    /// The storage medium used
    storage: &'a T,
}

impl<'a, T: storage::Store> LinkResolver<'a, T> {
    /// Creates a new LinkResolver
    pub fn new(link_resolutions: LinkResolution, storage: &'a T) -> Self {
        Self {
            max_resolution_depth: link_resolutions.max_resolution_depth(),
            visited: HashSet::new(),
            storage,
        }
    }

    /// Resolves the links of an object and returns a new Object with the Links resolved
    pub fn resolve(&mut self, object: Object) -> storage::StoreResult {
        self.resolve_links(object, 0)
    }

    fn resolve_links(&mut self, object: Object, resolution_depth: u16) -> storage::StoreResult {
        if resolution_depth > self.max_resolution_depth as u16 {
            return Ok(object);
        }
        let resolution_depth = resolution_depth + 1;

        match object {
            Object::List(items) => self.resolve_list_links(items, resolution_depth),
            Object::Map(fields) => self.resolve_map_links(fields, resolution_depth),
            Object::Link(key) => self.resolve_link(key, resolution_depth),
            other => Ok(other),
        }
    }

    fn resolve_list_links(
        &mut self,
        items: Vec<Object>,
        resolution_depth: u16,
    ) -> storage::StoreResult {
        let mut resolved = Vec::with_capacity(items.len());
        for item in items {
            resolved.push(self.resolve_links(item, resolution_depth)?);
        }
        Ok(Object::List(resolved))
    }

    fn resolve_map_links(
        &mut self,
        fields: Vec<MapField>,
        resolution_depth: u16,
    ) -> storage::StoreResult {
        let mut resolved = Vec::with_capacity(fields.len());
        for field in fields {
            let value = self.resolve_links(field.value, resolution_depth)?;
            resolved.push(MapField {
                name: field.name,
                value,
            });
        }
        Ok(Object::Map(resolved))
    }

    fn resolve_link(&mut self, key: Key, resolution_depth: u16) -> storage::StoreResult {
        if self.visited.contains(&key) {
            return Ok(Object::Link(key));
        }

        let object = self.storage.retrieve(&key)?;
        self.visited.insert(key);
        self.resolve_links(object, resolution_depth)
    }
}
