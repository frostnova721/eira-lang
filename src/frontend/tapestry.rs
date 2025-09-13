
#[derive(Debug,Clone, Copy, PartialEq)]
pub struct Tapestry(pub u64);

impl Tapestry {
    pub const fn new(strands: u64) -> Self {
        Tapestry { 0: strands }
    }
    // Check for strand presence
    pub fn has_strand(&self, strand: u64) -> bool {
        (self.0 & strand) != 0
    }

    // Add a strand to a weave
    pub fn weave(&mut self, strand: u64) {
        self.0 |= strand;
    }

    // Remove a strand from weave
    pub fn unweave(&mut self, strand: u64) {
        self.0 &= !strand
    }
}