pub trait Individual {
    type Node;

    fn node(&self) -> &Self::Node;
    fn node_mut(&mut self) -> &mut Self::Node;
}
