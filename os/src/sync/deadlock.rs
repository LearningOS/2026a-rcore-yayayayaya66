//! Per-process resource tracking for optional deadlock detection.
use alloc::collections::{BTreeMap, BTreeSet};

/// Resource allocation and pending requests.
pub struct DeadlockDetector {
    /// Whether unsafe requests are rejected.
    pub enabled: bool,
    available: BTreeMap<usize, usize>,
    held: BTreeMap<(usize, usize), usize>,
    waiting: BTreeMap<usize, usize>,
}
impl DeadlockDetector {
    /// Create an empty detector.
    pub fn new() -> Self { Self { enabled: false, available: BTreeMap::new(), held: BTreeMap::new(), waiting: BTreeMap::new() } }
    /// Register a resource and its initial quantity.
    pub fn add(&mut self, resource: usize, count: usize) { self.available.insert(resource, count); }
    /// Register a request and return whether it is safe to proceed.
    pub fn request(&mut self, tid: usize, resource: usize) -> bool {
        self.waiting.insert(tid, resource);
        if !self.enabled { return true; }
        let mut work = self.available.clone();
        let mut threads = BTreeSet::new();
        for &(thread, _) in self.held.keys() { threads.insert(thread); }
        for &thread in self.waiting.keys() { threads.insert(thread); }
        loop {
            let next = threads.iter().copied().find(|thread| match self.waiting.get(thread) {
                None => true,
                Some(resource) => work.get(resource).copied().unwrap_or(0) > 0,
            });
            match next {
                Some(thread) => {
                    threads.remove(&thread);
                    for (&(owner, resource), &count) in self.held.iter() {
                        if owner == thread { *work.entry(resource).or_insert(0) += count; }
                    }
                }
                None => break,
            }
        }
        if threads.contains(&tid) { self.waiting.remove(&tid); false } else { true }
    }
    /// Record a successfully acquired resource.
    pub fn acquired(&mut self, tid: usize, resource: usize) {
        self.waiting.remove(&tid);
        if let Some(n) = self.available.get_mut(&resource) { *n = n.saturating_sub(1); }
        *self.held.entry((tid, resource)).or_insert(0) += 1;
    }
    /// Record release, including semaphore signals from a different thread.
    pub fn release(&mut self, tid: usize, resource: usize) {
        if let Some(n) = self.held.get_mut(&(tid, resource)) { *n = n.saturating_sub(1); }
        *self.available.entry(resource).or_insert(0) += 1;
    }
}
