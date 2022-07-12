use std::collections::HashMap;

use la_arena::Idx;
use smallvec::SmallVec;
use smol_str::SmolStr;

pub type ExprIdx = Idx<Expr>;

#[derive(Debug)]
pub enum Expr {
    Ident(SmolStr),
    Int(u64),
    Float(f64),
    String(SmolStr),
    Array(SmallVec<[ExprIdx; 2]>),
    Record(HashMap<SmolStr, ExprIdx>),
    Error,
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