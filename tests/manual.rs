struct Simple;

const SOURCE: &'static str = "<start> ::= <expr>;\n<expr> ::= <number> \"+\" <expr> | <number>;\n<number> ::= \"0\" | <non_zero><digit>*;\n<non_zero> ::=\n              \"1\"\n            | \"2\"\n            | \"3\"\n            | \"4\"\n            | \"5\"\n            | \"6\"\n            | \"7\"\n            | \"8\"\n            | \"9\"\n            ;\n<digit> ::= \"0\" | <non_zero>;\n";
pub type ParseError = Box<::fandango::error::Error<Rule>>;
#[derive(Clone, Debug)]
pub struct nonterminal_start<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
    child_0: ::std::boxed::Box<nonterminal_expr<'source>>,
}
impl<'source> ::fandango::typing::Node for nonterminal_start<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'program nonterminal_expr<'source>, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'program mut nonterminal_expr<'source>, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        { (&self.child_0, ()) }
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        { (&mut self.child_0, ()) }
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_start<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_start<'source>) -> Type<'program, 'source> {
        Type::nonterminal_start(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_start<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_start<'source>) -> Type<'program, 'source> {
        Type::nonterminal_start(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_start<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_start<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_start(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_start<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        if ::core::cfg!(debug_assertions) {
            match (&(value.as_rule()), &Rule::start) {
                (left_val, right_val) => todo!(),
            };
        };
        let span = value.as_span();
        let (inner, _) = {
            let iter = &mut (value.into_inner());
            let out = (
                {
                    let tmp = iter.next().unwrap();
                    if cfg!(debug_assertions) {
                        let value = (tmp.as_rule());
                        #[allow(unreachable_patterns)]
                        match value {
                            Rule::expr => {}
                            _ => panic!(
                                "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                                stringify!(Rule::expr),
                                value
                            ),
                        }
                    }
                    tmp
                },
                {
                    let tmp = iter.next().unwrap();
                    if cfg!(debug_assertions) {
                        let value = (tmp.as_rule());
                        #[allow(unreachable_patterns)]
                        match value {
                            Rule::EOI => {}
                            _ => panic!(
                                "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                                stringify!(Rule::EOI),
                                value
                            ),
                        }
                    }
                    tmp
                },
            );
            if cfg!(debug_assertions) {
                let value = (iter.next());
                #[allow(unreachable_patterns)]
                match value {
                    Option::None => {}
                    _ => panic!(
                        "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                        stringify!(Option::None),
                        value
                    ),
                }
            }
            out
        };
        Ok(Self {
            child_0: nonterminal_expr::try_from((source.clone(), inner))?.into(),
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_expr<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
    child_0: nonterminal_expr_0<'source>,
}
impl<'source> ::fandango::typing::Node for nonterminal_expr<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'program nonterminal_expr_0<'source>, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'program mut nonterminal_expr_0<'source>, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&self.child_0, ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&mut self.child_0, ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_expr<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_expr<'source>) -> Type<'program, 'source> {
        Type::nonterminal_expr(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_expr<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_expr<'source>) -> Type<'program, 'source> {
        Type::nonterminal_expr(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_expr<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_expr<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_expr(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_expr<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        if ::core::cfg!(debug_assertions) {
            match (&(value.as_rule()), &Rule::expr) {
                (left_val, right_val) => todo!(),
            };
        };
        let span = value.as_span();
        let (child_0,) = {
            let iter = &mut (value.into_inner());
            let out = ({
                let tmp = iter.next().unwrap();
                if cfg!(debug_assertions) {
                    let value = (tmp.as_rule());
                    #[allow(unreachable_patterns)]
                    match value {
                        expr_0 => {}
                        _ => panic!(
                            "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                            stringify!(expr_0),
                            value
                        ),
                    }
                }
                tmp
            },);
            if cfg!(debug_assertions) {
                let value = (iter.next());
                #[allow(unreachable_patterns)]
                match value {
                    Option::None => {}
                    _ => panic!(
                        "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                        stringify!(Option::None),
                        value
                    ),
                }
            }
            out
        };
        Ok(Self {
            child_0: nonterminal_expr_0::try_from((source.clone(), child_0))?.into(),
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub enum nonterminal_expr_0<'source> {
    variant_0(nonterminal_expr_0_0<'source>),
    variant_1(::std::boxed::Box<nonterminal_number<'source>>),
}
impl<'source> ::fandango::typing::Node for nonterminal_expr_0<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (
        Option<&'program nonterminal_expr_0_0<'source>>,
        (Option<&'program nonterminal_number<'source>>, ()),
    )
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (
        Option<&'program mut nonterminal_expr_0_0<'source>>,
        (Option<&'program mut nonterminal_number<'source>>, ()),
    )
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        match self {
            Self::variant_0(inner) => inner.span(),
            Self::variant_1(inner) => inner.span(),
        }
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        match self {
            nonterminal_expr_0::variant_0(n) => (Some(n), (None, ())),
            nonterminal_expr_0::variant_1(n) => (None, (Some(n), ())),
        }
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        match self {
            nonterminal_expr_0::variant_0(n) => (Some(n), (None, ())),
            nonterminal_expr_0::variant_1(n) => (None, (Some(n), ())),
        }
    }
}
impl<'source> ::fandango::visitor::VisitableChildren for nonterminal_expr_0<'source> {
    fn visit_each<'node, V>(
        &'node mut self,
        visitor: V,
    ) -> ::fandango::visitor::VisitResult<V, Self::TypeMut<'node>>
    where
        V: ::fandango::visitor::Visitor<Self::TypeMut<'node>, Continue = V>,
    {
        match self {
            nonterminal_expr_0::variant_0(n) => visitor.visit(n, 0),
            nonterminal_expr_0::variant_1(n) => visitor.visit(&mut **n, 0),
        }
    }
    fn visit_nth<'node, V>(
        &'node mut self,
        visitor: V,
        idx: usize,
    ) -> ::fandango::visitor::MaybeVisitResult<V, Self::TypeMut<'node>>
    where
        V: ::fandango::visitor::Visitor<Self::TypeMut<'node>>,
    {
        if idx == 0 {
            match self {
                nonterminal_expr_0::variant_0(n) => Ok(visitor.visit(n, 0)),
                nonterminal_expr_0::variant_1(n) => Ok(visitor.visit(&mut **n, 0)),
            }
        } else {
            Err(visitor)
        }
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_expr_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_expr_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_expr_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_expr_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_expr_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_expr_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_expr_0<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_expr_0<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_expr_0(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_expr_0<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        if ::core::cfg!(debug_assertions) {
            match (&(value.as_rule()), &Rule::expr_0) {
                (left_val, right_val) => todo!(),
            };
        };
        let mut children = value.into_inner();
        let child_0 = children.next().expect("Expected exactly one descendent.");
        if ::core::cfg!(debug_assertions) {
            ::core::assert!(
                children.next().is_none(),
                "Expected exactly one descendent."
            );
        };
        Ok(match child_0.as_rule() {
            Rule::expr_0_0 => nonterminal_expr_0::variant_0(
                nonterminal_expr_0_0::try_from((source, child_0))?.into(),
            ),
            Rule::number => nonterminal_expr_0::variant_1(
                nonterminal_number::try_from((source, child_0))?.into(),
            ),
            _ => ::core::panic!(
                "not implemented: {}",
                format_args!("Not a child of this alternative.")
            ),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_expr_0_0<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
    child_0: ::std::boxed::Box<nonterminal_number<'source>>,
    child_1: nonterminal_expr_0_0_1<'source>,
    child_2: ::std::boxed::Box<nonterminal_expr<'source>>,
}
impl<'source> ::fandango::typing::Node for nonterminal_expr_0_0<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (
        &'program nonterminal_number<'source>,
        (
            &'program nonterminal_expr_0_0_1<'source>,
            (&'program nonterminal_expr<'source>, ()),
        ),
    )
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (
        &'program mut nonterminal_number<'source>,
        (
            &'program mut nonterminal_expr_0_0_1<'source>,
            (&'program mut nonterminal_expr<'source>, ()),
        ),
    )
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&self.child_0, (&self.child_1, (&self.child_2, ())))
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (
            &mut self.child_0,
            (&mut self.child_1, (&mut self.child_2, ())),
        )
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_expr_0_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_expr_0_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_expr_0_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_expr_0_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_expr_0_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_expr_0_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_expr_0_0<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_expr_0_0<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_expr_0_0(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_expr_0_0<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        if ::core::cfg!(debug_assertions) {
            match (&(value.as_rule()), &Rule::expr_0_0) {
                (left_val, right_val) => todo!(),
            };
        };
        let span = value.as_span();
        let (child_0, child_1, child_2) = {
            let iter = &mut (value.into_inner());
            let out = (
                {
                    let tmp = iter.next().unwrap();
                    if cfg!(debug_assertions) {
                        let value = (tmp.as_rule());
                        #[allow(unreachable_patterns)]
                        match value {
                            number => {}
                            _ => panic!(
                                "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                                stringify!(number),
                                value
                            ),
                        }
                    }
                    tmp
                },
                {
                    let tmp = iter.next().unwrap();
                    if cfg!(debug_assertions) {
                        let value = (tmp.as_rule());
                        #[allow(unreachable_patterns)]
                        match value {
                            expr_0_0_1 => {}
                            _ => panic!(
                                "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                                stringify!(expr_0_0_1),
                                value
                            ),
                        }
                    }
                    tmp
                },
                {
                    let tmp = iter.next().unwrap();
                    if cfg!(debug_assertions) {
                        let value = (tmp.as_rule());
                        #[allow(unreachable_patterns)]
                        match value {
                            expr => {}
                            _ => panic!(
                                "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                                stringify!(expr),
                                value
                            ),
                        }
                    }
                    tmp
                },
            );
            if cfg!(debug_assertions) {
                let value = (iter.next());
                #[allow(unreachable_patterns)]
                match value {
                    Option::None => {}
                    _ => panic!(
                        "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                        stringify!(Option::None),
                        value
                    ),
                }
            }
            out
        };
        Ok(Self {
            child_0: nonterminal_number::try_from((source.clone(), child_0))?.into(),
            child_1: nonterminal_expr_0_0_1::try_from((source.clone(), child_1))?.into(),
            child_2: nonterminal_expr::try_from((source.clone(), child_2))?.into(),
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_number<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
    child_0: nonterminal_number_0<'source>,
}
impl<'source> ::fandango::typing::Node for nonterminal_number<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'program nonterminal_number_0<'source>, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'program mut nonterminal_number_0<'source>, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&self.child_0, ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&mut self.child_0, ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_number<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_number<'source>) -> Type<'program, 'source> {
        Type::nonterminal_number(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_number<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_number<'source>) -> Type<'program, 'source> {
        Type::nonterminal_number(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_number<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_number<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_number(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_number<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        if ::core::cfg!(debug_assertions) {
            match (&(value.as_rule()), &Rule::number) {
                (left_val, right_val) => todo!(),
            };
        };
        let span = value.as_span();
        let (child_0,) = {
            let iter = &mut (value.into_inner());
            let out = ({
                let tmp = iter.next().unwrap();
                if cfg!(debug_assertions) {
                    let value = (tmp.as_rule());
                    #[allow(unreachable_patterns)]
                    match value {
                        number_0 => {}
                        _ => panic!(
                            "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                            stringify!(number_0),
                            value
                        ),
                    }
                }
                tmp
            },);
            if cfg!(debug_assertions) {
                let value = (iter.next());
                #[allow(unreachable_patterns)]
                match value {
                    Option::None => {}
                    _ => panic!(
                        "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                        stringify!(Option::None),
                        value
                    ),
                }
            }
            out
        };
        Ok(Self {
            child_0: nonterminal_number_0::try_from((source.clone(), child_0))?.into(),
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub enum nonterminal_number_0<'source> {
    variant_0(nonterminal_number_0_0<'source>),
    variant_1(nonterminal_number_0_1<'source>),
}
impl<'source> ::fandango::typing::Node for nonterminal_number_0<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (
        Option<&'program nonterminal_number_0_0<'source>>,
        (Option<&'program nonterminal_number_0_1<'source>>, ()),
    )
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (
        Option<&'program mut nonterminal_number_0_0<'source>>,
        (Option<&'program mut nonterminal_number_0_1<'source>>, ()),
    )
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        match self {
            Self::variant_0(inner) => inner.span(),
            Self::variant_1(inner) => inner.span(),
        }
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        match self {
            nonterminal_number_0::variant_0(n) => (Some(n), (None, ())),
            nonterminal_number_0::variant_1(n) => (None, (Some(n), ())),
        }
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        match self {
            nonterminal_number_0::variant_0(n) => (Some(n), (None, ())),
            nonterminal_number_0::variant_1(n) => (None, (Some(n), ())),
        }
    }
}
impl<'source> ::fandango::visitor::VisitableChildren for nonterminal_number_0<'source> {
    fn visit_each<'node, V>(
        &'node mut self,
        visitor: V,
    ) -> ::fandango::visitor::VisitResult<V, Self::TypeMut<'node>>
    where
        V: ::fandango::visitor::Visitor<Self::TypeMut<'node>, Continue = V>,
    {
        match self {
            nonterminal_number_0::variant_0(n) => visitor.visit(n, 0),
            nonterminal_number_0::variant_1(n) => visitor.visit(n, 0),
        }
    }
    fn visit_nth<'node, V>(
        &'node mut self,
        visitor: V,
        idx: usize,
    ) -> ::fandango::visitor::MaybeVisitResult<V, Self::TypeMut<'node>>
    where
        V: ::fandango::visitor::Visitor<Self::TypeMut<'node>>,
    {
        if idx == 0 {
            match self {
                nonterminal_number_0::variant_0(n) => Ok(visitor.visit(n, 0)),
                nonterminal_number_0::variant_1(n) => Ok(visitor.visit(n, 0)),
            }
        } else {
            Err(visitor)
        }
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_number_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_number_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_number_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_number_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_number_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_number_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_number_0<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_number_0<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_number_0(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_number_0<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        if ::core::cfg!(debug_assertions) {
            match (&(value.as_rule()), &Rule::number_0) {
                (left_val, right_val) => todo!(),
            };
        };
        let mut children = value.into_inner();
        let child_0 = children.next().expect("Expected exactly one descendent.");
        if ::core::cfg!(debug_assertions) {
            ::core::assert!(
                children.next().is_none(),
                "Expected exactly one descendent."
            );
        };
        Ok(match child_0.as_rule() {
            Rule::number_0_0 => nonterminal_number_0::variant_0(
                nonterminal_number_0_0::try_from((source, child_0))?.into(),
            ),
            Rule::number_0_1 => nonterminal_number_0::variant_1(
                nonterminal_number_0_1::try_from((source, child_0))?.into(),
            ),
            _ => ::core::panic!(
                "not implemented: {}",
                format_args!("Not a child of this alternative.")
            ),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_number_0_0<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
}
impl<'source> ::fandango::typing::Node for nonterminal_number_0_0<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'static str, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'static str, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&"0", ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&"0", ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_number_0_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_number_0_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_number_0_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_number_0_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_number_0_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_number_0_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_number_0_0<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_number_0_0<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_number_0_0(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_number_0_0<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        let span = value.as_span();
        if ::core::cfg!(debug_assertions) {
            match (&(span.as_str()), &"0") {
                (left_val, right_val) => todo!(),
            };
        };
        Ok(Self {
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_number_0_1<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
    child_0: ::std::boxed::Box<nonterminal_non_zero<'source>>,
    child_1: nonterminal_number_0_1_1<'source>,
}
impl<'source> ::fandango::typing::Node for nonterminal_number_0_1<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (
        &'program nonterminal_non_zero<'source>,
        (&'program nonterminal_number_0_1_1<'source>, ()),
    )
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (
        &'program mut nonterminal_non_zero<'source>,
        (&'program mut nonterminal_number_0_1_1<'source>, ()),
    )
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&self.child_0, (&self.child_1, ()))
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&mut self.child_0, (&mut self.child_1, ()))
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_number_0_1<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_number_0_1<'source>) -> Type<'program, 'source> {
        Type::nonterminal_number_0_1(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_number_0_1<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_number_0_1<'source>) -> Type<'program, 'source> {
        Type::nonterminal_number_0_1(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_number_0_1<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_number_0_1<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_number_0_1(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_number_0_1<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        if ::core::cfg!(debug_assertions) {
            match (&(value.as_rule()), &Rule::number_0_1) {
                (left_val, right_val) => todo!(),
            };
        };
        let span = value.as_span();
        let (child_0, child_1) = {
            let iter = &mut (value.into_inner());
            let out = (
                {
                    let tmp = iter.next().unwrap();
                    if cfg!(debug_assertions) {
                        let value = (tmp.as_rule());
                        #[allow(unreachable_patterns)]
                        match value {
                            non_zero => {}
                            _ => panic!(
                                "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                                stringify!(non_zero),
                                value
                            ),
                        }
                    }
                    tmp
                },
                {
                    let tmp = iter.next().unwrap();
                    if cfg!(debug_assertions) {
                        let value = (tmp.as_rule());
                        #[allow(unreachable_patterns)]
                        match value {
                            number_0_1_1 => {}
                            _ => panic!(
                                "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                                stringify!(number_0_1_1),
                                value
                            ),
                        }
                    }
                    tmp
                },
            );
            if cfg!(debug_assertions) {
                let value = (iter.next());
                #[allow(unreachable_patterns)]
                match value {
                    Option::None => {}
                    _ => panic!(
                        "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                        stringify!(Option::None),
                        value
                    ),
                }
            }
            out
        };
        Ok(Self {
            child_0: nonterminal_non_zero::try_from((source.clone(), child_0))?.into(),
            child_1: nonterminal_number_0_1_1::try_from((source.clone(), child_1))?.into(),
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_non_zero<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
    child_0: nonterminal_non_zero_0<'source>,
}
impl<'source> ::fandango::typing::Node for nonterminal_non_zero<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'program nonterminal_non_zero_0<'source>, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'program mut nonterminal_non_zero_0<'source>, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&self.child_0, ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&mut self.child_0, ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_non_zero<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_non_zero<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_non_zero(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_non_zero<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        if ::core::cfg!(debug_assertions) {
            match (&(value.as_rule()), &Rule::non_zero) {
                (left_val, right_val) => todo!(),
            };
        };
        let span = value.as_span();
        let (child_0,) = {
            let iter = &mut (value.into_inner());
            let out = ({
                let tmp = iter.next().unwrap();
                if cfg!(debug_assertions) {
                    let value = (tmp.as_rule());
                    #[allow(unreachable_patterns)]
                    match value {
                        non_zero_0 => {}
                        _ => panic!(
                            "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                            stringify!(non_zero_0),
                            value
                        ),
                    }
                }
                tmp
            },);
            if cfg!(debug_assertions) {
                let value = (iter.next());
                #[allow(unreachable_patterns)]
                match value {
                    Option::None => {}
                    _ => panic!(
                        "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                        stringify!(Option::None),
                        value
                    ),
                }
            }
            out
        };
        Ok(Self {
            child_0: nonterminal_non_zero_0::try_from((source.clone(), child_0))?.into(),
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub enum nonterminal_non_zero_0<'source> {
    variant_0(nonterminal_non_zero_0_0<'source>),
    variant_1(nonterminal_non_zero_0_1<'source>),
    variant_2(nonterminal_non_zero_0_2<'source>),
    variant_3(nonterminal_non_zero_0_3<'source>),
    variant_4(nonterminal_non_zero_0_4<'source>),
    variant_5(nonterminal_non_zero_0_5<'source>),
    variant_6(nonterminal_non_zero_0_6<'source>),
    variant_7(nonterminal_non_zero_0_7<'source>),
    variant_8(nonterminal_non_zero_0_8<'source>),
}
impl<'source> ::fandango::typing::Node for nonterminal_non_zero_0<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (
        Option<&'program nonterminal_non_zero_0_0<'source>>,
        (
            Option<&'program nonterminal_non_zero_0_1<'source>>,
            (
                Option<&'program nonterminal_non_zero_0_2<'source>>,
                (
                    Option<&'program nonterminal_non_zero_0_3<'source>>,
                    (
                        Option<&'program nonterminal_non_zero_0_4<'source>>,
                        (
                            Option<&'program nonterminal_non_zero_0_5<'source>>,
                            (
                                Option<&'program nonterminal_non_zero_0_6<'source>>,
                                (
                                    Option<&'program nonterminal_non_zero_0_7<'source>>,
                                    (Option<&'program nonterminal_non_zero_0_8<'source>>, ()),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (
        Option<&'program mut nonterminal_non_zero_0_0<'source>>,
        (
            Option<&'program mut nonterminal_non_zero_0_1<'source>>,
            (
                Option<&'program mut nonterminal_non_zero_0_2<'source>>,
                (
                    Option<&'program mut nonterminal_non_zero_0_3<'source>>,
                    (
                        Option<&'program mut nonterminal_non_zero_0_4<'source>>,
                        (
                            Option<&'program mut nonterminal_non_zero_0_5<'source>>,
                            (
                                Option<&'program mut nonterminal_non_zero_0_6<'source>>,
                                (
                                    Option<&'program mut nonterminal_non_zero_0_7<'source>>,
                                    (Option<&'program mut nonterminal_non_zero_0_8<'source>>, ()),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        match self {
            Self::variant_0(inner) => inner.span(),
            Self::variant_1(inner) => inner.span(),
            Self::variant_2(inner) => inner.span(),
            Self::variant_3(inner) => inner.span(),
            Self::variant_4(inner) => inner.span(),
            Self::variant_5(inner) => inner.span(),
            Self::variant_6(inner) => inner.span(),
            Self::variant_7(inner) => inner.span(),
            Self::variant_8(inner) => inner.span(),
        }
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        match self {
            nonterminal_non_zero_0::variant_0(n) => (
                Some(n),
                (
                    None,
                    (None, (None, (None, (None, (None, (None, (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_1(n) => (
                None,
                (
                    Some(n),
                    (None, (None, (None, (None, (None, (None, (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_2(n) => (
                None,
                (
                    None,
                    (Some(n), (None, (None, (None, (None, (None, (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_3(n) => (
                None,
                (
                    None,
                    (None, (Some(n), (None, (None, (None, (None, (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_4(n) => (
                None,
                (
                    None,
                    (None, (None, (Some(n), (None, (None, (None, (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_5(n) => (
                None,
                (
                    None,
                    (None, (None, (None, (Some(n), (None, (None, (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_6(n) => (
                None,
                (
                    None,
                    (None, (None, (None, (None, (Some(n), (None, (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_7(n) => (
                None,
                (
                    None,
                    (None, (None, (None, (None, (None, (Some(n), (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_8(n) => (
                None,
                (
                    None,
                    (None, (None, (None, (None, (None, (None, (Some(n), ()))))))),
                ),
            ),
        }
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        match self {
            nonterminal_non_zero_0::variant_0(n) => (
                Some(n),
                (
                    None,
                    (None, (None, (None, (None, (None, (None, (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_1(n) => (
                None,
                (
                    Some(n),
                    (None, (None, (None, (None, (None, (None, (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_2(n) => (
                None,
                (
                    None,
                    (Some(n), (None, (None, (None, (None, (None, (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_3(n) => (
                None,
                (
                    None,
                    (None, (Some(n), (None, (None, (None, (None, (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_4(n) => (
                None,
                (
                    None,
                    (None, (None, (Some(n), (None, (None, (None, (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_5(n) => (
                None,
                (
                    None,
                    (None, (None, (None, (Some(n), (None, (None, (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_6(n) => (
                None,
                (
                    None,
                    (None, (None, (None, (None, (Some(n), (None, (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_7(n) => (
                None,
                (
                    None,
                    (None, (None, (None, (None, (None, (Some(n), (None, ()))))))),
                ),
            ),
            nonterminal_non_zero_0::variant_8(n) => (
                None,
                (
                    None,
                    (None, (None, (None, (None, (None, (None, (Some(n), ()))))))),
                ),
            ),
        }
    }
}
impl<'source> ::fandango::visitor::VisitableChildren for nonterminal_non_zero_0<'source> {
    fn visit_each<'node, V>(
        &'node mut self,
        visitor: V,
    ) -> ::fandango::visitor::VisitResult<V, Self::TypeMut<'node>>
    where
        V: ::fandango::visitor::Visitor<Self::TypeMut<'node>, Continue = V>,
    {
        match self {
            nonterminal_non_zero_0::variant_0(n) => visitor.visit(n, 0),
            nonterminal_non_zero_0::variant_1(n) => visitor.visit(n, 0),
            nonterminal_non_zero_0::variant_2(n) => visitor.visit(n, 0),
            nonterminal_non_zero_0::variant_3(n) => visitor.visit(n, 0),
            nonterminal_non_zero_0::variant_4(n) => visitor.visit(n, 0),
            nonterminal_non_zero_0::variant_5(n) => visitor.visit(n, 0),
            nonterminal_non_zero_0::variant_6(n) => visitor.visit(n, 0),
            nonterminal_non_zero_0::variant_7(n) => visitor.visit(n, 0),
            nonterminal_non_zero_0::variant_8(n) => visitor.visit(n, 0),
        }
    }
    fn visit_nth<'node, V>(
        &'node mut self,
        visitor: V,
        idx: usize,
    ) -> ::fandango::visitor::MaybeVisitResult<V, Self::TypeMut<'node>>
    where
        V: ::fandango::visitor::Visitor<Self::TypeMut<'node>>,
    {
        if idx == 0 {
            match self {
                nonterminal_non_zero_0::variant_0(n) => Ok(visitor.visit(n, 0)),
                nonterminal_non_zero_0::variant_1(n) => Ok(visitor.visit(n, 0)),
                nonterminal_non_zero_0::variant_2(n) => Ok(visitor.visit(n, 0)),
                nonterminal_non_zero_0::variant_3(n) => Ok(visitor.visit(n, 0)),
                nonterminal_non_zero_0::variant_4(n) => Ok(visitor.visit(n, 0)),
                nonterminal_non_zero_0::variant_5(n) => Ok(visitor.visit(n, 0)),
                nonterminal_non_zero_0::variant_6(n) => Ok(visitor.visit(n, 0)),
                nonterminal_non_zero_0::variant_7(n) => Ok(visitor.visit(n, 0)),
                nonterminal_non_zero_0::variant_8(n) => Ok(visitor.visit(n, 0)),
            }
        } else {
            Err(visitor)
        }
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_non_zero_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_non_zero_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_non_zero_0(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_non_zero_0<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        if ::core::cfg!(debug_assertions) {
            match (&(value.as_rule()), &Rule::non_zero_0) {
                (left_val, right_val) => todo!(),
            };
        };
        let mut children = value.into_inner();
        let child_0 = children.next().expect("Expected exactly one descendent.");
        if ::core::cfg!(debug_assertions) {
            ::core::assert!(
                children.next().is_none(),
                "Expected exactly one descendent."
            );
        };
        Ok(match child_0.as_rule() {
            Rule::non_zero_0_0 => nonterminal_non_zero_0::variant_0(
                nonterminal_non_zero_0_0::try_from((source, child_0))?.into(),
            ),
            Rule::non_zero_0_1 => nonterminal_non_zero_0::variant_1(
                nonterminal_non_zero_0_1::try_from((source, child_0))?.into(),
            ),
            Rule::non_zero_0_2 => nonterminal_non_zero_0::variant_2(
                nonterminal_non_zero_0_2::try_from((source, child_0))?.into(),
            ),
            Rule::non_zero_0_3 => nonterminal_non_zero_0::variant_3(
                nonterminal_non_zero_0_3::try_from((source, child_0))?.into(),
            ),
            Rule::non_zero_0_4 => nonterminal_non_zero_0::variant_4(
                nonterminal_non_zero_0_4::try_from((source, child_0))?.into(),
            ),
            Rule::non_zero_0_5 => nonterminal_non_zero_0::variant_5(
                nonterminal_non_zero_0_5::try_from((source, child_0))?.into(),
            ),
            Rule::non_zero_0_6 => nonterminal_non_zero_0::variant_6(
                nonterminal_non_zero_0_6::try_from((source, child_0))?.into(),
            ),
            Rule::non_zero_0_7 => nonterminal_non_zero_0::variant_7(
                nonterminal_non_zero_0_7::try_from((source, child_0))?.into(),
            ),
            Rule::non_zero_0_8 => nonterminal_non_zero_0::variant_8(
                nonterminal_non_zero_0_8::try_from((source, child_0))?.into(),
            ),
            _ => ::core::panic!(
                "not implemented: {}",
                format_args!("Not a child of this alternative.")
            ),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_non_zero_0_0<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
}
impl<'source> ::fandango::typing::Node for nonterminal_non_zero_0_0<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'static str, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'static str, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&"1", ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&"1", ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_non_zero_0_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_non_zero_0_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_0<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_0<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_non_zero_0_0(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_non_zero_0_0<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        let span = value.as_span();
        if ::core::cfg!(debug_assertions) {
            match (&(span.as_str()), &"1") {
                (left_val, right_val) => todo!(),
            };
        };
        Ok(Self {
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_non_zero_0_1<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
}
impl<'source> ::fandango::typing::Node for nonterminal_non_zero_0_1<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'static str, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'static str, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&"2", ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&"2", ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_non_zero_0_1<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_non_zero_0_1<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_1(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_1<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_1<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_1(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_1<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_1<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_non_zero_0_1(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_non_zero_0_1<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        let span = value.as_span();
        if ::core::cfg!(debug_assertions) {
            match (&(span.as_str()), &"2") {
                (left_val, right_val) => todo!(),
            };
        };
        Ok(Self {
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_non_zero_0_2<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
}
impl<'source> ::fandango::typing::Node for nonterminal_non_zero_0_2<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'static str, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'static str, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&"3", ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&"3", ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_non_zero_0_2<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_non_zero_0_2<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_2(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_2<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_2<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_2(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_2<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_2<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_non_zero_0_2(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_non_zero_0_2<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        let span = value.as_span();
        if ::core::cfg!(debug_assertions) {
            match (&(span.as_str()), &"3") {
                (left_val, right_val) => todo!(),
            };
        };
        Ok(Self {
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_non_zero_0_3<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
}
impl<'source> ::fandango::typing::Node for nonterminal_non_zero_0_3<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'static str, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'static str, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&"4", ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&"4", ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_non_zero_0_3<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_non_zero_0_3<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_3(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_3<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_3<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_3(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_3<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_3<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_non_zero_0_3(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_non_zero_0_3<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        let span = value.as_span();
        if ::core::cfg!(debug_assertions) {
            match (&(span.as_str()), &"4") {
                (left_val, right_val) => todo!(),
            };
        };
        Ok(Self {
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_non_zero_0_4<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
}
impl<'source> ::fandango::typing::Node for nonterminal_non_zero_0_4<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'static str, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'static str, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&"5", ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&"5", ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_non_zero_0_4<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_non_zero_0_4<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_4(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_4<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_4<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_4(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_4<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_4<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_non_zero_0_4(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_non_zero_0_4<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        let span = value.as_span();
        if ::core::cfg!(debug_assertions) {
            match (&(span.as_str()), &"5") {
                (left_val, right_val) => todo!(),
            };
        };
        Ok(Self {
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_non_zero_0_5<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
}
impl<'source> ::fandango::typing::Node for nonterminal_non_zero_0_5<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'static str, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'static str, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&"6", ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&"6", ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_non_zero_0_5<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_non_zero_0_5<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_5(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_5<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_5<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_5(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_5<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_5<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_non_zero_0_5(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_non_zero_0_5<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        let span = value.as_span();
        if ::core::cfg!(debug_assertions) {
            match (&(span.as_str()), &"6") {
                (left_val, right_val) => todo!(),
            };
        };
        Ok(Self {
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_non_zero_0_6<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
}
impl<'source> ::fandango::typing::Node for nonterminal_non_zero_0_6<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'static str, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'static str, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&"7", ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&"7", ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_non_zero_0_6<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_non_zero_0_6<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_6(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_6<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_6<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_6(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_6<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_6<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_non_zero_0_6(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_non_zero_0_6<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        let span = value.as_span();
        if ::core::cfg!(debug_assertions) {
            match (&(span.as_str()), &"7") {
                (left_val, right_val) => todo!(),
            };
        };
        Ok(Self {
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_non_zero_0_7<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
}
impl<'source> ::fandango::typing::Node for nonterminal_non_zero_0_7<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'static str, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'static str, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&"8", ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&"8", ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_non_zero_0_7<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_non_zero_0_7<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_7(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_7<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_7<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_7(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_7<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_7<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_non_zero_0_7(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_non_zero_0_7<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        let span = value.as_span();
        if ::core::cfg!(debug_assertions) {
            match (&(span.as_str()), &"8") {
                (left_val, right_val) => todo!(),
            };
        };
        Ok(Self {
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_non_zero_0_8<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
}
impl<'source> ::fandango::typing::Node for nonterminal_non_zero_0_8<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'static str, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'static str, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&"9", ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&"9", ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_non_zero_0_8<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_non_zero_0_8<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_8(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_8<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_8<'source>) -> Type<'program, 'source> {
        Type::nonterminal_non_zero_0_8(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_non_zero_0_8<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_non_zero_0_8<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_non_zero_0_8(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_non_zero_0_8<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        let span = value.as_span();
        if ::core::cfg!(debug_assertions) {
            match (&(span.as_str()), &"9") {
                (left_val, right_val) => todo!(),
            };
        };
        Ok(Self {
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_number_0_1_1<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
    child_0: ::std::vec::Vec<nonterminal_digit<'source>>,
}
impl<'source> ::fandango::typing::Node for nonterminal_number_0_1_1<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = &'program Vec<nonterminal_digit<'source>>
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = &'program mut Vec<nonterminal_digit<'source>>
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        &self.child_0
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        &mut self.child_0
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_number_0_1_1<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_number_0_1_1<'source>) -> Type<'program, 'source> {
        Type::nonterminal_number_0_1_1(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_number_0_1_1<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_number_0_1_1<'source>) -> Type<'program, 'source> {
        Type::nonterminal_number_0_1_1(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_number_0_1_1<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_number_0_1_1<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_number_0_1_1(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_number_0_1_1<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        if ::core::cfg!(debug_assertions) {
            match (&(value.as_rule()), &Rule::number_0_1_1) {
                (left_val, right_val) => todo!(),
            };
        };
        let span = value.as_span();
        let child_0 = value
            .into_inner()
            .map(|value| {
                if ::core::cfg!(debug_assertions) {
                    match (&(value.as_rule()), &Rule::digit) {
                        (left_val, right_val) => todo!(),
                    };
                };
                Ok(nonterminal_digit::try_from((source.clone(), value))?.into())
            })
            .collect::<Result<_, Self::Error>>()?;
        Ok(Self {
            child_0,
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_digit<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
    child_0: nonterminal_digit_0<'source>,
}
impl<'source> ::fandango::typing::Node for nonterminal_digit<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'program nonterminal_digit_0<'source>, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'program mut nonterminal_digit_0<'source>, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&self.child_0, ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&mut self.child_0, ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_digit<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_digit<'source>) -> Type<'program, 'source> {
        Type::nonterminal_digit(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_digit<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_digit<'source>) -> Type<'program, 'source> {
        Type::nonterminal_digit(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_digit<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_digit<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_digit(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_digit<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        if ::core::cfg!(debug_assertions) {
            match (&(value.as_rule()), &Rule::digit) {
                (left_val, right_val) => todo!(),
            };
        };
        let span = value.as_span();
        let (child_0,) = {
            let iter = &mut (value.into_inner());
            let out = ({
                let tmp = iter.next().unwrap();
                if cfg!(debug_assertions) {
                    let value = (tmp.as_rule());
                    #[allow(unreachable_patterns)]
                    match value {
                        digit_0 => {}
                        _ => panic!(
                            "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                            stringify!(digit_0),
                            value
                        ),
                    }
                }
                tmp
            },);
            if cfg!(debug_assertions) {
                let value = (iter.next());
                #[allow(unreachable_patterns)]
                match value {
                    Option::None => {}
                    _ => panic!(
                        "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                        stringify!(Option::None),
                        value
                    ),
                }
            }
            out
        };
        Ok(Self {
            child_0: nonterminal_digit_0::try_from((source.clone(), child_0))?.into(),
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub enum nonterminal_digit_0<'source> {
    variant_0(nonterminal_digit_0_0<'source>),
    variant_1(::std::boxed::Box<nonterminal_non_zero<'source>>),
}
impl<'source> ::fandango::typing::Node for nonterminal_digit_0<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (
        Option<&'program nonterminal_digit_0_0<'source>>,
        (Option<&'program nonterminal_non_zero<'source>>, ()),
    )
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (
        Option<&'program mut nonterminal_digit_0_0<'source>>,
        (Option<&'program mut nonterminal_non_zero<'source>>, ()),
    )
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        match self {
            Self::variant_0(inner) => inner.span(),
            Self::variant_1(inner) => inner.span(),
        }
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        match self {
            nonterminal_digit_0::variant_0(n) => (Some(n), (None, ())),
            nonterminal_digit_0::variant_1(n) => (None, (Some(n), ())),
        }
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        match self {
            nonterminal_digit_0::variant_0(n) => (Some(n), (None, ())),
            nonterminal_digit_0::variant_1(n) => (None, (Some(n), ())),
        }
    }
}
impl<'source> ::fandango::visitor::VisitableChildren for nonterminal_digit_0<'source> {
    fn visit_each<'node, V>(
        &'node mut self,
        visitor: V,
    ) -> ::fandango::visitor::VisitResult<V, Self::TypeMut<'node>>
    where
        V: ::fandango::visitor::Visitor<Self::TypeMut<'node>, Continue = V>,
    {
        match self {
            nonterminal_digit_0::variant_0(n) => visitor.visit(n, 0),
            nonterminal_digit_0::variant_1(n) => visitor.visit(&mut **n, 0),
        }
    }
    fn visit_nth<'node, V>(
        &'node mut self,
        visitor: V,
        idx: usize,
    ) -> ::fandango::visitor::MaybeVisitResult<V, Self::TypeMut<'node>>
    where
        V: ::fandango::visitor::Visitor<Self::TypeMut<'node>>,
    {
        if idx == 0 {
            match self {
                nonterminal_digit_0::variant_0(n) => Ok(visitor.visit(n, 0)),
                nonterminal_digit_0::variant_1(n) => Ok(visitor.visit(&mut **n, 0)),
            }
        } else {
            Err(visitor)
        }
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_digit_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_digit_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_digit_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_digit_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_digit_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_digit_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_digit_0<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_digit_0<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_digit_0(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_digit_0<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        if ::core::cfg!(debug_assertions) {
            match (&(value.as_rule()), &Rule::digit_0) {
                (left_val, right_val) => todo!(),
            };
        };
        let mut children = value.into_inner();
        let child_0 = children.next().expect("Expected exactly one descendent.");
        if ::core::cfg!(debug_assertions) {
            ::core::assert!(
                children.next().is_none(),
                "Expected exactly one descendent."
            );
        };
        Ok(match child_0.as_rule() {
            Rule::digit_0_0 => nonterminal_digit_0::variant_0(
                nonterminal_digit_0_0::try_from((source, child_0))?.into(),
            ),
            Rule::non_zero => nonterminal_digit_0::variant_1(
                nonterminal_non_zero::try_from((source, child_0))?.into(),
            ),
            _ => ::core::panic!(
                "not implemented: {}",
                format_args!("Not a child of this alternative.")
            ),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_digit_0_0<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
}
impl<'source> ::fandango::typing::Node for nonterminal_digit_0_0<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'static str, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'static str, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&"0", ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&"0", ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_digit_0_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_digit_0_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_digit_0_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_digit_0_0<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_digit_0_0<'source>) -> Type<'program, 'source> {
        Type::nonterminal_digit_0_0(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_digit_0_0<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_digit_0_0<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_digit_0_0(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_digit_0_0<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        let span = value.as_span();
        if ::core::cfg!(debug_assertions) {
            match (&(span.as_str()), &"0") {
                (left_val, right_val) => todo!(),
            };
        };
        Ok(Self {
            span: Some((source, span.start(), span.end())),
        })
    }
}
#[derive(Clone, Debug)]
pub struct nonterminal_expr_0_0_1<'source> {
    span: ::std::option::Option<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        usize,
        usize,
    )>,
}
impl<'source> ::fandango::typing::Node for nonterminal_expr_0_0_1<'source> {
    type Type<'program>
        = Type<'program, 'source>
    where
        'source: 'program;
    type TypeMut<'program>
        = TypeMut<'program, 'source>
    where
        'source: 'program;
    type ChildrenRef<'program>
        = (&'static str, ())
    where
        'source: 'program;
    type ChildrenRefMut<'program>
        = (&'static str, ())
    where
        'source: 'program;
    fn span(&self) -> ::std::option::Option<::fandango::Span<'_>> {
        ::fandango::typing::maybe_owned_span(&self.span)
    }
    fn children<'program>(&'program self) -> Self::ChildrenRef<'program> {
        (&"+", ())
    }
    fn children_mut<'program>(&'program mut self) -> Self::ChildrenRefMut<'program> {
        (&"+", ())
    }
}
impl<'program, 'source> ::std::convert::From<&'program nonterminal_expr_0_0_1<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program nonterminal_expr_0_0_1<'source>) -> Type<'program, 'source> {
        Type::nonterminal_expr_0_0_1(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_expr_0_0_1<'source>>
    for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_expr_0_0_1<'source>) -> Type<'program, 'source> {
        Type::nonterminal_expr_0_0_1(node)
    }
}
impl<'program, 'source> ::std::convert::From<&'program mut nonterminal_expr_0_0_1<'source>>
    for TypeMut<'program, 'source>
where
    'source: 'program,
{
    fn from(node: &'program mut nonterminal_expr_0_0_1<'source>) -> TypeMut<'program, 'source> {
        TypeMut::nonterminal_expr_0_0_1(node)
    }
}
impl<'source>
    ::std::convert::TryFrom<(
        ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
        ::fandango::iterators::Pair<'source, Rule>,
    )> for nonterminal_expr_0_0_1<'source>
{
    type Error = ParseError;
    fn try_from(
        (source, value): (
            ::std::rc::Rc<::std::borrow::Cow<'source, str>>,
            ::fandango::iterators::Pair<'source, Rule>,
        ),
    ) -> Result<Self, Self::Error> {
        let span = value.as_span();
        if ::core::cfg!(debug_assertions) {
            match (&(span.as_str()), &"+") {
                (left_val, right_val) => todo!(),
            };
        };
        Ok(Self {
            span: Some((source, span.start(), span.end())),
        })
    }
}
const FANDANGO_ARRAY_0: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Statement<'static>,
>] = &[
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Statement::Production(::fandango::lang::Tagged::known(
            ::fandango::lang::Production::known(
                ::fandango::lang::Tagged::known(
                    ::fandango::lang::Nonterminal::new("start"),
                    SOURCE,
                    0usize,
                    7usize,
                    1106402148066141360u64,
                ),
                ::fandango::lang::Tagged::known(
                    ::fandango::lang::Alternative::known(FANDANGO_ARRAY_1),
                    SOURCE,
                    12usize,
                    18usize,
                    15581170352542024178u64,
                ),
            ),
            SOURCE,
            0usize,
            19usize,
            8743268160687037333u64,
        )),
        SOURCE,
        0usize,
        19usize,
        8743268160687037333u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Statement::Production(::fandango::lang::Tagged::known(
            ::fandango::lang::Production::known(
                ::fandango::lang::Tagged::known(
                    ::fandango::lang::Nonterminal::new("expr"),
                    SOURCE,
                    20usize,
                    26usize,
                    15581170352542024178u64,
                ),
                ::fandango::lang::Tagged::known(
                    ::fandango::lang::Alternative::known(FANDANGO_ARRAY_3),
                    SOURCE,
                    31usize,
                    61usize,
                    7019597709516279866u64,
                ),
            ),
            SOURCE,
            20usize,
            62usize,
            14843343230485821166u64,
        )),
        SOURCE,
        20usize,
        62usize,
        14843343230485821166u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Statement::Production(::fandango::lang::Tagged::known(
            ::fandango::lang::Production::known(
                ::fandango::lang::Tagged::known(
                    ::fandango::lang::Nonterminal::new("number"),
                    SOURCE,
                    63usize,
                    71usize,
                    9892072354177723751u64,
                ),
                ::fandango::lang::Tagged::known(
                    ::fandango::lang::Alternative::known(FANDANGO_ARRAY_6),
                    SOURCE,
                    76usize,
                    100usize,
                    1445095283839907721u64,
                ),
            ),
            SOURCE,
            63usize,
            101usize,
            17223678699549649383u64,
        )),
        SOURCE,
        63usize,
        101usize,
        17223678699549649383u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Statement::Production(::fandango::lang::Tagged::known(
            ::fandango::lang::Production::known(
                ::fandango::lang::Tagged::known(
                    ::fandango::lang::Nonterminal::new("non_zero"),
                    SOURCE,
                    102usize,
                    112usize,
                    17896383424192950609u64,
                ),
                ::fandango::lang::Tagged::known(
                    ::fandango::lang::Alternative::known(FANDANGO_ARRAY_9),
                    SOURCE,
                    131usize,
                    291usize,
                    6696415703409689506u64,
                ),
            ),
            SOURCE,
            102usize,
            292usize,
            11885402508572071240u64,
        )),
        SOURCE,
        102usize,
        292usize,
        11885402508572071240u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Statement::Production(::fandango::lang::Tagged::known(
            ::fandango::lang::Production::known(
                ::fandango::lang::Tagged::known(
                    ::fandango::lang::Nonterminal::new("digit"),
                    SOURCE,
                    293usize,
                    300usize,
                    341213552731996594u64,
                ),
                ::fandango::lang::Tagged::known(
                    ::fandango::lang::Alternative::known(FANDANGO_ARRAY_19),
                    SOURCE,
                    305usize,
                    321usize,
                    2604707106976618784u64,
                ),
            ),
            SOURCE,
            293usize,
            322usize,
            11825122968285969415u64,
        )),
        SOURCE,
        293usize,
        322usize,
        11825122968285969415u64,
    ),
];
const FANDANGO_ARRAY_1: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Concatenation<'static>,
>] = &[::fandango::lang::Tagged::known(
    ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_2),
    SOURCE,
    12usize,
    18usize,
    15581170352542024178u64,
)];
const FANDANGO_ARRAY_2: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[::fandango::lang::Tagged::known(
    ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
        ::fandango::lang::Symbol::Nonterminal(::fandango::lang::Tagged::known(
            ::fandango::lang::Nonterminal::new("expr"),
            SOURCE,
            12usize,
            18usize,
            15581170352542024178u64,
        )),
        SOURCE,
        12usize,
        18usize,
        15581170352542024178u64,
    )),
    SOURCE,
    12usize,
    18usize,
    15581170352542024178u64,
)];
const FANDANGO_ARRAY_3: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Concatenation<'static>,
>] = &[
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_4),
        SOURCE,
        31usize,
        50usize,
        247027086840027101u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_5),
        SOURCE,
        53usize,
        61usize,
        9892072354177723751u64,
    ),
];
const FANDANGO_ARRAY_4: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
            ::fandango::lang::Symbol::Nonterminal(::fandango::lang::Tagged::known(
                ::fandango::lang::Nonterminal::new("number"),
                SOURCE,
                31usize,
                39usize,
                9892072354177723751u64,
            )),
            SOURCE,
            31usize,
            39usize,
            9892072354177723751u64,
        )),
        SOURCE,
        31usize,
        39usize,
        9892072354177723751u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
            ::fandango::lang::Symbol::String(::fandango::lang::Tagged::known(
                ::std::borrow::Cow::Borrowed("+"),
                SOURCE,
                41usize,
                42usize,
                7874756943448743542u64,
            )),
            SOURCE,
            40usize,
            43usize,
            3697678206658502662u64,
        )),
        SOURCE,
        40usize,
        43usize,
        3697678206658502662u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
            ::fandango::lang::Symbol::Nonterminal(::fandango::lang::Tagged::known(
                ::fandango::lang::Nonterminal::new("expr"),
                SOURCE,
                44usize,
                50usize,
                15581170352542024178u64,
            )),
            SOURCE,
            44usize,
            50usize,
            15581170352542024178u64,
        )),
        SOURCE,
        44usize,
        50usize,
        15581170352542024178u64,
    ),
];
const FANDANGO_ARRAY_5: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[::fandango::lang::Tagged::known(
    ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
        ::fandango::lang::Symbol::Nonterminal(::fandango::lang::Tagged::known(
            ::fandango::lang::Nonterminal::new("number"),
            SOURCE,
            53usize,
            61usize,
            9892072354177723751u64,
        )),
        SOURCE,
        53usize,
        61usize,
        9892072354177723751u64,
    )),
    SOURCE,
    53usize,
    61usize,
    9892072354177723751u64,
)];
const FANDANGO_ARRAY_6: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Concatenation<'static>,
>] = &[
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_7),
        SOURCE,
        76usize,
        80usize,
        15000350270615078019u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_8),
        SOURCE,
        82usize,
        100usize,
        3796803665485363574u64,
    ),
];
const FANDANGO_ARRAY_7: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[::fandango::lang::Tagged::known(
    ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
        ::fandango::lang::Symbol::String(::fandango::lang::Tagged::known(
            ::std::borrow::Cow::Borrowed("0"),
            SOURCE,
            77usize,
            78usize,
            18187302216140149989u64,
        )),
        SOURCE,
        76usize,
        79usize,
        6682204933907026391u64,
    )),
    SOURCE,
    76usize,
    79usize,
    6682204933907026391u64,
)];
const FANDANGO_ARRAY_8: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
            ::fandango::lang::Symbol::Nonterminal(::fandango::lang::Tagged::known(
                ::fandango::lang::Nonterminal::new("non_zero"),
                SOURCE,
                82usize,
                92usize,
                17896383424192950609u64,
            )),
            SOURCE,
            82usize,
            92usize,
            17896383424192950609u64,
        )),
        SOURCE,
        82usize,
        92usize,
        17896383424192950609u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Operator::Kleene(::fandango::lang::Tagged::known(
            ::fandango::lang::Symbol::Nonterminal(::fandango::lang::Tagged::known(
                ::fandango::lang::Nonterminal::new("digit"),
                SOURCE,
                92usize,
                99usize,
                341213552731996594u64,
            )),
            SOURCE,
            92usize,
            99usize,
            341213552731996594u64,
        )),
        SOURCE,
        92usize,
        100usize,
        14031342670026632640u64,
    ),
];
const FANDANGO_ARRAY_9: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Concatenation<'static>,
>] = &[
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_10),
        SOURCE,
        131usize,
        147usize,
        7435710708702168118u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_11),
        SOURCE,
        149usize,
        165usize,
        10742472133653286127u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_12),
        SOURCE,
        167usize,
        183usize,
        13856083133130550766u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_13),
        SOURCE,
        185usize,
        201usize,
        17472921388188839056u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_14),
        SOURCE,
        203usize,
        219usize,
        1857217909833400350u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_15),
        SOURCE,
        221usize,
        237usize,
        14155817818649700593u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_16),
        SOURCE,
        239usize,
        255usize,
        14979854272399052758u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_17),
        SOURCE,
        257usize,
        273usize,
        8997501477499799866u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_18),
        SOURCE,
        275usize,
        291usize,
        3370885169796838514u64,
    ),
];
const FANDANGO_ARRAY_10: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[::fandango::lang::Tagged::known(
    ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
        ::fandango::lang::Symbol::String(::fandango::lang::Tagged::known(
            ::std::borrow::Cow::Borrowed("1"),
            SOURCE,
            132usize,
            133usize,
            16569625464242099095u64,
        )),
        SOURCE,
        131usize,
        134usize,
        14488916458788220904u64,
    )),
    SOURCE,
    131usize,
    134usize,
    14488916458788220904u64,
)];
const FANDANGO_ARRAY_11: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[::fandango::lang::Tagged::known(
    ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
        ::fandango::lang::Symbol::String(::fandango::lang::Tagged::known(
            ::std::borrow::Cow::Borrowed("2"),
            SOURCE,
            150usize,
            151usize,
            10421790385219844055u64,
        )),
        SOURCE,
        149usize,
        152usize,
        2082053291164486785u64,
    )),
    SOURCE,
    149usize,
    152usize,
    2082053291164486785u64,
)];
const FANDANGO_ARRAY_12: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[::fandango::lang::Tagged::known(
    ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
        ::fandango::lang::Symbol::String(::fandango::lang::Tagged::known(
            ::std::borrow::Cow::Borrowed("3"),
            SOURCE,
            168usize,
            169usize,
            6303548800193346799u64,
        )),
        SOURCE,
        167usize,
        170usize,
        6564453380404105238u64,
    )),
    SOURCE,
    167usize,
    170usize,
    6564453380404105238u64,
)];
const FANDANGO_ARRAY_13: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[::fandango::lang::Tagged::known(
    ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
        ::fandango::lang::Symbol::String(::fandango::lang::Tagged::known(
            ::std::borrow::Cow::Borrowed("4"),
            SOURCE,
            186usize,
            187usize,
            17458395330688339326u64,
        )),
        SOURCE,
        185usize,
        188usize,
        16791822245982818106u64,
    )),
    SOURCE,
    185usize,
    188usize,
    16791822245982818106u64,
)];
const FANDANGO_ARRAY_14: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[::fandango::lang::Tagged::known(
    ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
        ::fandango::lang::Symbol::String(::fandango::lang::Tagged::known(
            ::std::borrow::Cow::Borrowed("5"),
            SOURCE,
            204usize,
            205usize,
            13858474648501043698u64,
        )),
        SOURCE,
        203usize,
        206usize,
        5174881775561756491u64,
    )),
    SOURCE,
    203usize,
    206usize,
    5174881775561756491u64,
)];
const FANDANGO_ARRAY_15: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[::fandango::lang::Tagged::known(
    ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
        ::fandango::lang::Symbol::String(::fandango::lang::Tagged::known(
            ::std::borrow::Cow::Borrowed("6"),
            SOURCE,
            222usize,
            223usize,
            16351538610376684187u64,
        )),
        SOURCE,
        221usize,
        224usize,
        10103043410743375004u64,
    )),
    SOURCE,
    221usize,
    224usize,
    10103043410743375004u64,
)];
const FANDANGO_ARRAY_16: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[::fandango::lang::Tagged::known(
    ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
        ::fandango::lang::Symbol::String(::fandango::lang::Tagged::known(
            ::std::borrow::Cow::Borrowed("7"),
            SOURCE,
            240usize,
            241usize,
            1231940324599871575u64,
        )),
        SOURCE,
        239usize,
        242usize,
        10550720158848192221u64,
    )),
    SOURCE,
    239usize,
    242usize,
    10550720158848192221u64,
)];
const FANDANGO_ARRAY_17: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[::fandango::lang::Tagged::known(
    ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
        ::fandango::lang::Symbol::String(::fandango::lang::Tagged::known(
            ::std::borrow::Cow::Borrowed("8"),
            SOURCE,
            258usize,
            259usize,
            4626802739630028119u64,
        )),
        SOURCE,
        257usize,
        260usize,
        12202792084480943227u64,
    )),
    SOURCE,
    257usize,
    260usize,
    12202792084480943227u64,
)];
const FANDANGO_ARRAY_18: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[::fandango::lang::Tagged::known(
    ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
        ::fandango::lang::Symbol::String(::fandango::lang::Tagged::known(
            ::std::borrow::Cow::Borrowed("9"),
            SOURCE,
            276usize,
            277usize,
            14044894836604074669u64,
        )),
        SOURCE,
        275usize,
        278usize,
        3774666582722538070u64,
    )),
    SOURCE,
    275usize,
    278usize,
    3774666582722538070u64,
)];
const FANDANGO_ARRAY_19: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Concatenation<'static>,
>] = &[
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_20),
        SOURCE,
        305usize,
        309usize,
        15000350270615078019u64,
    ),
    ::fandango::lang::Tagged::known(
        ::fandango::lang::Concatenation::known(FANDANGO_ARRAY_21),
        SOURCE,
        311usize,
        321usize,
        17896383424192950609u64,
    ),
];
const FANDANGO_ARRAY_20: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[::fandango::lang::Tagged::known(
    ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
        ::fandango::lang::Symbol::String(::fandango::lang::Tagged::known(
            ::std::borrow::Cow::Borrowed("0"),
            SOURCE,
            306usize,
            307usize,
            18187302216140149989u64,
        )),
        SOURCE,
        305usize,
        308usize,
        6682204933907026391u64,
    )),
    SOURCE,
    305usize,
    308usize,
    6682204933907026391u64,
)];
const FANDANGO_ARRAY_21: &'static [::fandango::lang::Tagged<
    'static,
    ::fandango::lang::Operator<'static>,
>] = &[::fandango::lang::Tagged::known(
    ::fandango::lang::Operator::Symbol(::fandango::lang::Tagged::known(
        ::fandango::lang::Symbol::Nonterminal(::fandango::lang::Tagged::known(
            ::fandango::lang::Nonterminal::new("non_zero"),
            SOURCE,
            311usize,
            321usize,
            17896383424192950609u64,
        )),
        SOURCE,
        311usize,
        321usize,
        17896383424192950609u64,
    )),
    SOURCE,
    311usize,
    321usize,
    17896383424192950609u64,
)];
impl ::fandango::typing::Structured for nonterminal_start<'_> {
    type FandangoType = ::fandango::lang::Nonterminal<'static>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match STRUCTURE.inner().statements() {
                ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Statement::Production(c) => c,
            _ => unreachable!(),
        }
    })
    .inner()
    .nonterminal();
}
impl ::fandango::typing::Structured for nonterminal_expr<'_> {
    type FandangoType = ::fandango::lang::Nonterminal<'static>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Symbol(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::Nonterminal(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_number<'_> {
    type FandangoType = ::fandango::lang::Nonterminal<'static>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[1usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Symbol(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::Nonterminal(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_expr_0_0_1<'_> {
    type FandangoType = ::std::borrow::Cow<'static, str>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[1usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[1usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Symbol(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::String(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_expr_0_0<'_> {
    type FandangoType = ::fandango::lang::Concatenation<'static>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match STRUCTURE.inner().statements() {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[1usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Statement::Production(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        .alternative()
        .inner()
        .concatenations()
        {
            ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_expr_0<'_> {
    type FandangoType = ::fandango::lang::Alternative<'static>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match STRUCTURE.inner().statements() {
                ::std::borrow::Cow::Borrowed(inner) => &inner[1usize],
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Statement::Production(c) => c,
            _ => unreachable!(),
        }
    })
    .inner()
    .alternative();
}
impl ::fandango::typing::Structured for nonterminal_number_0_0<'_> {
    type FandangoType = ::std::borrow::Cow<'static, str>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[2usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Symbol(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::String(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_non_zero<'_> {
    type FandangoType = ::fandango::lang::Nonterminal<'static>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[2usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[1usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Symbol(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::Nonterminal(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_digit<'_> {
    type FandangoType = ::fandango::lang::Nonterminal<'static>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[2usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[1usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[1usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Kleene(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::Nonterminal(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_number_0_1_1<'_> {
    type FandangoType = ::fandango::lang::Operator<'static>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match STRUCTURE.inner().statements() {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[2usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                {
                    ::fandango::lang::Statement::Production(c) => c,
                    _ => unreachable!(),
                }
            })
            .inner()
            .alternative()
            .inner()
            .concatenations()
            {
                ::std::borrow::Cow::Borrowed(inner) => &inner[1usize],
                _ => unreachable!(),
            }
        })
        .inner()
        .operators()
        {
            ::std::borrow::Cow::Borrowed(inner) => &inner[1usize],
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_number_0_1<'_> {
    type FandangoType = ::fandango::lang::Concatenation<'static>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match STRUCTURE.inner().statements() {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[2usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Statement::Production(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        .alternative()
        .inner()
        .concatenations()
        {
            ::std::borrow::Cow::Borrowed(inner) => &inner[1usize],
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_number_0<'_> {
    type FandangoType = ::fandango::lang::Alternative<'static>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match STRUCTURE.inner().statements() {
                ::std::borrow::Cow::Borrowed(inner) => &inner[2usize],
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Statement::Production(c) => c,
            _ => unreachable!(),
        }
    })
    .inner()
    .alternative();
}
impl ::fandango::typing::Structured for nonterminal_non_zero_0_0<'_> {
    type FandangoType = ::std::borrow::Cow<'static, str>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[3usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Symbol(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::String(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_non_zero_0_1<'_> {
    type FandangoType = ::std::borrow::Cow<'static, str>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[3usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[1usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Symbol(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::String(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_non_zero_0_2<'_> {
    type FandangoType = ::std::borrow::Cow<'static, str>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[3usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[2usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Symbol(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::String(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_non_zero_0_3<'_> {
    type FandangoType = ::std::borrow::Cow<'static, str>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[3usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[3usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Symbol(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::String(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_non_zero_0_4<'_> {
    type FandangoType = ::std::borrow::Cow<'static, str>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[3usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[4usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Symbol(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::String(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_non_zero_0_5<'_> {
    type FandangoType = ::std::borrow::Cow<'static, str>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[3usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[5usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Symbol(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::String(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_non_zero_0_6<'_> {
    type FandangoType = ::std::borrow::Cow<'static, str>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[3usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[6usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Symbol(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::String(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_non_zero_0_7<'_> {
    type FandangoType = ::std::borrow::Cow<'static, str>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[3usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[7usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Symbol(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::String(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_non_zero_0_8<'_> {
    type FandangoType = ::std::borrow::Cow<'static, str>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[3usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[8usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Symbol(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::String(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_non_zero_0<'_> {
    type FandangoType = ::fandango::lang::Alternative<'static>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match STRUCTURE.inner().statements() {
                ::std::borrow::Cow::Borrowed(inner) => &inner[3usize],
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Statement::Production(c) => c,
            _ => unreachable!(),
        }
    })
    .inner()
    .alternative();
}
impl ::fandango::typing::Structured for nonterminal_digit_0_0<'_> {
    type FandangoType = ::std::borrow::Cow<'static, str>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match ({
                match ({
                    match ({
                        match ({
                            match STRUCTURE.inner().statements() {
                                ::std::borrow::Cow::Borrowed(inner) => &inner[4usize],
                                _ => unreachable!(),
                            }
                        })
                        .inner()
                        {
                            ::fandango::lang::Statement::Production(c) => c,
                            _ => unreachable!(),
                        }
                    })
                    .inner()
                    .alternative()
                    .inner()
                    .concatenations()
                    {
                        ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                        _ => unreachable!(),
                    }
                })
                .inner()
                .operators()
                {
                    ::std::borrow::Cow::Borrowed(inner) => &inner[0usize],
                    _ => unreachable!(),
                }
            })
            .inner()
            {
                ::fandango::lang::Operator::Symbol(c) => c,
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Symbol::String(c) => c,
            _ => unreachable!(),
        }
    });
}
impl ::fandango::typing::Structured for nonterminal_digit_0<'_> {
    type FandangoType = ::fandango::lang::Alternative<'static>;
    const STRUCTURE: &'static ::fandango::lang::Tagged<'static, Self::FandangoType> = ({
        match ({
            match STRUCTURE.inner().statements() {
                ::std::borrow::Cow::Borrowed(inner) => &inner[4usize],
                _ => unreachable!(),
            }
        })
        .inner()
        {
            ::fandango::lang::Statement::Production(c) => c,
            _ => unreachable!(),
        }
    })
    .inner()
    .alternative();
}
#[derive(Clone, Debug)]
pub enum Type<'program, 'source>
where
    'source: 'program,
{
    nonterminal_number(&'program nonterminal_number<'source>),
    nonterminal_number_0_1(&'program nonterminal_number_0_1<'source>),
    nonterminal_non_zero_0_4(&'program nonterminal_non_zero_0_4<'source>),
    nonterminal_non_zero_0_5(&'program nonterminal_non_zero_0_5<'source>),
    nonterminal_non_zero_0(&'program nonterminal_non_zero_0<'source>),
    nonterminal_number_0_0(&'program nonterminal_number_0_0<'source>),
    nonterminal_non_zero(&'program nonterminal_non_zero<'source>),
    nonterminal_digit_0_0(&'program nonterminal_digit_0_0<'source>),
    nonterminal_start(&'program nonterminal_start<'source>),
    nonterminal_non_zero_0_6(&'program nonterminal_non_zero_0_6<'source>),
    nonterminal_number_0(&'program nonterminal_number_0<'source>),
    nonterminal_expr_0(&'program nonterminal_expr_0<'source>),
    nonterminal_expr_0_0(&'program nonterminal_expr_0_0<'source>),
    nonterminal_expr(&'program nonterminal_expr<'source>),
    nonterminal_non_zero_0_1(&'program nonterminal_non_zero_0_1<'source>),
    nonterminal_digit(&'program nonterminal_digit<'source>),
    nonterminal_non_zero_0_3(&'program nonterminal_non_zero_0_3<'source>),
    nonterminal_non_zero_0_2(&'program nonterminal_non_zero_0_2<'source>),
    nonterminal_non_zero_0_8(&'program nonterminal_non_zero_0_8<'source>),
    nonterminal_non_zero_0_0(&'program nonterminal_non_zero_0_0<'source>),
    nonterminal_number_0_1_1(&'program nonterminal_number_0_1_1<'source>),
    nonterminal_expr_0_0_1(&'program nonterminal_expr_0_0_1<'source>),
    nonterminal_digit_0(&'program nonterminal_digit_0<'source>),
    nonterminal_non_zero_0_7(&'program nonterminal_non_zero_0_7<'source>),
}
#[derive(Debug)]
pub enum TypeMut<'program, 'source>
where
    'source: 'program,
{
    nonterminal_number(&'program mut nonterminal_number<'source>),
    nonterminal_number_0_1(&'program mut nonterminal_number_0_1<'source>),
    nonterminal_non_zero_0_4(&'program mut nonterminal_non_zero_0_4<'source>),
    nonterminal_non_zero_0_5(&'program mut nonterminal_non_zero_0_5<'source>),
    nonterminal_non_zero_0(&'program mut nonterminal_non_zero_0<'source>),
    nonterminal_number_0_0(&'program mut nonterminal_number_0_0<'source>),
    nonterminal_non_zero(&'program mut nonterminal_non_zero<'source>),
    nonterminal_digit_0_0(&'program mut nonterminal_digit_0_0<'source>),
    nonterminal_start(&'program mut nonterminal_start<'source>),
    nonterminal_non_zero_0_6(&'program mut nonterminal_non_zero_0_6<'source>),
    nonterminal_number_0(&'program mut nonterminal_number_0<'source>),
    nonterminal_expr_0(&'program mut nonterminal_expr_0<'source>),
    nonterminal_expr_0_0(&'program mut nonterminal_expr_0_0<'source>),
    nonterminal_expr(&'program mut nonterminal_expr<'source>),
    nonterminal_non_zero_0_1(&'program mut nonterminal_non_zero_0_1<'source>),
    nonterminal_digit(&'program mut nonterminal_digit<'source>),
    nonterminal_non_zero_0_3(&'program mut nonterminal_non_zero_0_3<'source>),
    nonterminal_non_zero_0_2(&'program mut nonterminal_non_zero_0_2<'source>),
    nonterminal_non_zero_0_8(&'program mut nonterminal_non_zero_0_8<'source>),
    nonterminal_non_zero_0_0(&'program mut nonterminal_non_zero_0_0<'source>),
    nonterminal_number_0_1_1(&'program mut nonterminal_number_0_1_1<'source>),
    nonterminal_expr_0_0_1(&'program mut nonterminal_expr_0_0_1<'source>),
    nonterminal_digit_0(&'program mut nonterminal_digit_0<'source>),
    nonterminal_non_zero_0_7(&'program mut nonterminal_non_zero_0_7<'source>),
}
impl<'program, 'source> From<TypeMut<'program, 'source>> for Type<'program, 'source>
where
    'source: 'program,
{
    fn from(mutable: TypeMut<'program, 'source>) -> Type<'program, 'source> {
        match mutable {
            TypeMut::nonterminal_number(n) => Type::nonterminal_number(n),
            TypeMut::nonterminal_number_0_1(n) => Type::nonterminal_number_0_1(n),
            TypeMut::nonterminal_non_zero_0_4(n) => Type::nonterminal_non_zero_0_4(n),
            TypeMut::nonterminal_non_zero_0_5(n) => Type::nonterminal_non_zero_0_5(n),
            TypeMut::nonterminal_non_zero_0(n) => Type::nonterminal_non_zero_0(n),
            TypeMut::nonterminal_number_0_0(n) => Type::nonterminal_number_0_0(n),
            TypeMut::nonterminal_non_zero(n) => Type::nonterminal_non_zero(n),
            TypeMut::nonterminal_digit_0_0(n) => Type::nonterminal_digit_0_0(n),
            TypeMut::nonterminal_start(n) => Type::nonterminal_start(n),
            TypeMut::nonterminal_non_zero_0_6(n) => Type::nonterminal_non_zero_0_6(n),
            TypeMut::nonterminal_number_0(n) => Type::nonterminal_number_0(n),
            TypeMut::nonterminal_expr_0(n) => Type::nonterminal_expr_0(n),
            TypeMut::nonterminal_expr_0_0(n) => Type::nonterminal_expr_0_0(n),
            TypeMut::nonterminal_expr(n) => Type::nonterminal_expr(n),
            TypeMut::nonterminal_non_zero_0_1(n) => Type::nonterminal_non_zero_0_1(n),
            TypeMut::nonterminal_digit(n) => Type::nonterminal_digit(n),
            TypeMut::nonterminal_non_zero_0_3(n) => Type::nonterminal_non_zero_0_3(n),
            TypeMut::nonterminal_non_zero_0_2(n) => Type::nonterminal_non_zero_0_2(n),
            TypeMut::nonterminal_non_zero_0_8(n) => Type::nonterminal_non_zero_0_8(n),
            TypeMut::nonterminal_non_zero_0_0(n) => Type::nonterminal_non_zero_0_0(n),
            TypeMut::nonterminal_number_0_1_1(n) => Type::nonterminal_number_0_1_1(n),
            TypeMut::nonterminal_expr_0_0_1(n) => Type::nonterminal_expr_0_0_1(n),
            TypeMut::nonterminal_digit_0(n) => Type::nonterminal_digit_0(n),
            TypeMut::nonterminal_non_zero_0_7(n) => Type::nonterminal_non_zero_0_7(n),
        }
    }
}
pub const STRUCTURE: &'static ::fandango::lang::Tagged<'static, ::fandango::lang::Program> =
    &::fandango::lang::Tagged::known(
        ::fandango::lang::Program::known(FANDANGO_ARRAY_0),
        SOURCE,
        0usize,
        323usize,
        12678565929550771045u64,
    );
pub const _PEST_SOURCE: &'static str = "lit_7821370793304450885 = { \"+\" }\nlit_9524377253765094652 = { \"0\" }\nlit_15996882599689774755 = { \"1\" }\nlit_1688230434390951827 = { \"2\" }\nlit_71628934819250136 = { \"3\" }\nlit_6714267926367760633 = { \"4\" }\nlit_12391330923868597711 = { \"5\" }\nlit_84863073120411398 = { \"6\" }\nlit_10947813778151601303 = { \"7\" }\nlit_1325535459589835608 = { \"8\" }\nlit_8163090209484044995 = { \"9\" }\nstart = { SOI ~ expr ~ EOI }\nexpr = { expr_0 }\nexpr_0 = { expr_0_0 | number }\nexpr_0_0 = { number ~ expr_0_0_1 ~ expr }\nnumber = { number_0 }\nnumber_0 = { number_0_0 | number_0_1 }\nnumber_0_0 = { lit_9524377253765094652 }\nnumber_0_1 = { non_zero ~ number_0_1_1 }\nnon_zero = { non_zero_0 }\nnon_zero_0 = { non_zero_0_0 | non_zero_0_1 | non_zero_0_2 | non_zero_0_3 | non_zero_0_4 | non_zero_0_5 | non_zero_0_6 | non_zero_0_7 | non_zero_0_8 }\nnon_zero_0_0 = { lit_15996882599689774755 }\nnon_zero_0_1 = { lit_1688230434390951827 }\nnon_zero_0_2 = { lit_71628934819250136 }\nnon_zero_0_3 = { lit_6714267926367760633 }\nnon_zero_0_4 = { lit_12391330923868597711 }\nnon_zero_0_5 = { lit_84863073120411398 }\nnon_zero_0_6 = { lit_10947813778151601303 }\nnon_zero_0_7 = { lit_1325535459589835608 }\nnon_zero_0_8 = { lit_8163090209484044995 }\nnumber_0_1_1 = { digit* }\ndigit = { digit_0 }\ndigit_0 = { digit_0_0 | non_zero }\ndigit_0_0 = { lit_9524377253765094652 }\nexpr_0_0_1 = { lit_7821370793304450885 }\n";
impl Simple {
    pub fn extract<'source>(
        source: &'source str,
    ) -> ::std::result::Result<nonterminal_start<'_>, ParseError> {
        use ::fandango::Parser;
        let (grammar,) = {
            let iter = &mut (Simple::parse(Rule::start, source)?);
            let out = ({
                let tmp = iter.next().unwrap();
                if cfg!(debug_assertions) {
                    let value = (tmp.as_rule());
                    #[allow(unreachable_patterns)]
                    match value {
                        Rule::start => {}
                        _ => panic!(
                            "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                            stringify!(Rule::start),
                            value
                        ),
                    }
                }
                tmp
            },);
            if cfg!(debug_assertions) {
                let value = (iter.next());
                #[allow(unreachable_patterns)]
                match value {
                    Option::None => {}
                    _ => panic!(
                        "assertion failed: `(value matches pattern)`
 pattern: `{}`,
   value: `{:?}`",
                        stringify!(Option::None),
                        value
                    ),
                }
            }
            out
        };
        let source = ::std::rc::Rc::new(::std::borrow::Cow::Borrowed(source));
        nonterminal_start::try_from((source, grammar))
    }
}
#[allow(dead_code, non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Rule {
    #[doc = "End-of-input"]
    EOI,
    r#lit_7821370793304450885,
    r#lit_9524377253765094652,
    r#lit_15996882599689774755,
    r#lit_1688230434390951827,
    r#lit_71628934819250136,
    r#lit_6714267926367760633,
    r#lit_12391330923868597711,
    r#lit_84863073120411398,
    r#lit_10947813778151601303,
    r#lit_1325535459589835608,
    r#lit_8163090209484044995,
    r#start,
    r#expr,
    r#expr_0,
    r#expr_0_0,
    r#number,
    r#number_0,
    r#number_0_0,
    r#number_0_1,
    r#non_zero,
    r#non_zero_0,
    r#non_zero_0_0,
    r#non_zero_0_1,
    r#non_zero_0_2,
    r#non_zero_0_3,
    r#non_zero_0_4,
    r#non_zero_0_5,
    r#non_zero_0_6,
    r#non_zero_0_7,
    r#non_zero_0_8,
    r#number_0_1_1,
    r#digit,
    r#digit_0,
    r#digit_0_0,
    r#expr_0_0_1,
}
impl Rule {
    pub fn all_rules() -> &'static [Rule] {
        &[
            Rule::r#lit_7821370793304450885,
            Rule::r#lit_9524377253765094652,
            Rule::r#lit_15996882599689774755,
            Rule::r#lit_1688230434390951827,
            Rule::r#lit_71628934819250136,
            Rule::r#lit_6714267926367760633,
            Rule::r#lit_12391330923868597711,
            Rule::r#lit_84863073120411398,
            Rule::r#lit_10947813778151601303,
            Rule::r#lit_1325535459589835608,
            Rule::r#lit_8163090209484044995,
            Rule::r#start,
            Rule::r#expr,
            Rule::r#expr_0,
            Rule::r#expr_0_0,
            Rule::r#number,
            Rule::r#number_0,
            Rule::r#number_0_0,
            Rule::r#number_0_1,
            Rule::r#non_zero,
            Rule::r#non_zero_0,
            Rule::r#non_zero_0_0,
            Rule::r#non_zero_0_1,
            Rule::r#non_zero_0_2,
            Rule::r#non_zero_0_3,
            Rule::r#non_zero_0_4,
            Rule::r#non_zero_0_5,
            Rule::r#non_zero_0_6,
            Rule::r#non_zero_0_7,
            Rule::r#non_zero_0_8,
            Rule::r#number_0_1_1,
            Rule::r#digit,
            Rule::r#digit_0,
            Rule::r#digit_0_0,
            Rule::r#expr_0_0_1,
        ]
    }
}
#[allow(clippy::all)]
impl ::fandango::Parser<Rule> for Simple {
    fn parse<'i>(
        rule: Rule,
        input: &'i str,
    ) -> ::std::result::Result<::pest::iterators::Pairs<'i, Rule>, ::pest::error::Error<Rule>> {
        mod rules {
            #![allow(clippy::upper_case_acronyms)]
            pub mod hidden {
                use super::super::Rule;
                #[inline]
                #[allow(dead_code, non_snake_case, unused_variables)]
                pub fn skip(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    Ok(state)
                }
            }
            pub mod visible {
                use super::super::Rule;
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#lit_7821370793304450885(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#lit_7821370793304450885, |state| {
                        state.match_string("+")
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#lit_9524377253765094652(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#lit_9524377253765094652, |state| {
                        state.match_string("0")
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#lit_15996882599689774755(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#lit_15996882599689774755, |state| {
                        state.match_string("1")
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#lit_1688230434390951827(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#lit_1688230434390951827, |state| {
                        state.match_string("2")
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#lit_71628934819250136(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#lit_71628934819250136, |state| {
                        state.match_string("3")
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#lit_6714267926367760633(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#lit_6714267926367760633, |state| {
                        state.match_string("4")
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#lit_12391330923868597711(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#lit_12391330923868597711, |state| {
                        state.match_string("5")
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#lit_84863073120411398(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#lit_84863073120411398, |state| {
                        state.match_string("6")
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#lit_10947813778151601303(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#lit_10947813778151601303, |state| {
                        state.match_string("7")
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#lit_1325535459589835608(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#lit_1325535459589835608, |state| {
                        state.match_string("8")
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#lit_8163090209484044995(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#lit_8163090209484044995, |state| {
                        state.match_string("9")
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#start(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#start, |state| {
                        state.sequence(|state| {
                            self::r#SOI(state)
                                .and_then(|state| super::hidden::skip(state))
                                .and_then(|state| self::r#expr(state))
                                .and_then(|state| super::hidden::skip(state))
                                .and_then(|state| self::r#EOI(state))
                        })
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#expr(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#expr, |state| self::r#expr_0(state))
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#expr_0(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#expr_0, |state| {
                        self::r#expr_0_0(state).or_else(|state| self::r#number(state))
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#expr_0_0(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#expr_0_0, |state| {
                        state.sequence(|state| {
                            self::r#number(state)
                                .and_then(|state| super::hidden::skip(state))
                                .and_then(|state| self::r#expr_0_0_1(state))
                                .and_then(|state| super::hidden::skip(state))
                                .and_then(|state| self::r#expr(state))
                        })
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#number(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#number, |state| self::r#number_0(state))
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#number_0(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#number_0, |state| {
                        self::r#number_0_0(state).or_else(|state| self::r#number_0_1(state))
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#number_0_0(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#number_0_0, |state| {
                        self::r#lit_9524377253765094652(state)
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#number_0_1(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#number_0_1, |state| {
                        state.sequence(|state| {
                            self::r#non_zero(state)
                                .and_then(|state| super::hidden::skip(state))
                                .and_then(|state| self::r#number_0_1_1(state))
                        })
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#non_zero(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#non_zero, |state| self::r#non_zero_0(state))
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#non_zero_0(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#non_zero_0, |state| {
                        self::r#non_zero_0_0(state)
                            .or_else(|state| self::r#non_zero_0_1(state))
                            .or_else(|state| self::r#non_zero_0_2(state))
                            .or_else(|state| self::r#non_zero_0_3(state))
                            .or_else(|state| self::r#non_zero_0_4(state))
                            .or_else(|state| self::r#non_zero_0_5(state))
                            .or_else(|state| self::r#non_zero_0_6(state))
                            .or_else(|state| self::r#non_zero_0_7(state))
                            .or_else(|state| self::r#non_zero_0_8(state))
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#non_zero_0_0(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#non_zero_0_0, |state| {
                        self::r#lit_15996882599689774755(state)
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#non_zero_0_1(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#non_zero_0_1, |state| {
                        self::r#lit_1688230434390951827(state)
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#non_zero_0_2(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#non_zero_0_2, |state| {
                        self::r#lit_71628934819250136(state)
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#non_zero_0_3(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#non_zero_0_3, |state| {
                        self::r#lit_6714267926367760633(state)
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#non_zero_0_4(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#non_zero_0_4, |state| {
                        self::r#lit_12391330923868597711(state)
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#non_zero_0_5(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#non_zero_0_5, |state| {
                        self::r#lit_84863073120411398(state)
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#non_zero_0_6(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#non_zero_0_6, |state| {
                        self::r#lit_10947813778151601303(state)
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#non_zero_0_7(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#non_zero_0_7, |state| {
                        self::r#lit_1325535459589835608(state)
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#non_zero_0_8(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#non_zero_0_8, |state| {
                        self::r#lit_8163090209484044995(state)
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#number_0_1_1(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#number_0_1_1, |state| {
                        state.sequence(|state| {
                            state.optional(|state| {
                                self::r#digit(state).and_then(|state| {
                                    state.repeat(|state| {
                                        state.sequence(|state| {
                                            super::hidden::skip(state)
                                                .and_then(|state| self::r#digit(state))
                                        })
                                    })
                                })
                            })
                        })
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#digit(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#digit, |state| self::r#digit_0(state))
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#digit_0(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#digit_0, |state| {
                        self::r#digit_0_0(state).or_else(|state| self::r#non_zero(state))
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#digit_0_0(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#digit_0_0, |state| {
                        self::r#lit_9524377253765094652(state)
                    })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#expr_0_0_1(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::r#expr_0_0_1, |state| {
                        self::r#lit_7821370793304450885(state)
                    })
                }
                #[inline]
                #[allow(dead_code, non_snake_case, unused_variables)]
                pub fn EOI(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.rule(Rule::EOI, |state| state.end_of_input())
                }
                #[inline]
                #[allow(dead_code, non_snake_case, unused_variables)]
                pub fn SOI(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<::std::boxed::Box<::pest::ParserState<'_, Rule>>>
                {
                    state.start_of_input()
                }
            }
            pub use self::visible::*;
        }
        ::pest::state(input, |state| match rule {
            Rule::r#lit_7821370793304450885 => rules::r#lit_7821370793304450885(state),
            Rule::r#lit_9524377253765094652 => rules::r#lit_9524377253765094652(state),
            Rule::r#lit_15996882599689774755 => rules::r#lit_15996882599689774755(state),
            Rule::r#lit_1688230434390951827 => rules::r#lit_1688230434390951827(state),
            Rule::r#lit_71628934819250136 => rules::r#lit_71628934819250136(state),
            Rule::r#lit_6714267926367760633 => rules::r#lit_6714267926367760633(state),
            Rule::r#lit_12391330923868597711 => rules::r#lit_12391330923868597711(state),
            Rule::r#lit_84863073120411398 => rules::r#lit_84863073120411398(state),
            Rule::r#lit_10947813778151601303 => rules::r#lit_10947813778151601303(state),
            Rule::r#lit_1325535459589835608 => rules::r#lit_1325535459589835608(state),
            Rule::r#lit_8163090209484044995 => rules::r#lit_8163090209484044995(state),
            Rule::r#start => rules::r#start(state),
            Rule::r#expr => rules::r#expr(state),
            Rule::r#expr_0 => rules::r#expr_0(state),
            Rule::r#expr_0_0 => rules::r#expr_0_0(state),
            Rule::r#number => rules::r#number(state),
            Rule::r#number_0 => rules::r#number_0(state),
            Rule::r#number_0_0 => rules::r#number_0_0(state),
            Rule::r#number_0_1 => rules::r#number_0_1(state),
            Rule::r#non_zero => rules::r#non_zero(state),
            Rule::r#non_zero_0 => rules::r#non_zero_0(state),
            Rule::r#non_zero_0_0 => rules::r#non_zero_0_0(state),
            Rule::r#non_zero_0_1 => rules::r#non_zero_0_1(state),
            Rule::r#non_zero_0_2 => rules::r#non_zero_0_2(state),
            Rule::r#non_zero_0_3 => rules::r#non_zero_0_3(state),
            Rule::r#non_zero_0_4 => rules::r#non_zero_0_4(state),
            Rule::r#non_zero_0_5 => rules::r#non_zero_0_5(state),
            Rule::r#non_zero_0_6 => rules::r#non_zero_0_6(state),
            Rule::r#non_zero_0_7 => rules::r#non_zero_0_7(state),
            Rule::r#non_zero_0_8 => rules::r#non_zero_0_8(state),
            Rule::r#number_0_1_1 => rules::r#number_0_1_1(state),
            Rule::r#digit => rules::r#digit(state),
            Rule::r#digit_0 => rules::r#digit_0(state),
            Rule::r#digit_0_0 => rules::r#digit_0_0(state),
            Rule::r#expr_0_0_1 => rules::r#expr_0_0_1(state),
            Rule::EOI => rules::EOI(state),
        })
    }
}
