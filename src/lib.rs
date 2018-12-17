use std::cmp::{Ord, Ordering::*};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use nohash_hasher::{BuildNoHashHasher, IntMap};

pub struct PerfectHasher<Id, C, H> {
    // Key is the Id
    alloted: IntMap<Id, C>,
    hasher: H,
}

macro_rules! PerfectHasher {
    ($size:ty) => {
        impl<C, H> PerfectHasher<$size, C, H>
        where
            H: Hasher,
            C: Hash + Ord,
        {
            pub fn new(hasher: H) -> Self {
                PerfectHasher {
                    alloted: IntMap::default(),
                    hasher,
                }
            }

            pub fn with_capacity(capacity: $size, hasher: H) -> Self {
                PerfectHasher {
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

                let mut comparison = Equal;

                loop {
                    let entry = self
                        .alloted
                        .entry(hash)
                        .and_modify(|cached| comparison = (*cached).cmp(&content));

                    match comparison {
                        Equal => {
                            entry.or_insert(content);
                            return hash;
                        }
                        Less => hash += 1,
                        Greater => hash -= 1,
                    }
                }
            }

            pub fn dissociate(&mut self, id: $size) {
                self.alloted.remove(&id);
            }
        }
    };
}

PerfectHasher!(u8);
PerfectHasher!(u16);
PerfectHasher!(u32);
PerfectHasher!(u64);
PerfectHasher!(usize);
