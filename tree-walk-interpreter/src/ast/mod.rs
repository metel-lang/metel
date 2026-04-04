use crate::error::Span;

// ── Top-level ─────────────────────────────────────────────────────────────────

/// A complete source file is a list of declarations.
pub type Program = Vec<Decl>;

#[derive(Debug, Clone)]
pub enum Decl {
    Let(LetDecl),
    Mut(MutDecl),
    Fun(FunDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Impl(ImplBlock),
    Trait(TraitDecl),
    Stmt(Stmt),
}

// ── Declarations ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LetDecl {
    pub name: String,
    pub type_ann: Option<TypeExpr>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MutDecl {
    pub name: String,
    pub type_ann: Option<TypeExpr>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FunDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<FieldDef>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<VariantDef>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ImplBlock {
    /// `Some("TraitName")` for `impl Trait for Type`, `None` for `impl Type`
    pub trait_name: Option<String>,
    pub target_type: TypeExpr,
    pub methods: Vec<FunDecl>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TraitDecl {
    pub name: String,
    pub methods: Vec<TraitMethod>,
    pub span: Span,
}

// ── Supporting types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: String,
    pub bound: Option<TypeExpr>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub mutable: bool,  // true for `mut self`
    pub name: String,   // "self" for receiver params
    pub type_ann: TypeExpr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub type_ann: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct VariantDef {
    pub name: String,
    pub fields: Vec<FieldDef>,  // empty = unit variant
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub default_body: Option<Block>,
    pub span: Span,
}

// ── Statements ────────────────────────────────────────────────────────────────

/// A `{ decl* expr? }` block.  The optional `tail` is a bare expression
/// (no semicolon) whose value is the value of the whole block — used by
/// `if`-expressions, `loop`-expressions, closures, and similar.
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Decl>,
    pub tail:  Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    ForIn(ForInStmt),
    Loop(LoopStmt),
    Return(ReturnStmt),
    Break(BreakStmt),
    Continue(Span),
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Block,
    pub else_branch: Option<ElseBranch>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ElseBranch {
    Block(Block),
    If(Box<IfStmt>),
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub init: Option<Box<Decl>>,
    pub condition: Option<Expr>,
    pub step: Option<Expr>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ForInStmt {
    pub binding: String,
    pub iterable: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LoopStmt {
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct BreakStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

// ── Expressions ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal, Span),
    Ident(String, Span),
    Path(Vec<String>, Span),              // e.g. Direction::North
    Tuple(Vec<Expr>, Span),
    Array(Vec<Expr>, Span),
    BinOp(Box<Expr>, BinOp, Box<Expr>, Span),
    UnaryOp(UnaryOp, Box<Expr>, Span),
    Assign(AssignTarget, AssignOp, Box<Expr>, Span),
    Call { callee: Box<Expr>, args: Vec<Expr>, span: Span },
    MethodCall { receiver: Box<Expr>, method: String, args: Vec<Expr>, span: Span },
    FieldAccess { object: Box<Expr>, field: String, span: Span },
    TupleAccess { object: Box<Expr>, index: usize, span: Span },
    Index { object: Box<Expr>, index: Box<Expr>, span: Span },
    Cast { expr: Box<Expr>, target_type: TypeExpr, span: Span },
    Match { scrutinee: Box<Expr>, arms: Vec<MatchArm>, span: Span },
    If { condition: Box<Expr>, then_branch: Block, else_branch: Block, span: Span },
    Loop { body: Block, span: Span },
    Closure { params: Vec<Param>, return_type: Option<TypeExpr>, body: Block, span: Span },
    StructLiteral { path: Vec<String>, fields: Vec<(String, Expr)>, span: Span },
    PropagateError { expr: Box<Expr>, span: Span },  // the ? operator
}

impl Expr {
    pub fn span(&self) -> &Span {
        match self {
            Expr::Literal(_, s) | Expr::Ident(_, s) | Expr::Path(_, s)
            | Expr::Tuple(_, s) | Expr::Array(_, s) | Expr::BinOp(_, _, _, s)
            | Expr::UnaryOp(_, _, s) | Expr::Assign(_, _, _, s)
            | Expr::Call { span: s, .. } | Expr::MethodCall { span: s, .. }
            | Expr::FieldAccess { span: s, .. } | Expr::TupleAccess { span: s, .. }
            | Expr::Index { span: s, .. } | Expr::Cast { span: s, .. }
            | Expr::Match { span: s, .. } | Expr::If { span: s, .. }
            | Expr::Loop { span: s, .. } | Expr::Closure { span: s, .. }
            | Expr::StructLiteral { span: s, .. } | Expr::PropagateError { span: s, .. } => s,
        }
    }
}

// ── Literals ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Nope,
    Unit,
}

// ── Operators ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
    Range,        // ..
    RangeInclusive, // ..=
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Assign,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign,
}

// ── Assignment targets ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum AssignTarget {
    Ident(String, Span),
    FieldAccess { object: Box<Expr>, field: String, span: Span },
    Index { object: Box<Expr>, index: Box<Expr>, span: Span },
}

// ── Pattern matching ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard(Span),
    Binding(String, Span),
    Literal(Literal, Span),
    Nope(Span),
    EnumVariant { path: Vec<String>, fields: Vec<String>, span: Span },
    Tuple(Vec<Pattern>, Span),
}

// ── Types ─────────────────────────────────────────────────────────────────────

/// Type expressions as written in source code (before resolution).
#[derive(Debug, Clone)]
pub enum TypeExpr {
    Named(String, Vec<TypeExpr>),          // Int, String, Perhaps<Int>, ...
    Unit,                                  // ()
    Tuple(Vec<TypeExpr>),                  // (Int, String)
    Array(Box<TypeExpr>),                  // T[]
    Fun(Vec<TypeExpr>, Box<TypeExpr>),     // fun(Int) -> Bool
}
