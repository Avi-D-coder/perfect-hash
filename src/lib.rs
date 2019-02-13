use std::cmp::{Ord, Ordering::*};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Index;

use nohash_hasher::{BuildNoHashHasher, IntMap};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id<T> {
    key: T,
}

impl<T> Id<T> {
    fn new(key: T) -> Id<T> {
        Id { key }
    }
}

macro_rules! PerfectHasher {
    ($name:ident, $map_name:ident, $contents:ident, $size:ty) => {
        impl Into<$size> for Id<$size> {
            fn into(self) -> $size {
                self.key
            }
        }

        impl Into<$size> for &Id<$size> {
            fn into(self) -> $size {
                self.key
            }
        }

        pub struct $name<C, H> {
            // Key is the Id
            alloted: IntMap<$size, C>,
            hasher: H,
        }

        impl<C, H> Index<Id<$size>> for $name<C, H> {
            type Output = C;
            fn index(&self, id: Id<$size>) -> &C {
                &self.alloted[&id.into()]
            }
        }

        impl<C> $name<C, DefaultHasher> {
            /// If you are not worried about DDoS you should use a faster hasher.
            /// Rust defaults to SipHash which is more DDoS resistant, but very slow.
            /// I recommend the fasthash crate for a collection of faster hashing algorithms.
            /// You should use `with_hasher(hasher)` or `default()` when you don't need DDoS resistance.
            pub fn new() -> Self {
                $name {
                    alloted: IntMap::default(),
                    hasher: DefaultHasher::default(),
                }
            }
        }

        impl<C, H> Default for $name<C, H>
        where
            H: Hasher + Default,
            C: Hash + Ord,
        {
            fn default() -> $name<C, H> {
                $name::with_hasher(H::default())
            }
        }

        impl<C, H> $name<C, H>
        where
            H: Hasher,
            C: Hash + Ord,
        {
            pub fn with_hasher(hasher: H) -> Self {
                $name {
                    alloted: IntMap::default(),
                    hasher,
                }
            }

            pub fn with_capacity(capacity: $size) -> $name<C, DefaultHasher> {
                $name {
                    alloted: HashMap::with_capacity_and_hasher(
                        capacity as usize,
                        BuildNoHashHasher::default(),
                    ),
                    hasher: DefaultHasher::default(),
                }
            }

            pub fn with_capacity_and_hasher(capacity: $size, hasher: H) -> Self {
                $name {
                    alloted: HashMap::with_capacity_and_hasher(
                        capacity as usize,
                        BuildNoHashHasher::default(),
                    ),
                    hasher,
                }
            }

            pub fn unique_id(&mut self, content: C) -> Id<$size> {
                content.hash(&mut self.hasher);
                let mut hash = self.hasher.finish() as $size;

                loop {
                    let mut comparison = Equal;

                    let entry = self
                        .alloted
                        .entry(hash)
                        .and_modify(|cached| comparison = content.cmp(cached));

                    match comparison {
                        Equal => {
                            entry.or_insert(content);
                            return Id::new(hash);
                        }
                        Less => hash = hash.wrapping_sub(1),
                        Greater => hash = hash.wrapping_add(1),
                    }
                }
            }

            pub fn get(&self, id: $size) -> Option<&C> {
                self.alloted.get(&id)
            }

            pub fn dissociate(&mut self, id: Id<$size>) {
                self.alloted.remove(&id.into());
            }

            /// Returns an `Iterator` of content associated with the ids from `ids` `Iterator`.
            pub fn contents<'i, I>(&self, ids: I) -> $contents<I, C, H>
            where
                I: Iterator<Item = &'i Id<$size>>,
            {
                $contents { ids, ph: self }
            }
        }

        pub struct $contents<'l, I, C, H> {
            ids: I,
            ph: &'l $name<C, H>,
        }

        impl<'i, 'l, I, C, H> Iterator for $contents<'l, I, C, H>
        where
            I: Iterator<Item = &'i Id<$size>>,
        {
            type Item = &'l C;
            fn next(&mut self) -> Option<&'l C> {
                self.ids
                    .next()
                    .map_or(None, |id: &Id<$size>| self.ph.alloted.get(&id.into()))
            }
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.ids.size_hint()
            }
        }

        pub struct $map_name<C, H, T> {
            // Key is the Id
            alloted: IntMap<$size, (C, T)>,
            hasher: H,
        }

        impl<C, H, T> Index<Id<$size>> for $map_name<C, H, T> {
            type Output = (C, T);
            fn index(&self, id: Id<$size>) -> &(C, T) {
                &self.alloted[&id.into()]
            }
        }

        impl<C, T> $map_name<C, DefaultHasher, T> {
            /// If you are not worried about DDoS you should use a faster hasher.
            /// Rust defaults to SipHash which is more DDoS resistant, but very slow.
            /// I recommend the fasthash crate for a collection of faster hashing algorithms.
            /// You should use `with_hasher(hasher)` or `default()` when you don't need DDoS resistance.
            pub fn new() -> Self {
                $map_name {
                    alloted: IntMap::default(),
                    hasher: DefaultHasher::default(),
                }
            }
        }

        impl<C, H, T> Default for $map_name<C, H, T>
        where
            H: Hasher + Default,
            C: Hash + Ord,
        {
            fn default() -> $map_name<C, H, T> {
                $map_name::with_hasher(H::default())
            }
        }

        impl<C, H, T> $map_name<C, H, T>
        where
            H: Hasher,
            C: Hash + Ord,
        {
            pub fn with_hasher(hasher: H) -> Self {
                $map_name {
                    alloted: IntMap::default(),
                    hasher,
                }
            }

            pub fn with_capacity(capacity: $size) -> $map_name<C, DefaultHasher, T> {
                $map_name {
                    alloted: HashMap::with_capacity_and_hasher(
                        capacity as usize,
                        BuildNoHashHasher::default(),
                    ),
                    hasher: DefaultHasher::default(),
                }
            }

            pub fn with_capacity_and_hasher(capacity: $size, hasher: H) -> Self {
                $map_name {
                    alloted: HashMap::with_capacity_and_hasher(
                        capacity as usize,
                        BuildNoHashHasher::default(),
                    ),
                    hasher,
                }
            }

            pub fn unique_id<F>(&mut self, content: C, data: T, modify: F) -> Id<$size>
            where
                F: FnOnce(&mut T, &T),
            {
                content.hash(&mut self.hasher);
                let mut hash = self.hasher.finish() as $size;

                loop {
                    let mut comparison = Equal;

                    let entry = self
                        .alloted
                        .entry(hash)
                        .and_modify(|(cached, _)| comparison = content.cmp(cached));

                    match comparison {
                        Equal => {
                            entry
                                .and_modify(|(_, old_data)| modify(old_data, &data))
                                .or_insert((content, data));
                            return Id::new(hash);
                        }
                        Less => hash = hash.wrapping_sub(1),
                        Greater => hash = hash.wrapping_add(1),
                    }
                }
            }

            pub fn index_mut(&mut self, id: Id<$size>) -> (&C, &mut T) {
                self.get_mut(id.into()).unwrap()
            }

            pub fn get_mut(&mut self, id: $size) -> Option<(&C, &mut T)> {
                self.alloted
                    .get_mut(&id)
                    .map(|(content, data)| (&*content, data))
            }

            // TODO Implement entry API

            pub fn dissociate(&mut self, id: Id<$size>) {
                self.alloted.remove(&id.into());
            }

            // TODO /// Returns an `Iterator` of content associated with the ids from `ids` `Iterator`.
            // pub fn contents<I, P>(&self, ids: I) -> $contents<I, C, H>
            // where
            //     I: Iterator<Item = Id<$size>>,
            //     P: Iterator<Item = C>,
            // {
            //     $contents { ids, ph: self }
            // }
        }
    };
}

