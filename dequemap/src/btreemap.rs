use alloc::collections::vec_deque::IntoIter as DequeIntoIter;
use alloc::collections::vec_deque::Iter as DequeIter;

use alloc::collections::BTreeSet;
use alloc::collections::VecDeque;
use alloc::collections::{btree_map, BTreeMap};
use core::borrow::Borrow;
use core::fmt;
use core::iter::DoubleEndedIterator;
use core::iter::ExactSizeIterator;
use core::iter::FromIterator;
use core::iter::FusedIterator;
use core::mem::replace;
use core::ops::{Index, IndexMut};

///Double-ended queue with Map feature.
///
///DequeBTreeMap is a data structure that combines the functionality of a double-ended queue
///(Deque) and a map. It allows you to insert and remove key-value pairs from either end of
///the queue in a constant time, and provides map-like access to the values by their keys.
///
///The implementation of DequeBTreeMap uses a BTreeMap to store the entries, and a VecDeque to
///store the indices in the order they were added to the map. This allows DequeBTreeMap to
///provide efficient O(log n) insertion, removal, and access operations. It also implements
///many common traits, such as Default, PartialEq, PartialOrd, Clone, and Debug.
///
///DequeBTreeMap provides several methods for inserting and removing key-value pairs. The
///insert() method inserts a key-value pair into the map, and returns the old value if the
///key was already present. The push_back() and push_front() methods insert a key-value pair
///at the back or front of the queue, respectively, and return the old value if the key was
///already present.
///
///DequeBTreeMap also provides the entry() method, which returns an Entry enum that represents
///either a vacant or occupied entry in the map. This can be used to insert or update values
///in the map while also managing the indices in the queue.
///
///In addition, DequeBTreeMap provides methods for accessing and iterating over the entries in
///the map. The get() and get_mut() methods allow you to retrieve a reference to the value
///associated with a key, and the iter() and into_iter() methods return iterators over the
///entries in the map. DequeBTreeMap also implements the Index and IndexMut traits, which allow
///you to access and modify the values in the map using index syntax (e.g., map[key]).
///
///Overall, DequeBTreeMap is a useful data structure for situations where you need to maintain
///the insertion order of the entries while also providing efficient access to the values by
///their keys.
///
///One potential limitation of DequeBTreeMap is that it is not optimized for processing large
///batches of data with many duplicates. This is because the insert() method has a
///worst-case time complexity of O(n), where n is the number of entries in the map. This
///means that if you try to insert a large number of duplicate keys into the map, the
///performance may degrade significantly.
///
///Additionally, DequeBTreeMap uses a BTreeMap internally, which means that the keys must
///implement the Ord trait. This means that the keys must have a total order and must be
///comparable using the <, >, <=, and >= operators. This may not always be desirable,
///depending on the types of keys you need to use with DequeBTreeMap.
///
///Overall, while DequeBTreeMap is a useful data structure in many cases, it is important to
///consider its performance and limitations when deciding whether to use it in your own code.
///
/// When the element is present, the maximum time complexity is O(n). So it is not suitable for
/// processing large batches of data with too many duplicates.
///
/// Here are some examples of using DequeBTreeMap in Rust code:
///
///```
///// Create a new, empty DequeBTreeMap
///use dequemap::DequeBTreeMap;
///let mut map: DequeBTreeMap<String, i32> = DequeBTreeMap::new();
///
///// Insert a key-value pair at the back of the queue
///map.push_back("foo".to_string(), 42);
///
///// Insert another key-value pair at the front of the queue
///map.push_front("bar".to_string(), -1);
///
///// Insert a key-value pair into the map
///let old_value = map.insert("baz".to_string(), 123);
///
///// Get a reference to the value associated with a key
///let value = map.get("baz");
///
///// Iterate over the entries in the map
///for (key, value) in map.iter() {
///println!("{}: {}", key, value);
///}
///```
///
///The above content and some comments in the code are written by ChatGPT.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DequeBTreeMap<K, V> {
    entries: BTreeMap<K, V>,
    indices: VecDeque<K>,
}

impl<K, V> DequeBTreeMap<K, V> {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            indices: VecDeque::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: BTreeMap::default(),
            indices: VecDeque::with_capacity(capacity),
        }
    }
}

impl<K, V> Default for DequeBTreeMap<K, V> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::default(),
            indices: VecDeque::default(),
        }
    }
}

