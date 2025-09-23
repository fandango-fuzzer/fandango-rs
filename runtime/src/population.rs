//! Population primitives for evolutionary algorithms

use crate::measurement::HasMeasurement;

/// An individual within a population
pub trait Individual: HasMeasurement {
    /// The node contained by this individual
    type Node;

    /// Get the node
    fn node(&self) -> &Self::Node;
    /// Get the node, mutably
    fn node_mut(&mut self) -> &mut Self::Node;
}
