pub trait Children {
    type ChildrenRef<'a>
    where
        Self: 'a;
    type ChildrenRefMut<'a>
    where
        Self: 'a;

    fn children(&self) -> Self::ChildrenRef<'_>;
    fn children_mut(&mut self) -> Self::ChildrenRefMut<'_>;
}
