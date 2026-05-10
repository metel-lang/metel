use std::collections::HashMap;
use std::collections::HashSet;

use crate::ast::*;
use crate::error::{ErrorCode, YoloscriptError};
use crate::typed_ast::*;
use crate::typeinference::*;
use crate::types::Type;

type SchemeEnv = HashMap<String, TypeScheme>;

/// Run the type checker over an untyped AST, producing a fully typed AST.
pub fn check(program: Program) -> Result<TypedProgram, YoloscriptError> {
    let mut ctx = InferContext::new();

    // Pre-pass: hoist top-level function names so forward references work.
    hoist_fun_decls(&program.decls, &mut ctx);

    // Pass 1: walk AST, emit constraints, collect pending generalisations.
    let mut pending: Vec<PendingFun> = vec![];
    infer_program(&program, &mut ctx, &mut pending)?;
    let subst = ctx.solve()?;

    // Build SchemeEnv by applying the substitution and generalising.
    let mut scheme_env: SchemeEnv = HashMap::new();
    for pf in pending {
        let resolved = subst.apply(&pf.fun_ty);
        let scheme = generalize(resolved, &pf.env_fvs);
        scheme_env.insert(pf.name, scheme);
    }

    // Pass 2: re-derive concrete types and build TypedAST.
    construct_program(&program, &subst, &scheme_env)
}

// ── Pending generalisation record ─────────────────────────────────────────────

struct PendingFun {
    name:    String,
    fun_ty:  InferType,
    env_fvs: HashSet<TypeVar>,
}

// ── Pre-pass: function hoisting ───────────────────────────────────────────────

/// Register the names of all direct `FunDecl`s in `decls` with fresh type
/// variables so that forward references and mutual recursion work.
fn hoist_fun_decls(decls: &[Decl], ctx: &mut InferContext) {
    for decl in decls {
        if let Decl::Fun(fun) = decl {
            if fun.generics.is_empty() {
                let fresh = ctx.fresh_var();
                ctx.bind_mono(&fun.name, fresh);
            }
        }
    }
}

// ── Pass 1: type inference ────────────────────────────────────────────────────

fn infer_program(
    program: &Program,
    ctx: &mut InferContext,
    pending: &mut Vec<PendingFun>,
) -> Result<(), YoloscriptError> {
    for decl in &program.decls {
        infer_decl(decl, ctx, pending)?;
    }
    Ok(())
}

fn infer_decl(
    decl: &Decl,
    ctx: &mut InferContext,
    pending: &mut Vec<PendingFun>,
) -> Result<(), YoloscriptError> {
    match decl {
        Decl::Let(ld) => {
            let val_ty = infer_expr(&ld.value, ctx)?;
            if let Some(ann) = &ld.type_ann {
                ctx.add_constraint(val_ty.clone(), type_expr_to_infer(ann), ld.span.clone());
            }
            ctx.bind_mono(&ld.name, val_ty);
            Ok(())
        }
        Decl::Mut(md) => {
            let val_ty = infer_expr(&md.value, ctx)?;
            if let Some(ann) = &md.type_ann {
                ctx.add_constraint(val_ty.clone(), type_expr_to_infer(ann), md.span.clone());
            }
            ctx.bind_mono(&md.name, val_ty);
            Ok(())
        }
        Decl::Fun(fd) => infer_fun_decl(fd, ctx, pending),
        Decl::Struct(_) | Decl::Enum(_) | Decl::Trait(_) => Ok(()),
        Decl::Impl(_) => Err(YoloscriptError::internal("impl blocks not yet supported")),
        Decl::Stmt(stmt) => infer_stmt(stmt, ctx, pending),
    }
}

