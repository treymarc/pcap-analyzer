use std::collections::HashMap;

/// Defragmentation engine
pub trait DefragEngine {
    /// This function updates the engine with a new Fragment
    /// Returns a Fragment describing the defragmentation operation result
    fn update<'a>(&mut self, id:u16, offset:usize, more_fragments:bool, frag:&'a[u8]) -> Fragment<'a>;
}

pub enum Fragment<'a> {
    /// Data is not fragmented - return original slice
    NoFrag(&'a[u8]),
    /// Data was defragmented - return buffer
    Complete(Vec<u8>),
    /// Fragment is part of a (yet) unfinished buffer
    Incomplete,
    /// Defragmentation error
    Error,
}




// pub struct IP4Fragment {
// }

pub struct IP4DefragEngine {
    // XXX we need to store all fragments, with offsets
    // XXX index this by 3-tuple ?
    ipv4_fragments: HashMap<u16,Vec<u8>>,
}

impl IP4DefragEngine {
    pub fn new() -> IP4DefragEngine {
        IP4DefragEngine{
            ipv4_fragments: HashMap::new(),
        }
    }
}

impl DefragEngine for IP4DefragEngine {
        fn update<'a>(&mut self, id:u16, frag_offset:usize, more_fragments:bool, frag:&'a[u8]) -> Fragment<'a> {
            if more_fragments == false {
                if frag_offset == 0 { Fragment::NoFrag(frag) }
                else {
                    // This is the last fragment
                    match self.ipv4_fragments.remove(&id) {
                        None    => { warn!("could not get first fragment buffer for ID {}", id); Fragment::Error },
                        Some(mut f) => {
                            // reassembly strategy: last frag wins
                            if frag_offset < f.len() {
                                warn!("overlapping fragment frag_offset {}, keep_f.len={}", frag_offset, f.len());
                                f.truncate(frag_offset);
                            }
                            else if frag_offset > f.len() {
                                warn!("missed fragment frag_offset {}, keep_f.len={}", frag_offset, f.len());
                                f.resize(frag_offset, 0xff);
                            }
                            f.extend_from_slice(frag);
                            Fragment::Complete(f)
                        }
                    }
                }
            } else {
                // Fragment is part of a larger buffer
                debug!("more fragments {}", id);
                if frag_offset == 0 {
                    // first fragment
                    debug!("first fragment");
                    // XXX if keep_f.len() != 0 we already received a fragment 0
                    let v = frag.to_vec();
                    warn!("inserting defrag buffer key={} len={}", id, frag.len());
                    // insert ipv4 *data* but keep ipv4 header for the first packet
                    if self.ipv4_fragments.contains_key(&id) {
                        warn!("IPv4 defrag collision for id {}", id)
                    }
                    self.ipv4_fragments.insert(id, v);
                } else {
                    match self.ipv4_fragments.get_mut(&id) {
                        Some(f) => {
                            // reassembly strategy: last frag wins
                            if frag_offset < f.len() {
                                warn!("overlapping fragment frag_offset {}, keep_f.len={}", frag_offset, f.len());
                                f.truncate(frag_offset);
                            }
                            else if frag_offset > f.len() {
                                warn!("missed fragment frag_offset {}, keep_f.len={}", frag_offset, f.len());
                                f.resize(frag_offset, 0xff);
                            }
                            f.extend_from_slice(frag)
                        },
                        None    => warn!("could not get first fragment buffer for ID {}", id),
                    }
                }
                Fragment::Incomplete
            }
        }
}