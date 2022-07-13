use std::{collections::HashMap, ops::Deref};

use awlyc_error::Span;
use la_arena::Idx;
use smallvec::SmallVec;
use smol_str::SmolStr;

pub type ExprIdx = Idx<Spanned<Expr>>;

#[derive(Debug)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug)]
pub enum Expr {
    Path(Vec<Spanned<SmolStr>>),
    Int(u64),
    Float(f64),
    String(SmolStr),
    Array(SmallVec<[ExprIdx; 2]>),
    Record(HashMap<SmolStr, ExprIdx>),
    Binop(Binop),
    Call(Call),
    Error,
}

#[derive(Debug)]
pub struct Call {
    pub callee: ExprIdx,
    pub args: Spanned<Vec<ExprIdx>>,
}

#[derive(Debug)]
pub enum BinopKind {
    Add,
}

#[derive(Debug)]
pub struct Binop {
    pub lhs: ExprIdx,
    pub op: BinopKind,
    pub rhs: ExprIdx,
}

#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub name: SmolStr,
    pub path: SmolStr,
}

#[derive(Debug)]
pub struct FnDecl {
    pub name: Spanned<SmolStr>,
    pub params: Spanned<FnParams>,
    pub body: ExprIdx,
}

#[derive(Debug)]
pub struct FnParams(pub Vec<FnParam>);

#[derive(Debug)]
pub struct FnParam(pub SmolStr);