impl<K, V> DequeBTreeMap<K, V>
where
    K: Clone + Ord,
{
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical.
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some(v) = self.entries.get_mut(&key) {
            Some(replace(v, value))
        } else {
            self.entries.insert(key.clone(), value);
            self.indices.push_back(key);
            None
        }
    }

    #[inline]
    pub fn push_back(&mut self, key: K, value: V) -> Option<V> {
        let old_val = self.remove_entry(&key);
        self.entries.insert(key.clone(), value);
        self.indices.push_back(key);
        old_val
    }

    #[inline]
    pub fn push_front(&mut self, key: K, value: V) -> Option<V> {
        let old_val = self.remove_entry(&key);
        self.entries.insert(key.clone(), value);
        self.indices.push_front(key);
        old_val
    }

    #[inline]
    pub fn entry(&mut self, key: K) -> Entry<K, V>
    where
        K: Ord,
    {
        match self.entries.entry(key) {
            btree_map::Entry::Vacant(entry) => Entry::Vacant(VacantEntry {
                vacant: entry,
                indices: &mut self.indices,
            }),
            btree_map::Entry::Occupied(entry) => Entry::Occupied(OccupiedEntry { occupied: entry }),
        }
    }

    #[inline]
    fn remove_entry(&mut self, key: &K) -> Option<V> {
        if let Some(old_val) = self.entries.remove(key) {
            self.remove_from_index(key);
            Some(old_val)
        } else {
            None
        }
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.indices.shrink_to_fit();
    }

    #[inline]
    pub fn capacity(&mut self) -> usize {
        self.indices.capacity()
    }
}

impl<K, V> DequeBTreeMap<K, V> {
    /// Reserves capacity for at least additional more elements to be inserted in the given VecDeque.
    /// The collection may reserve more space to avoid frequent reallocations.
    pub fn reserve(&mut self, additional: usize) {
        self.indices.reserve(additional);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.indices.clear();
        self.entries.clear();
    }

    #[inline]
    pub fn remove(&mut self, k: &K) -> Option<V>
    where
        K: Ord,
    {
        if let Some(old_val) = self.entries.remove(k) {
            self.remove_from_index(k);
            Some(old_val)
        } else {
            None
        }
    }

    #[inline]
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.entries.get(k)
    }

    #[inline]
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.entries.get_key_value(key)
    }

    #[inline]
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.entries.get_mut(k)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            inner: self.indices.iter(),
            entries: &self.entries,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    #[inline]
    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.entries.contains_key(k)
    }

    #[inline]
    pub fn front(&self) -> Option<(&K, &V)>
    where
        K: Ord,
    {
        if self.is_empty() {
            return None;
        }
        if let Some(k) = self.indices.front() {
            self.entries.get(k).map(|v| (k, v))
        } else {
            None
        }
    }

    #[inline]
    pub fn pop_front(&mut self) -> Option<(K, V)>
    where
        K: Ord,
    {
        if let Some(k) = self.indices.pop_front() {
            self.entries.remove(&k).map(|v| (k, v))
        } else {
            None
        }
    }

    #[inline]
    pub fn back(&self) -> Option<(&K, &V)>
    where
        K: Ord,
    {
        if self.is_empty() {
            return None;
        }
        if let Some(k) = self.indices.back() {
            self.entries.get(k).map(|v| (k, v))
        } else {
            None
        }
    }

    #[inline]
    pub fn pop_back(&mut self) -> Option<(K, V)>
    where
        K: Ord,
    {
        if let Some(k) = self.indices.pop_back() {
            self.entries.remove(&k).map(|v| (k, v))
        } else {
            None
        }
    }

    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
    where
        K: Ord + Clone,
        F: FnMut(&K, &mut V) -> bool,
    {
        let mut removeds = BTreeSet::new();
        self.entries.retain(|k, v| {
            if f(k, v) {
                true
            } else {
                removeds.insert(k.clone());
                false
            }
        });
        self.indices.retain(|k| !removeds.contains(k))
    }

    #[inline]
    fn get_index(&self, k: &K) -> Option<usize>
    where
        K: Ord,
    {
        self.indices
            .iter()
            .enumerate()
            .find(|(_, x)| *x == k)
            .map(|(idx, _)| idx)
    }

    #[inline]
    fn remove_from_index(&mut self, k: &K) -> Option<K>
    where
        K: Ord,
    {
        if let Some(idx) = self.get_index(k) {
            self.indices.remove(idx)
        } else {
            None
        }
    }
}

