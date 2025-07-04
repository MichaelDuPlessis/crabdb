use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    ops,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

type Shard<K, V> = RwLock<HashMap<K, V>>;

/// Holds an immutable reference to a value and a guard to prevent race conditions
pub struct Ref<'a, K, V> {
    /// The guard of the map
    guard: RwLockReadGuard<'a, HashMap<K, V>>,
    /// The value being returned
    value: *const V,
}

impl<'a, K, V> ops::Deref for Ref<'a, K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value }
    }
}

/// Holds an mutable reference to a value and a guard to prevent race conditions
pub struct RefMut<'a, K, V> {
    /// The guard of the map
    guard: RwLockWriteGuard<'a, HashMap<K, V>>,
    /// The value being returned
    value: *mut V,
}

impl<'a, K, V> ops::Deref for RefMut<'a, K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value }
    }
}

impl<'a, K, V> ops::DerefMut for RefMut<'a, K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.value }
    }
}

/// A HashMap that can be shared between threads safely while still remaining effiecient
pub struct ConcurrentMap<K, V> {
    /// The shards in the hashmap
    shards: Vec<Shard<K, V>>,
}

impl<K, V> ConcurrentMap<K, V> {
    /// Creates a new concurrent hashmap with the specified number of shards
    pub fn new(num_shards: usize) -> Self {
        let mut shards = Vec::with_capacity(num_shards);

        for _ in 0..num_shards {
            shards.push(RwLock::new(HashMap::default()));
        }

        Self { shards }
    }

    /// Gets the numbre of shards in the hashmap
    fn num_shards(&self) -> usize {
        self.shards.len()
    }
}

impl<K, V> ConcurrentMap<K, V>
where
    K: Hash + Eq,
{
    /// Gets the shard index that a key belongs to
    fn shard_index(&self, key: &K) -> usize {
        // TODO: Should the default hasher be used?
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.num_shards()
    }

    /// Gets the shard that a key belongs too
    fn shard(&self, key: &K) -> &Shard<K, V> {
        // getting the index that the key belongs in
        let shard_index = self.shard_index(&key);
        // getting shard
        let shard = unsafe { self.shards.get_unchecked(shard_index) };

        shard
    }

    /// Inserts a new element into the ConcurrentMap
    /// and returns the element was already there if one exists
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        // getting the shard
        let shard = self.shard(&key);
        // inserting the value
        let mut map = shard.write().unwrap();
        map.insert(key, value)
    }

    /// Reads an element from the map and returns None if no element is found
    pub fn get<'a>(&'a self, key: &K) -> Option<Ref<'a, K, V>> {
        // getting the shard
        let shard = self.shard(key);

        // getting the element
        let map_guard = shard.read().unwrap();
        match map_guard.get(key) {
            Some(value) => {
                let value_ptr = value as *const V;
                Some(Ref {
                    guard: map_guard,
                    value: value_ptr,
                })
            }
            None => None,
        }
    }

    /// Reads an element from the map and returns None if no element is found
    pub fn get_mut<'a>(&'a self, key: &K) -> Option<RefMut<'a, K, V>> {
        // getting the shard
        let shard = self.shard(key);

        // getting the element
        let mut map_guard = shard.write().unwrap();
        match map_guard.get_mut(key) {
            Some(value) => {
                let value_ptr = value as *mut V;
                Some(RefMut {
                    guard: map_guard,
                    value: value_ptr,
                })
            }
            None => None,
        }
    }

    /// Removes an element from the ConcurrentMap. Returns None if the element does not exist
    pub fn remove(&self, key: &K) -> Option<V> {
        // getting the shard
        let shard = self.shard(&key);
        // inserting the value
        let mut map = shard.write().unwrap();
        map.remove(key)
    }
}
