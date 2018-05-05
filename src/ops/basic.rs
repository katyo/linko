use decimal::{d128};
use futures::{Stream};

use dsl::def::{NodeKind, InputKind, OutputKind};
use dsl::imp::{Observables, NodeDecl, NodeDecls};

fn neg_impl(mut ins: Observables) -> Observables {
    let (_, ai) = ins.get("a").into();
    
    let ro = Box::new(ai.map(|a| -a));
    
    Observables::new().put("=", ro)
}

fn neg_decl(decls: &mut NodeDecls) {
    decls.put(NodeDecl::new(
        NodeKind::new("-")
            .with_in(InputKind::new("a"))
            .with_out(OutputKind::new("=")),
        neg_impl));
}

fn add_impl(mut ins: Observables) -> Observables {
    let (av, ai) = ins.get("a").into();
    let (bv, bi) = ins.get("b").into();
    
    let ro = Box::new(ai.map(|_| ()).select(bi.map(|_| ())).map(move |_| {
        trace!("{:?} + {:?}", *av.borrow(), *bv.borrow());
        match (*av.borrow(), *bv.borrow()) {
            (Some(a), Some(b)) => Some(a + b),
            _ => None,
        }
    }).skip_while(|opt| Ok(opt.is_none())).map(Option::unwrap));
    
    Observables::new().put("=", ro)
}

fn add_decl(decls: &mut NodeDecls) {
    decls.put(NodeDecl::new(
        NodeKind::new("+")
            .with_in(InputKind::new("a"))
            .with_in(InputKind::new("b"))
            .with_out(OutputKind::new("=")),
        add_impl));
}

fn inv_impl(mut ins: Observables) -> Observables {
    let (_, ai) = ins.get("a").into();
    
    let ro = Box::new(ai.map(|a| d128::from(1)/a));
    
    Observables::new().put("=", ro)
}

fn inv_decl(decls: &mut NodeDecls) {
    decls.put(NodeDecl::new(
        NodeKind::new("^-1")
            .with_in(InputKind::new("a"))
            .with_out(OutputKind::new("=")),
        inv_impl));
}

fn mul_impl(mut ins: Observables) -> Observables {
    let (av, ai) = ins.get("a").into();
    let (bv, bi) = ins.get("b").into();
    
    let ro = Box::new(ai.map(|_| ()).select(bi.map(|_| ())).map(move |_| {
        trace!("{:?} * {:?}", *av.borrow(), *bv.borrow());
        match (*av.borrow(), *bv.borrow()) {
            (Some(a), Some(b)) => Some(a * b),
            _ => None,
        }
    }).skip_while(|opt| Ok(opt.is_none())).map(Option::unwrap));
    
    Observables::new().put("=", ro)
}

fn mul_decl(decls: &mut NodeDecls) {
    decls.put(NodeDecl::new(
        NodeKind::new("*")
            .with_in(InputKind::new("a"))
            .with_in(InputKind::new("b"))
            .with_out(OutputKind::new("=")),
        mul_impl));
}

pub fn basic_ops(decls: &mut NodeDecls) {
    neg_decl(decls);
    add_decl(decls);
    inv_decl(decls);
    mul_decl(decls);
}