fn infer_fun_decl(
    fun: &FunDecl,
    ctx: &mut InferContext,
    pending: &mut Vec<PendingFun>,
) -> Result<(), YoloscriptError> {
    if !fun.generics.is_empty() {
        return Err(YoloscriptError::internal(format!(
            "generic function `{}` not yet supported",
            fun.name
        )));
    }

    // Param types: use annotation if present, otherwise a fresh variable.
    let param_types: Vec<InferType> = fun.params.iter().map(|p| {
        if let Some(ann) = &p.type_ann { type_expr_to_infer(ann) } else { ctx.fresh_var() }
    }).collect();

    // Return type: use annotation if present, otherwise a fresh variable.
    let ret_ty = if let Some(ann) = &fun.return_type {
        type_expr_to_infer(ann)
    } else {
        ctx.fresh_var()
    };

    // Capture env free vars before entering the function scope (used for generalisation).
    let env_fvs = ctx.env_free_vars();

    ctx.push_scope();
    for (param, pt) in fun.params.iter().zip(param_types.iter()) {
        ctx.bind_mono(&param.name, pt.clone());
    }

    let saved_ret = ctx.push_return_type(ret_ty.clone());
    let body_ty = infer_block(&fun.body, ctx, pending)?;

    // The block's tail type must unify with the declared return type.
    ctx.add_constraint(body_ty, ret_ty.clone(), fun.body.span.clone());

    ctx.pop_return_type(saved_ret);
    ctx.pop_scope();

    let fun_ty = InferType::Fun(param_types, Box::new(ret_ty));

    // Unify with the pre-hoisted fresh variable registered during the pre-pass.
    if let Some(pre_reg) = ctx.lookup(&fun.name) {
        ctx.add_constraint(pre_reg, fun_ty.clone(), fun.span.clone());
    }

    pending.push(PendingFun { name: fun.name.clone(), fun_ty, env_fvs });
    Ok(())
}

fn infer_block(
    block: &Block,
    ctx: &mut InferContext,
    pending: &mut Vec<PendingFun>,
) -> Result<InferType, YoloscriptError> {
    hoist_fun_decls(&block.stmts, ctx);
    for stmt in &block.stmts {
        infer_decl(stmt, ctx, pending)?;
    }
    match &block.tail {
        Some(tail) => infer_expr(tail, ctx),
        None => Ok(InferType::unit()),
    }
}

fn infer_stmt(
    stmt: &Stmt,
    ctx: &mut InferContext,
    _pending: &mut Vec<PendingFun>,
) -> Result<(), YoloscriptError> {
    match stmt {
        Stmt::Expr(e) => { infer_expr(e, ctx)?; Ok(()) }
        Stmt::Return(r) => {
            let ret_ty = match &r.value {
                Some(e) => infer_expr(e, ctx)?,
                None    => InferType::unit(),
            };
            if let Some(expected) = ctx.current_return_type().cloned() {
                ctx.add_constraint(ret_ty, expected, r.span.clone());
            }
            Ok(())
        }
        _ => Err(YoloscriptError::internal("statement not yet supported")),
    }
}

fn infer_expr(expr: &Expr, ctx: &mut InferContext) -> Result<InferType, YoloscriptError> {
    match expr {
        Expr::Literal(lit, _)          => Ok(infer_literal(lit, ctx)),
        Expr::Ident(name, span)        => {
            ctx.lookup(name).ok_or_else(|| YoloscriptError::type_error(
                ErrorCode::E0003,
                format!("undefined name `{name}`"),
                span,
            ))
        }
        Expr::BinOp(lhs, op, rhs, span) => infer_binop(lhs, op, rhs, span, ctx),
        Expr::UnaryOp(op, operand, span) => infer_unaryop(op, operand, span, ctx),
        _ => Err(YoloscriptError::internal("expression not yet supported")),
    }
}

fn infer_literal(lit: &Literal, ctx: &mut InferContext) -> InferType {
    match lit {
        Literal::Int(_)   => InferType::int(),
        Literal::Float(_) => InferType::float(),
        Literal::Bool(_)  => InferType::bool(),
        Literal::Str(_)   => InferType::str(),
        Literal::Unit     => InferType::unit(),
        Literal::Nope     => InferType::Named("Perhaps".to_string(), vec![ctx.fresh_var()]),
    }
}

fn infer_binop(
    lhs: &Expr,
    op: &BinOp,
    rhs: &Expr,
    span: &Span,
    ctx: &mut InferContext,
) -> Result<InferType, YoloscriptError> {
    let lhs_ty = infer_expr(lhs, ctx)?;
    let rhs_ty = infer_expr(rhs, ctx)?;
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
            let result = ctx.fresh_var();
            ctx.add_constraint(lhs_ty, result.clone(), span.clone());
            ctx.add_constraint(rhs_ty, result.clone(), span.clone());
            Ok(result)
        }
        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            ctx.add_constraint(lhs_ty, rhs_ty, span.clone());
            Ok(InferType::bool())
        }
        BinOp::And | BinOp::Or => {
            ctx.add_constraint(lhs_ty, InferType::bool(), span.clone());
            ctx.add_constraint(rhs_ty, InferType::bool(), span.clone());
            Ok(InferType::bool())
        }
        BinOp::Range | BinOp::RangeInclusive => {
            ctx.add_constraint(lhs_ty, InferType::int(), span.clone());
            ctx.add_constraint(rhs_ty, InferType::int(), span.clone());
            Ok(InferType::Named("Range".to_string(), vec![InferType::int()]))
        }
    }
}