PerfectHasher!(PerfectHasher8, PerfectHashMap8, ContentsU8, u8);
PerfectHasher!(PerfectHasher16, PerfectHashMap16, ContentsU16, u16);
PerfectHasher!(PerfectHasher32, PerfectHashMap32, ContentsU32, u32);
PerfectHasher!(PerfectHasher64, PerfectHashMap64, ContentsU64, u64);
PerfectHasher!(PerfectHasher, PerfectHashMap, Contents, usize);

mod test {
    use super::*;

    /// A Hasher that always outputs `0` for testing purposes.
    #[allow(dead_code)]
    pub struct CollideHasher;

    impl Hasher for CollideHasher {
        fn write(&mut self, _: &[u8]) {}

        fn finish(&self) -> u64 {
            0
        }
    }

    #[test]
    fn collision_resilience() {
        let mut ph: PerfectHasher<char, CollideHasher> = PerfectHasher::with_hasher(CollideHasher);
        assert_eq!(Id::new(0), ph.unique_id('a'));
        assert_eq!(Id::new(1), ph.unique_id('b'));
    }

    #[test]
    fn collision_wrap() {
        let mut ph: PerfectHasher<char, CollideHasher> = PerfectHasher::with_hasher(CollideHasher);
        assert_eq!(Id::new(0), ph.unique_id('b'));
        assert_eq!(Id::new(usize::max_value()), ph.unique_id('a'));
    }

    #[test]
    fn dissociate() {
        let mut ph: PerfectHasher<char, CollideHasher> = PerfectHasher::with_hasher(CollideHasher);
        assert_eq!(Id::new(0), ph.unique_id('a'));
        ph.dissociate(Id::new(0));
        assert_eq!(Id::new(0), ph.unique_id('b'));
    }

    #[test]
    fn index() {
        let mut ph = PerfectHasher::with_hasher(DefaultHasher::default());
        let id = ph.unique_id(String::from("foo"));
        assert_eq!("foo", ph[id])
    }
}