impl<'a, K, Q, V> Index<&'a Q> for DequeBTreeMap<K, V>
where
    K: Borrow<Q> + Ord,
    Q: Ord,
{
    type Output = V;

    fn index(&self, key: &'a Q) -> &Self::Output {
        self.get(key).expect("no entry found for key")
    }
}

impl<K: Ord, V> Index<usize> for DequeBTreeMap<K, V> {
    type Output = V;

    fn index(&self, index: usize) -> &Self::Output {
        let key = self
            .indices
            .get(index)
            .expect("DequeBTreeMap: index out of bounds");
        self.entries
            .get(key)
            .expect("DequeBTreeMap: index out of bounds")
    }
}

impl<K: Ord, V> IndexMut<usize> for DequeBTreeMap<K, V> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let key = self
            .indices
            .get(index)
            .expect("DequeBTreeMap: index out of bounds");
        self.entries
            .get_mut(key)
            .expect("DequeBTreeMap: index out of bounds")
    }
}

impl<K, V> IntoIterator for DequeBTreeMap<K, V>
where
    K: Ord,
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.indices.into_iter(),
            entries: self.entries,
        }
    }
}

impl<'a, K, V> Extend<(&'a K, &'a V)> for DequeBTreeMap<K, V>
where
    K: Ord + Copy,
    V: Copy,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (&'a K, &'a V)>,
    {
        for (k, v) in iter {
            self.insert(*k, *v);
        }
    }
}

impl<K, V> Extend<(K, V)> for DequeBTreeMap<K, V>
where
    K: Ord + Clone,
{
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<K, V> FromIterator<(K, V)> for DequeBTreeMap<K, V>
where
    K: Ord + Clone,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        let mut map = DequeBTreeMap::new();
        map.extend(iter);
        map
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for DequeBTreeMap<K, V>
where
    K: Ord + Clone,
{
    fn from(items: [(K, V); N]) -> Self {
        let mut map = DequeBTreeMap::new();
        map.extend(items);
        map
    }
}

impl<'a, K: Ord, V> IntoIterator for &'a DequeBTreeMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Clone)]
pub struct Iter<'a, K, V> {
    inner: DequeIter<'a, K>,
    entries: &'a BTreeMap<K, V>,
}

impl<'a, K: Ord, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(k) = self.inner.next() {
            self.entries.get(k).map(|v| (k, v))
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.inner.count()
    }
}

impl<K: Ord, V> DoubleEndedIterator for Iter<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(k) = self.inner.next_back() {
            self.entries.get(k).map(|v| (k, v))
        } else {
            None
        }
    }
}

impl<K: Ord, V> ExactSizeIterator for Iter<'_, K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K: Ord, V> FusedIterator for Iter<'_, K, V> {}

pub struct IntoIter<K, V> {
    inner: DequeIntoIter<K>,
    entries: BTreeMap<K, V>,
}

impl<K: Ord, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(k) = self.inner.next() {
            self.entries.remove(&k).map(|v| (k, v))
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.inner.count()
    }
}

impl<K: Ord, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(k) = self.inner.next_back() {
            self.entries.remove(&k).map(|v| (k, v))
        } else {
            None
        }
    }
}

impl<K: Ord, V> ExactSizeIterator for IntoIter<K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K: Ord, V> FusedIterator for IntoIter<K, V> {}

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`DequeBTreeMap`].
///
/// [`entry`]: DequeBTreeMap::entry
pub enum Entry<'a, K, V> {
    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V>),
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V>),
}

impl<'a, K: Ord, V> Entry<'a, K, V> {
    /// Ensures a value is in the entry by inserting the default if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_insert(self, default: V) -> &'a mut V
    where
        K: Clone,
    {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result
    /// of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V
    where
        K: Clone,
    {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(default()),
        }
    }