fn infer_unaryop(
    op: &UnaryOp,
    operand: &Expr,
    span: &Span,
    ctx: &mut InferContext,
) -> Result<InferType, YoloscriptError> {
    let ty = infer_expr(operand, ctx)?;
    match op {
        UnaryOp::Neg => Ok(ty),
        UnaryOp::Not => {
            ctx.add_constraint(ty, InferType::bool(), span.clone());
            Ok(InferType::bool())
        }
    }
}

// ── Type conversions ──────────────────────────────────────────────────────────

/// Convert a source-level `TypeExpr` to an `InferType` for use during inference.
fn type_expr_to_infer(te: &TypeExpr) -> InferType {
    match te {
        TypeExpr::Named(name, args) => {
            let arg_tys: Vec<_> = args.iter().map(type_expr_to_infer).collect();
            match (name.as_str(), arg_tys.len()) {
                ("Int",    0) => InferType::int(),
                ("Float",  0) => InferType::float(),
                ("Bool",   0) => InferType::bool(),
                ("String", 0) => InferType::str(),
                ("Never",  0) => InferType::never(),
                _             => InferType::Named(name.clone(), arg_tys),
            }
        }
        TypeExpr::Unit         => InferType::unit(),
        TypeExpr::Tuple(ts)    => InferType::Tuple(ts.iter().map(type_expr_to_infer).collect()),
        TypeExpr::Array(t)     => InferType::Array(Box::new(type_expr_to_infer(t))),
        TypeExpr::Fun(ps, ret) => InferType::Fun(
            ps.iter().map(type_expr_to_infer).collect(),
            Box::new(ret.as_deref().map(type_expr_to_infer).unwrap_or(InferType::unit())),
        ),
    }
}

/// Convert a fully-solved `InferType` to a concrete `Type`.
/// Returns E0002 if any type variable is still unresolved.
fn infer_type_to_type(ty: &InferType, span: &Span) -> Result<Type, YoloscriptError> {
    match ty {
        InferType::Concrete(t) => Ok(t.clone()),
        InferType::Never       => Ok(Type::Never),
        InferType::Var(_)      => Err(YoloscriptError::type_error(
            ErrorCode::E0002,
            "cannot infer type; add a type annotation",
            span,
        )),
        InferType::Fun(params, ret) => {
            let p: Result<Vec<_>, _> = params.iter().map(|p| infer_type_to_type(p, span)).collect();
            Ok(Type::Fun(p?, Box::new(infer_type_to_type(ret, span)?)))
        }
        InferType::Tuple(ts) => {
            let t: Result<Vec<_>, _> = ts.iter().map(|t| infer_type_to_type(t, span)).collect();
            Ok(Type::Tuple(t?))
        }
        InferType::Array(t) => Ok(Type::Array(Box::new(infer_type_to_type(t, span)?))),
        InferType::Named(name, args) => {
            let a: Result<Vec<_>, _> = args.iter().map(|a| infer_type_to_type(a, span)).collect();
            let args = a?;
            match (name.as_str(), args.len()) {
                ("Perhaps", 1) => Ok(Type::Perhaps(Box::new(args.into_iter().next().unwrap()))),
                ("Result",  2) => {
                    let mut it = args.into_iter();
                    Ok(Type::Result(Box::new(it.next().unwrap()), Box::new(it.next().unwrap())))
                }
                _ => Ok(Type::Named(name.clone(), args)),
            }
        }
    }
}

fn resolved_to_type(ty: &InferType, subst: &Substitution, span: &Span) -> Result<Type, YoloscriptError> {
    infer_type_to_type(&subst.apply(ty), span)
}

// ── Pass 2: construction ──────────────────────────────────────────────────────

/// Scope-aware context for Pass 2. Mirrors InferContext's scope management but
/// holds concrete `Type` values; no constraint emission.
struct ConstructCtx<'a> {
    subst:      &'a Substitution,
    scheme_env: &'a SchemeEnv,
    env:        Vec<HashMap<String, Type>>,
}

impl<'a> ConstructCtx<'a> {
    fn new(subst: &'a Substitution, scheme_env: &'a SchemeEnv) -> Self {
        Self { subst, scheme_env, env: vec![HashMap::new()] }
    }

    fn push_scope(&mut self) { self.env.push(HashMap::new()); }
    fn pop_scope(&mut self)  { self.env.pop(); }

