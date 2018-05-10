extern crate fnv;
extern crate linked_hash_map;

use fnv::FnvBuildHasher;
use linked_hash_map::LinkedHashMap;
use std::hash::Hash;
use std::borrow::Borrow;

#[cfg(feature = "heapsize_impl")]
mod heapsize;

pub type FnvLinkedMap<K, V> = LinkedHashMap<K, V, FnvBuildHasher>;

/// A iterator over the items of the LRU cache.
pub type LruCacheIter<'a, K, V> = linked_hash_map::Iter<'a, K, V>;

/// A iterator over the mutable items of the LRU cache.
pub type LruCacheIterMut<'a, K, V> = linked_hash_map::IterMut<'a, K, V>;

pub type LruCacheKeys<'a, K, V> = linked_hash_map::Keys<'a, K, V>;

#[derive(Clone, Debug, Default)]
pub struct LruCache<K: Eq + Hash, V> {
    inner: FnvLinkedMap<K, V>,
    max_size: usize,
    refresh: bool,
}

impl<K: Eq + Hash, V> LruCache<K, V> {
    pub fn new(capacity: usize, refresh: bool) -> Self {
        LruCache {
            refresh,
            inner: FnvLinkedMap::default(),
            max_size: capacity,
        }
    }

    pub fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.contains_key(k)
    }

    //refresh order
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        if self.refresh {
            self.inner.get_refresh(k)
        } else {
            self.inner.get_mut(k)
        }
    }

    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.get(k)
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let old_val = self.inner.insert(k, v);
        if self.len() > self.max_size {
            self.pop_front();
        }
        old_val
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn pop_front(&mut self) -> Option<(K, V)> {
        self.inner.pop_front()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn keys(&self) -> LruCacheKeys<K, V> {
        self.inner.keys()
    }

    pub fn iter(&self) -> LruCacheIter<K, V> {
        self.inner.iter()
    }

    /// Return an iterator over the key-value pairs of the map, in their order
    pub fn iter_mut(&mut self) -> LruCacheIterMut<K, V> {
        self.inner.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_get() {
        let mut cache = LruCache::new(2, false);
        cache.insert(1, 10);
        cache.insert(2, 20);
        assert_eq!(cache.get(&1), Some(&10));
        assert_eq!(cache.get(&2), Some(&20));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_put_update() {
        let mut cache = LruCache::new(1, false);
        cache.insert("1", 10);
        cache.insert("1", 19);
        assert_eq!(cache.get("1"), Some(&19));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_contains_key() {
        let mut cache = LruCache::new(1, false);
        cache.insert("1", 10);
        assert_eq!(cache.contains_key("1"), true);
    }

    #[test]
    fn test_expire_lru() {
        let mut cache = LruCache::new(2, false);
        cache.insert("foo1", "bar1");
        cache.insert("foo2", "bar2");
        cache.insert("foo3", "bar3");
        assert!(cache.get("foo1").is_none());
        cache.insert("foo2", "bar2update");
        cache.insert("foo4", "bar4");
        assert!(cache.get("foo3").is_none());
    }
}
