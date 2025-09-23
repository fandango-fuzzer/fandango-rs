use crate::measurement::HasMeasurement;

pub trait Individual: HasMeasurement {
    type Node;

    fn node(&self) -> &Self::Node;
    fn node_mut(&mut self) -> &mut Self::Node;
}