    fn bind(&mut self, name: impl Into<String>, ty: Type) {
        self.env.last_mut().unwrap().insert(name.into(), ty);
    }

    fn lookup(&self, name: &str) -> Option<&Type> {
        self.env.iter().rev().find_map(|s| s.get(name))
    }
}

fn construct_program(
    program:    &Program,
    subst:      &Substitution,
    scheme_env: &SchemeEnv,
) -> Result<TypedProgram, YoloscriptError> {
    let mut ctx = ConstructCtx::new(subst, scheme_env);

    // Hoist resolved function types so forward references work in Pass 2.
    for decl in &program.decls {
        if let Decl::Fun(fd) = decl {
            if let Some(scheme) = scheme_env.get(&fd.name) {
                if let Ok(ty) = infer_type_to_type(&scheme.ty, &fd.span) {
                    ctx.bind(&fd.name, ty);
                }
            }
        }
    }

    let mut out = vec![];
    for decl in &program.decls {
        out.push(construct_decl(decl, &mut ctx)?);
    }
    Ok(out)
}

fn construct_decl(decl: &Decl, ctx: &mut ConstructCtx) -> Result<TypedDecl, YoloscriptError> {
    match decl {
        Decl::Let(ld) => {
            let expected_ty = ld.type_ann.as_ref()
                .map(|ann| resolved_to_type(&type_expr_to_infer(ann), ctx.subst, &ld.span))
                .transpose()?;
            let value = construct_expr(&ld.value, expected_ty.as_ref(), ctx)?;
            let ty = expected_ty.unwrap_or_else(|| value.ty().clone());
            ctx.bind(&ld.name, ty);
            Ok(TypedDecl::Let(TypedLetDecl {
                name: ld.name.clone(), type_ann: ld.type_ann.clone(),
                value, span: ld.span.clone(),
            }))
        }
        Decl::Mut(md) => {
            let expected_ty = md.type_ann.as_ref()
                .map(|ann| resolved_to_type(&type_expr_to_infer(ann), ctx.subst, &md.span))
                .transpose()?;
            let value = construct_expr(&md.value, expected_ty.as_ref(), ctx)?;
            let ty = expected_ty.unwrap_or_else(|| value.ty().clone());
            ctx.bind(&md.name, ty);
            Ok(TypedDecl::Mut(TypedMutDecl {
                name: md.name.clone(), type_ann: md.type_ann.clone(),
                value, span: md.span.clone(),
            }))
        }
        Decl::Fun(fd)    => construct_fun_decl(fd, ctx),
        Decl::Struct(sd) => Ok(TypedDecl::Struct(TypedStructDecl {
            name: sd.name.clone(), generics: sd.generics.clone(),
            fields: sd.fields.clone(), span: sd.span.clone(),
        })),
        Decl::Enum(ed)   => Ok(TypedDecl::Enum(TypedEnumDecl {
            name: ed.name.clone(), generics: ed.generics.clone(),
            variants: ed.variants.clone(), span: ed.span.clone(),
        })),
        Decl::Impl(_) => Err(YoloscriptError::internal("impl blocks not yet supported")),
        Decl::Trait(td)  => Ok(TypedDecl::Trait(TypedTraitDecl {
            name: td.name.clone(), methods: td.methods.clone(), span: td.span.clone(),
        })),
        Decl::Stmt(stmt) => Ok(TypedDecl::Stmt(construct_stmt(stmt, ctx)?)),
    }
}

fn construct_fun_decl(fun: &FunDecl, ctx: &mut ConstructCtx) -> Result<TypedDecl, YoloscriptError> {
    let fun_ty = ctx.scheme_env.get(&fun.name)
        .ok_or_else(|| YoloscriptError::internal(format!("missing type for fn `{}`", fun.name)))
        .map(|s| &s.ty)?;

    let (param_types, _ret_type) = match fun_ty {
        InferType::Fun(params, ret) => {
            let pts: Result<Vec<_>, _> = params.iter()
                .map(|p| infer_type_to_type(p, &fun.span))
                .collect();
            (pts?, infer_type_to_type(ret, &fun.span)?)
        }
        _ => return Err(YoloscriptError::internal(format!("expected Fun type for `{}`", fun.name))),
    };

    ctx.push_scope();
    for (param, ty) in fun.params.iter().zip(param_types.iter()) {
        ctx.bind(&param.name, ty.clone());
    }

    let body = construct_block(&fun.body, ctx)?;
    ctx.pop_scope();

    Ok(TypedDecl::Fun(TypedFunDecl {
        name: fun.name.clone(), generics: fun.generics.clone(),
        params: fun.params.clone(), return_type: fun.return_type.clone(),
        body, span: fun.span.clone(),
    }))
}

