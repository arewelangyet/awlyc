use std::collections::HashMap;

use la_arena::Idx;
use smallvec::SmallVec;
use smol_str::SmolStr;

#[derive(Debug)]
pub struct Module {
    pub imports: Vec<ImportDecl>,
    pub functions: Vec<FnDecl>,
    pub expr: ExprIdx,
}

pub type ExprIdx = Idx<Expr>;

#[derive(Debug)]
pub enum Expr {
    Path(Vec<SmolStr>),
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
    pub args: Vec<ExprIdx>,
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

#[derive(Debug)]
pub struct ImportDecl {
    pub name: SmolStr,
    pub path: SmolStr,
}

#[derive(Debug)]
pub struct FnDecl {
    pub name: SmolStr,
    pub params: FnParams,
    pub body: ExprIdx,
}

#[derive(Debug)]
pub struct FnParams(pub Vec<FnParam>);

#[derive(Debug)]
pub struct FnParam(pub SmolStr);
