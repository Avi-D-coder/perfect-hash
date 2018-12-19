use std::cmp::{Ord, Ordering::*};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use nohash_hasher::{BuildNoHashHasher, IntMap};

macro_rules! PerfectHasher {
    ($name:ident, $size:ty) => {
        pub struct $name<C, H> {
            // Key is the Id
            alloted: IntMap<$size, C>,
            hasher: H,
        }

        impl<C, H> $name<C, H>
        where
            H: Hasher,
            C: Hash + Ord,
        {
            pub fn new(hasher: H) -> Self {
                $name {
                    alloted: IntMap::default(),
                    hasher,
                }
            }

            pub fn with_capacity(capacity: $size, hasher: H) -> Self {
                $name {
                    alloted: HashMap::with_capacity_and_hasher(
                        capacity as usize,
                        BuildNoHashHasher::default(),
                    ),
                    hasher,
                }
            }

            pub fn unique_id(&mut self, content: C) -> $size {
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
                            return hash;
                        }
                        Less => hash = hash.wrapping_sub(1),
                        Greater => hash = hash.wrapping_add(1),
                    }
                }
            }

            pub fn dissociate(&mut self, id: $size) {
                self.alloted.remove(&id);
            }
        }
    };
}

PerfectHasher!(PerfectHasher8, u8);
PerfectHasher!(PerfectHasher16, u16);
PerfectHasher!(PerfectHasher32, u32);
PerfectHasher!(PerfectHasher64, u64);
PerfectHasher!(PerfectHasher, usize);

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
        let mut ph: PerfectHasher<char, CollideHasher> = PerfectHasher::new(CollideHasher);
        assert_eq!(0, ph.unique_id('a'));
        assert_eq!(1, ph.unique_id('b'));
    }

    #[test]
    fn collision_wrap() {
        let mut ph: PerfectHasher<char, CollideHasher> = PerfectHasher::new(CollideHasher);
        assert_eq!(0, ph.unique_id('b'));
        assert_eq!(usize::max_value(), ph.unique_id('a'));
    }
}
