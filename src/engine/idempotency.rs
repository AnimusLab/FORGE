use std::collections::HashSet;

pub struct IdempotencyStore {
    seen: HashSet<String>,
}

impl IdempotencyStore {
    pub fn new() -> Self {
        Self { seen: HashSet::new() }
    }

    // Returns true if new request, false if duplicate
    pub fn check_and_insert(&mut self, request_id: &str) -> bool {
        if self.seen.contains(request_id) {
            false
        } else {
            self.seen.insert(request_id.to_string());
            true
        }
    }
}