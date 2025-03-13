pub fn expr_from_simple_assign_target(
  simple_assign_target: &swc_ecma_ast::SimpleAssignTarget,
) -> swc_ecma_ast::Expr {
  use swc_ecma_ast::Expr;

  match simple_assign_target {
    swc_ecma_ast::SimpleAssignTarget::Ident(binding_ident) => Expr::Ident(binding_ident.id.clone()),
    swc_ecma_ast::SimpleAssignTarget::Member(member_expr) => Expr::Member(member_expr.clone()),
    swc_ecma_ast::SimpleAssignTarget::SuperProp(super_prop_expr) => {
      Expr::SuperProp(super_prop_expr.clone())
    }
    swc_ecma_ast::SimpleAssignTarget::Paren(paren_expr) => Expr::Paren(paren_expr.clone()),
    swc_ecma_ast::SimpleAssignTarget::OptChain(opt_chain_expr) => {
      Expr::OptChain(opt_chain_expr.clone())
    }
    swc_ecma_ast::SimpleAssignTarget::TsAs(ts_as_expr) => Expr::TsAs(ts_as_expr.clone()),
    swc_ecma_ast::SimpleAssignTarget::TsSatisfies(ts_satisfies_expr) => {
      Expr::TsSatisfies(ts_satisfies_expr.clone())
    }
    swc_ecma_ast::SimpleAssignTarget::TsNonNull(ts_non_null_expr) => {
      Expr::TsNonNull(ts_non_null_expr.clone())
    }
    swc_ecma_ast::SimpleAssignTarget::TsTypeAssertion(ts_type_assertion) => {
      Expr::TsTypeAssertion(ts_type_assertion.clone())
    }
    swc_ecma_ast::SimpleAssignTarget::TsInstantiation(ts_instantiation) => {
      Expr::TsInstantiation(ts_instantiation.clone())
    }
    swc_ecma_ast::SimpleAssignTarget::Invalid(invalid) => Expr::Invalid(invalid.clone()),
  }
}

pub fn pat_from_assign_target_pat(
  assign_target_pat: &swc_ecma_ast::AssignTargetPat,
) -> swc_ecma_ast::Pat {
  use swc_ecma_ast::Pat;

  match assign_target_pat {
    swc_ecma_ast::AssignTargetPat::Array(array_pat) => Pat::Array(array_pat.clone()),
    swc_ecma_ast::AssignTargetPat::Object(object_pat) => Pat::Object(object_pat.clone()),
    swc_ecma_ast::AssignTargetPat::Invalid(invalid) => Pat::Invalid(invalid.clone()),
  }
}