    /// Ensures a value is in the entry by inserting,
    /// if empty, the result of the default function.
    ///
    /// This method allows for generating key-derived values for
    /// insertion by providing the default function a reference
    /// to the key that was moved during the `.entry(key)` method call.
    ///
    /// The reference to the moved key is provided
    /// so that cloning or copying the key is
    /// unnecessary, unlike with `.or_insert_with(|| ... )`.
    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V
    where
        K: Clone,
    {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            }
        }
    }

    /// Returns a reference to this entry’s key.
    pub fn key(&self) -> &K {
        match *self {
            Self::Occupied(ref entry) => entry.key(),
            Self::Vacant(ref entry) => entry.key(),
        }
    }

    /// Provides in-place mutable access to an occupied entry
    /// before any potential inserts into the map.
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Self::Occupied(mut entry) => {
                f(entry.get_mut());
                Self::Occupied(entry)
            }
            Self::Vacant(entry) => Self::Vacant(entry),
        }
    }
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Ord + Clone,
    V: Default,
{
    /// Ensures a value is in the entry by inserting the default value if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_default(self) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(Default::default()),
        }
    }
}

impl<K, V> fmt::Debug for Entry<'_, K, V>
where
    K: fmt::Debug + Ord,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entry::Vacant(entry) => entry.fmt(f),
            Entry::Occupied(entry) => entry.fmt(f),
        }
    }
}

/// A view into a vacant entry in an [`DequeBTreeMap`]. It is part of the [`Entry`] `enum`.
pub struct VacantEntry<'a, K, V> {
    /// The underlying vacant entry.
    vacant: btree_map::VacantEntry<'a, K, V>,
    /// The vector that stores all slots.
    indices: &'a mut VecDeque<K>,
}

impl<'a, K, V> VacantEntry<'a, K, V>
where
    K: Ord,
{
    /// Gets a reference to the key that would be used when inserting a value through the VacantEntry.
    pub fn key(&self) -> &K {
        self.vacant.key()
    }

    /// Take ownership of the key.
    pub fn into_key(self) -> K {
        self.vacant.into_key()
    }

    /// Sets the value of the entry with the `VacantEntry`’s key,
    /// and returns a mutable reference to it.
    pub fn insert(self, value: V) -> &'a mut V
    where
        K: Clone,
    {
        self.indices.push_back(self.vacant.key().clone());
        self.vacant.insert(value)
    }
}

impl<K, V> fmt::Debug for VacantEntry<'_, K, V>
where
    K: fmt::Debug + Ord,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VacantEntry")
            .field("key", self.key())
            .finish()
    }
}

/// A view into an occupied entry in a [`DequeBTreeMap`]. It is part of the [`Entry`] `enum`.
pub struct OccupiedEntry<'a, K, V> {
    /// The underlying occupied entry.
    occupied: btree_map::OccupiedEntry<'a, K, V>,
}

