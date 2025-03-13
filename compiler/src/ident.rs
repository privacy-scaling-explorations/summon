#[derive(Debug)]
pub struct Ident {
  pub sym: swc_atoms::Atom,
  pub span: swc_common::Span,
}

impl Ident {
  pub fn from_swc_ident(ident: &swc_ecma_ast::IdentName) -> Self {
    Ident {
      sym: ident.sym.clone(),
      span: ident.span,
    }
  }

  pub fn this(span: swc_common::Span) -> Self {
    Ident {
      sym: swc_atoms::Atom::from("this"),
      span,
    }
  }
}