fn construct_block(block: &Block, ctx: &mut ConstructCtx) -> Result<TypedBlock, YoloscriptError> {
    let mut stmts = vec![];
    for stmt in &block.stmts {
        stmts.push(construct_decl(stmt, ctx)?);
    }
    let tail = match &block.tail {
        Some(e) => Some(Box::new(construct_expr(e, None, ctx)?)),
        None    => None,
    };
    Ok(TypedBlock { stmts, tail, span: block.span.clone() })
}

fn construct_stmt(stmt: &Stmt, ctx: &mut ConstructCtx) -> Result<TypedStmt, YoloscriptError> {
    match stmt {
        Stmt::Expr(e) => Ok(TypedStmt::Expr(construct_expr(e, None, ctx)?)),
        Stmt::Return(r) => {
            let value = match &r.value {
                Some(e) => Some(construct_expr(e, None, ctx)?),
                None    => None,
            };
            Ok(TypedStmt::Return(TypedReturnStmt { value, span: r.span.clone() }))
        }
        _ => Err(YoloscriptError::internal("statement not yet supported in construct")),
    }
}

fn construct_expr(expr: &Expr, expected_ty: Option<&Type>, ctx: &mut ConstructCtx) -> Result<TypedExpr, YoloscriptError> {
    match expr {
        Expr::Literal(lit, span) => {
            let ty = construct_literal_type(lit, expected_ty, span)?;
            Ok(TypedExpr::Literal(lit.clone(), ty, span.clone()))
        }
        Expr::Ident(name, span) => {
            let ty = ctx.lookup(name).cloned().ok_or_else(|| YoloscriptError::type_error(
                ErrorCode::E0003,
                format!("undefined name `{name}`"),
                span,
            ))?;
            Ok(TypedExpr::Ident(name.clone(), ty, span.clone()))
        }
        Expr::BinOp(lhs, op, rhs, span) => construct_binop(lhs, op, rhs, span, ctx),
        Expr::UnaryOp(op, operand, span) => construct_unaryop(op, operand, span, ctx),
        _ => Err(YoloscriptError::internal("expression not yet supported in construct")),
    }
}


fn construct_literal_type(lit: &Literal, expected_ty: Option<&Type>, span: &Span) -> Result<Type, YoloscriptError> {
    match lit {
        Literal::Int(_)   => Ok(Type::Int),
        Literal::Float(_) => Ok(Type::Float),
        Literal::Bool(_)  => Ok(Type::Bool),
        Literal::Str(_)   => Ok(Type::Str),
        Literal::Unit     => Ok(Type::Unit),
        // nope's type cannot be re-derived from the literal alone. Pass 2 must receive
        // the expected type from the enclosing binding's annotation (propagated via
        // construct_expr's expected_ty parameter). If no annotation, E0002 — but Pass 1
        // should have already caught the unannotated case via an unresolved type var.
        Literal::Nope     => expected_ty.cloned().ok_or_else(|| YoloscriptError::type_error(
            ErrorCode::E0002,
            "cannot infer type of `nope`; add a type annotation",
            span,
        )),
    }
}

fn construct_binop(
    lhs: &Expr,
    op:  &BinOp,
    rhs: &Expr,
    span: &Span,
    ctx: &mut ConstructCtx,
) -> Result<TypedExpr, YoloscriptError> {
    let lhs = construct_expr(lhs, None, ctx)?;
    let rhs = construct_expr(rhs, None, ctx)?;
    let ty = match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => lhs.ty().clone(),
        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => Type::Bool,
        BinOp::And | BinOp::Or => Type::Bool,
        BinOp::Range | BinOp::RangeInclusive => Type::Named("Range".to_string(), vec![Type::Int]),
    };
    Ok(TypedExpr::BinOp(Box::new(lhs), op.clone(), Box::new(rhs), ty, span.clone()))
}

fn construct_unaryop(
    op:      &UnaryOp,
    operand: &Expr,
    span:    &Span,
    ctx:     &mut ConstructCtx,
) -> Result<TypedExpr, YoloscriptError> {
    let operand = construct_expr(operand, None, ctx)?;
    let ty = match op {
        UnaryOp::Neg => operand.ty().clone(),
        UnaryOp::Not => Type::Bool,
    };
    Ok(TypedExpr::UnaryOp(op.clone(), Box::new(operand), ty, span.clone()))
}