impl<'a, K, V> OccupiedEntry<'a, K, V>
where
    K: Ord,
{
    /// Gets a reference to the key in the entry.
    pub fn key(&self) -> &K {
        self.occupied.key()
    }

    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &V {
        self.occupied.get()
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` that may outlive the
    /// destruction of the `Entry` value, see [`into_mut`].
    ///
    /// [`into_mut`]: OccupiedEntry::into_mut
    pub fn get_mut(&mut self) -> &mut V {
        self.occupied.get_mut()
    }

    /// Converts the entry into a mutable reference to its value.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
    ///
    /// [`get_mut`]: OccupiedEntry::get_mut
    pub fn into_mut(self) -> &'a mut V {
        self.occupied.into_mut()
    }

    /// Sets the value of the entry with the `OccupiedEntry`’s key,
    /// and returns the entry’s old value.
    pub fn insert(&mut self, value: V) -> V
    where
        K: Clone,
    {
        replace(self.occupied.get_mut(), value)
    }
}

impl<K, V> fmt::Debug for OccupiedEntry<'_, K, V>
where
    K: fmt::Debug + Ord,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OccupiedEntry")
            .field("key", self.key())
            .field("value", self.get())
            .finish()
    }
}

#[cfg(feature = "serde")]
impl<K, V> serde::ser::Serialize for DequeBTreeMap<K, V>
where
    K: serde::ser::Serialize + Ord,
    V: serde::ser::Serialize,
{
    fn serialize<T>(&self, serializer: T) -> Result<T::Ok, T::Error>
    where
        T: serde::ser::Serializer,
    {
        serializer.collect_map(self)
    }
}

#[cfg(feature = "serde")]
struct DequeBTreeMapVisitor<K, V>(core::marker::PhantomData<(K, V)>);

#[cfg(feature = "serde")]
impl<'de, K, V> serde::de::Visitor<'de> for DequeBTreeMapVisitor<K, V>
where
    K: serde::de::Deserialize<'de> + Ord + Clone,
    V: serde::de::Deserialize<'de>,
{
    type Value = DequeBTreeMap<K, V>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "a map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut values = DequeBTreeMap::with_capacity(map.size_hint().unwrap_or(0));
        while let Some((key, value)) = map.next_entry()? {
            values.insert(key, value);
        }
        Ok(values)
    }
}

/// Requires crate feature `"serde"`
#[cfg(feature = "serde")]
impl<'de, K, V> serde::de::Deserialize<'de> for DequeBTreeMap<K, V>
where
    K: serde::de::Deserialize<'de> + Ord + Clone,
    V: serde::de::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_map(DequeBTreeMapVisitor(core::marker::PhantomData))
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V, E> serde::de::IntoDeserializer<'de, E> for DequeBTreeMap<K, V>
where
    K: serde::de::IntoDeserializer<'de, E> + Ord,
    V: serde::de::IntoDeserializer<'de, E>,
    E: serde::de::Error,
{
    type Deserializer = serde::de::value::MapDeserializer<'de, <Self as IntoIterator>::IntoIter, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        serde::de::value::MapDeserializer::new(self.into_iter())
    }
}

#[cfg(feature = "serde")]
#[test]
fn test_dequebtreemap_serde() {
    use alloc::vec::Vec;
    let to_vec = |map: &DequeBTreeMap<i32, i32>| {
        map.iter()
            .map(|t| (*t.0, *t.1))
            .collect::<Vec<(i32, i32)>>()
    };

    let mut map = DequeBTreeMap::new();
    map.push_back(2, 20);
    map.push_back(1, 10);
    map.push_back(9, 90);
    map.push_back(3, 30);
    map.push_back(5, 50);

    assert_eq!(to_vec(&map), [(2, 20), (1, 10), (9, 90), (3, 30), (5, 50)]);

    let data = bincode::serialize(&map).unwrap();
    let map: DequeBTreeMap<i32, i32> = bincode::deserialize(&data).unwrap();
    assert_eq!(to_vec(&map), [(2, 20), (1, 10), (9, 90), (3, 30), (5, 50)]);
}

#[test]
fn test_insert() {
    use alloc::vec::Vec;
    let to_vec = |map: &DequeBTreeMap<i32, i32>| {
        map.iter()
            .map(|t| (*t.0, *t.1))
            .collect::<Vec<(i32, i32)>>()
    };

    let mut map = DequeBTreeMap::new();
    map.insert(2, 20);
    map.insert(1, 10);
    map.insert(9, 90);
    assert_eq!(to_vec(&map), [(2, 20), (1, 10), (9, 90)]);

    map.insert(7, 70);
    map.insert(1, 100);
    assert_eq!(to_vec(&map), [(2, 20), (1, 100), (9, 90), (7, 70)]);

    assert_eq!(map.entries.len(), map.indices.len());

    assert_eq!(map.pop_front(), Some((2, 20)));
    assert_eq!(map.pop_back(), Some((7, 70)));
    assert_eq!(to_vec(&map), [(1, 100), (9, 90)]);

    map.insert(3, 30);
    map.insert(7, 70);
    map.insert(9, 900);
    map.push_back(1, 10);
    assert_eq!(to_vec(&map), [(9, 900), (3, 30), (7, 70), (1, 10)]);
    assert_eq!(map.entries.len(), map.indices.len());
}

#[test]
fn test_entry() {
    use alloc::vec::Vec;
    let to_vec = |map: &DequeBTreeMap<i32, i32>| {
        map.iter()
            .map(|t| (*t.0, *t.1))
            .collect::<Vec<(i32, i32)>>()
    };

    let mut map = DequeBTreeMap::new();
    map.entry(2).or_insert(20);
    map.entry(1).or_insert(10);
    map.entry(9).or_insert(90);
    map.entry(3).or_insert(30);
    map.entry(5).or_insert(50);
    assert_eq!(map.get(&1), Some(&10));
    assert_eq!(map.get(&2), Some(&20));
    assert_eq!(map.get(&3), Some(&30));
    assert_eq!(map.get(&5), Some(&50));
    assert_eq!(map.get(&9), Some(&90));

    assert_eq!(to_vec(&map), [(2, 20), (1, 10), (9, 90), (3, 30), (5, 50)]);
    assert_eq!(map.entries.len(), map.indices.len());

    map.entry(3).and_modify(|v| *v = 300);

    assert_eq!(to_vec(&map), [(2, 20), (1, 10), (9, 90), (3, 300), (5, 50)]);
    assert_eq!(map.entries.len(), map.indices.len());

    map.entry(7).or_insert_with(|| 70);
    assert_eq!(
        to_vec(&map),
        [(2, 20), (1, 10), (9, 90), (3, 300), (5, 50), (7, 70)]
    );
    assert_eq!(map.entries.len(), map.indices.len());
}

#[test]
fn test_dequemap() {
    use alloc::vec::Vec;
    let to_vec = |map: &DequeBTreeMap<i32, i32>| {
        map.iter()
            .map(|t| (*t.0, *t.1))
            .collect::<Vec<(i32, i32)>>()
    };

    let mut map = DequeBTreeMap::new();
    map.push_back(2, 20);
    map.push_back(1, 10);
    map.push_back(9, 90);
    map.push_back(3, 30);
    map.push_back(5, 50);
    assert_eq!(map.get(&1), Some(&10));
    assert_eq!(map.get(&2), Some(&20));
    assert_eq!(map.get(&3), Some(&30));
    assert_eq!(map.get(&5), Some(&50));
    assert_eq!(map.get(&9), Some(&90));
    assert_eq!(map.len(), 5);
    assert_eq!(map.pop_front(), Some((2, 20)));
    assert_eq!(map.len(), 4);
    assert_eq!(map.pop_back(), Some((5, 50)));
    assert_eq!(map.len(), 3);
    assert_eq!(to_vec(&map), [(1, 10), (9, 90), (3, 30)]);
    assert_eq!(map.entries.len(), map.indices.len());

    let mut map1: DequeBTreeMap<i32, i32> = DequeBTreeMap::new();
    map1.push_back(7, 70);
    map1.push_back(9, 900);
    map.extend(map1);
    assert_eq!(to_vec(&map), [(1, 10), (9, 900), (3, 30), (7, 70)]);
    assert_eq!(map.entries.len(), map.indices.len());

    assert_eq!(map.front(), Some((&1, &10)));
    assert_eq!(map.back(), Some((&7, &70)));

    assert_eq!(to_vec(&map), [(1, 10), (9, 900), (3, 30), (7, 70)]);
    assert_eq!(map.entries.len(), map.indices.len());

    map.remove(&3);
    assert_eq!(to_vec(&map), [(1, 10), (9, 900), (7, 70)]);
    assert_eq!(map.entries.len(), map.indices.len());
}

#[test]
fn test_dequemap_index() {
    let mut map = DequeBTreeMap::new();
    map.push_back(2, 20);
    map.push_back(1, 10);
    map.push_back(9, 90);
    assert_eq!(map.index_mut(1), &mut 10);
    assert_eq!(map.index(2), &90);
}

#[test]
fn test_dequemap_extend() {
    use alloc::vec::Vec;
    let to_vec = |map: &DequeBTreeMap<i32, i32>| {
        map.iter()
            .map(|t| (*t.0, *t.1))
            .collect::<Vec<(i32, i32)>>()
    };
    let mut map = DequeBTreeMap::new();
    map.push_back(2, 20);
    map.push_back(1, 10);
    map.push_back(9, 90);
    map.extend([(10, 100), (5, 50)]);
    assert_eq!(
        to_vec(&map),
        [(2, 20), (1, 10), (9, 90), (10, 100), (5, 50)]
    );
    assert_eq!(map.entries.len(), map.indices.len());
}

#[test]
fn test_dequemap_retain() {
    let mut map = DequeBTreeMap::new();
    map.push_back(2, 20);
    map.push_back(1, 10);
    map.push_back(9, 90);
    map.extend([(10, 100), (5, 50)]);

    assert_eq!(map.entries.len(), map.indices.len());
    assert_eq!(map.entries.len(), 5);

    map.retain(|k, _| *k != 10 && *k != 2);

    assert_eq!(map.entries.len(), map.indices.len());
    assert_eq!(map.entries.len(), 3);
}
