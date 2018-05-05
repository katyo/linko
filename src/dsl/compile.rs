use std::rc::{Rc};
use std::collections::{HashMap};

use futures::{Stream};
use futures::stream::{empty, once};
use future_pubsub::unsync::{into_cloneable};

use dsl::def::{Mesh, Node, Link, Value};
use dsl::imp::{ValueCell, NodeDecl, NodeDecls, Observable, Observables};

#[derive(Debug, Clone, PartialEq)]
pub struct InputControl {
    pub name: String,
    pub value: Value,
}

pub type ControlStream = Box<Stream<Item = InputControl, Error = ()>>;

#[derive(Debug, Clone, PartialEq)]
pub struct OutputChange {
    pub link: Rc<Link>,
    pub value: Value,
}

impl OutputChange {
    pub fn new<V: Into<Value>>(link: Link, value: V) -> Self {
        Self { link: Rc::new(link), value: value.into() }
    }

    pub fn wrap<V: Into<Value>>(link: Rc<Link>, value: V) -> Self {
        Self { link: link.clone(), value: value.into() }
    }
}

pub type ChangesStream = Box<Stream<Item = OutputChange, Error = ()>>;

pub type ValuesMap = HashMap<Rc<Link>, ValueCell>;

pub fn compile(decls: &NodeDecls, mesh: &Mesh, ctrl_stream: ControlStream) -> Result<(ValuesMap, ChangesStream), String> {
    let ctrl_stream = into_cloneable(ctrl_stream);
    
    // validate nodes
    for ref node in &mesh.nodes {
        if node.name == "" { return Err("Empty node name".into()); }
        if node.kind == "" { return Err("Empty node kind".into()); }
        
        let decl = if let Some(decl) = decls.get(&node.kind) { decl }
        else { return Err(format!("Unsupported node kind `{}`", node.kind)); };

        validate_inputs(mesh, decl, node)?;
        validate_outputs(decl, node)?;
    }

    let mut observables: HashMap<Link, Observable> = HashMap::new();

    for ref ctrl in &mesh.ctrls {
        let name = ctrl.name.clone();
        let stream = once(Ok(ctrl.value))
            .chain(ctrl_stream.clone()
                   .filter(move |item| item.name == name)
                   .map(|item| item.value))
            .map(|v| { trace!("ctrl in {}", v); v })
            .map_err(|_| ());
        
        observables.insert(Link::Ctrl { name: ctrl.name.clone() },
                           Observable::from(stream));
    }

    let mut nodes: Vec<_> = mesh.nodes.iter().collect();

    loop {
        let new_nodes: Vec<_> = nodes.iter().cloned().filter(|ref node| {
            for ref input in &node.ins {
                if !observables.contains_key(&input.link) {
                    return true;
                }
            }
            debug!("instantiate node `{}`", node.name);
            let mut ins = Observables::new();
            for ref input in &node.ins {
                ins = ins.put(&input.name, observables.get(&input.link).unwrap().clone());
            }
            let decl = decls.get(&node.kind).unwrap();
            let mut outs = decl.imp(ins);
            for ref output in &node.outs {
                observables.insert(Link::Output { node: node.name.clone(), out: output.name.clone() },
                                   outs.get(&output.name));
            }
            false
        }).collect();
        
        if new_nodes.is_empty() { break; }
        
        if new_nodes.len() == nodes.len() {
            return Err(format!("Unable to instantiate nodes {} due to cyclic dependencies",
                               new_nodes.iter().fold(String::new(), |out, node| if out == "" {
                                   format!("`{}`", node.name)
                               } else {
                                   format!("{}, `{}`", out, node.name)
                               })));
        }

        nodes = new_nodes;
    }

    let mut values_map = ValuesMap::new();

    let mut change_stream: ChangesStream = Box::new(empty());
    
    for (link, observable) in observables.iter() {
        let (value, stream) = observable.clone().into();
        let link = Rc::new(link.clone());
        values_map.insert(link.clone(), value);
        change_stream = Box::new(
            change_stream.select(stream.map(move |value| OutputChange::wrap(link.clone(), value)))
        );
    }

    Ok((values_map, change_stream))
}

fn validate_outputs(decl: &NodeDecl, node: &Node) -> Result<(), String> {
    // check existing outputs
    for ref output in &node.outs {
        if output.name == "" { return Err(format!("Empty output name in node `{}`", node.name)); }
        if decl.def.get_out(&output.name).is_none() {
            return Err(format!("Extra output `{}` in node `{}`", output.name, node.name));
        }
    }

    Ok(())
}

fn validate_inputs(mesh: &Mesh, decl: &NodeDecl, node: &Node) -> Result<(), String> {
    // check missing inputs
    for ref input_kind in &decl.def.ins {
        if node.get_in(&input_kind.name).is_none() {
            return Err(format!("Missing input `{}` in node `{}`", input_kind.name, node.name));
        }
    }
    
    // check existing inputs
    for ref input in &node.ins {
        if input.name == "" { return Err(format!("Empty input name in node `{}`", node.name)); }
        if decl.def.get_in(&input.name).is_none() {
            return Err(format!("Extra input `{}` in node `{}`", input.name, node.name));
        }
        
        match &input.link {
            &Link::Output { node: ref link_node_name, out: ref link_out } => {
                if let Some(ref link_node) = mesh.get_node(link_node_name) {
                    if link_node.get_out(link_out).is_none() {
                        return Err(format!("Input `{}` of node `{}` linked to missing output `{}` of node `{}`", input.name, node.name, link_out, link_node_name));
                    }
                } else {
                    return Err(format!("Input `{}` of node `{}` linked to missing node `{}`", input.name, node.name, link_node_name));
                }
            },
            &Link::Ctrl { name: ref link_ctrl } => {
                if mesh.get_ctrl(link_ctrl).is_none() {
                    return Err(format!("Input `{}` of node `{}` linked to missing control `{}`", input.name, node.name, link_ctrl));
                }
            },
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use dsl::def::{Mesh, Link};
    use dsl::imp::{NodeDecls};
    use dsl::{OutputChange, compile};
    use ops::{basic_ops};
    use serde_json::{from_str};
    use futures::{Stream, Future};
    use futures::stream::{empty};
    use futures::future::{lazy};
    use tokio::executor::current_thread::{block_on_all, spawn};

    #[test]
    fn test_compile_ok() {
        let ops = NodeDecls::new().with(basic_ops);

        let mesh: Mesh = from_str(r#"{
  "nodes": [
    { "name": "mul", "kind": "*", "ins": [
      { "name": "a", "link": { "name": "a" } },
      { "name": "b", "link": { "name": "b" } }
    ], "outs": [
      { "name": "=" }
    ] },
    { "name": "add", "kind": "+", "ins": [
      { "name": "a", "link": { "node": "mul", "out": "=" } },
      { "name": "b", "link": { "name": "c" } }
    ], "outs": [
      { "name": "=" }
    ] }
  ],
  "ctrls": [
    { "name": "a", "value": "2" },
    { "name": "b", "value": "3" },
    { "name": "c", "value": "1" }
  ]
}"#).unwrap();

        let res = compile(&ops, &mesh, Box::new(empty()));

        if let Err(ref err) = res { println!("Err: {}", err); }
        
        assert!(res.is_ok());

        let (_, out) = res.unwrap();

        block_on_all(lazy(|| {
            spawn(out.collect().map(|vals| {
                println!("{:?}", vals);
                assert_eq!(vals.iter()
                           .filter(|out| *out.link == Link::output("mul", "="))
                           .map(|out| out.value)
                           .last(), Some(6.into()));
                assert_eq!(vals.iter()
                           .filter(|out| *out.link == Link::output("add", "="))
                           .map(|out| out.value)
                           .last(), Some(7.into()));
            }));

            Ok::<_, ()>(())
        })).unwrap();
    }

    #[test]
    fn test_compile_err_cyclic_deps() {
        let ops = NodeDecls::new().with(basic_ops);

        let mesh: Mesh = from_str(r#"{
  "nodes": [
    { "name": "mul", "kind": "*", "ins": [
      { "name": "a", "link": { "node": "add", "out": "=" } },
      { "name": "b", "link": { "name": "a" } }
    ], "outs": [
      { "name": "=" }
    ] },
    { "name": "add", "kind": "+", "ins": [
      { "name": "a", "link": { "node": "mul", "out": "=" } },
      { "name": "b", "link": { "name": "b" } }
    ], "outs": [
      { "name": "=" }
    ] }
  ],
  "ctrls": [
    { "name": "a", "value": "1" },
    { "name": "b", "value": "2" }
  ]
}"#).unwrap();

        let res = compile(&ops, &mesh, Box::new(empty()));

        assert!(res.is_err());

        if let Err(ref err) = res {
            assert_eq!(err.as_str(), "Unable to instantiate nodes `mul`, `add` due to cyclic dependencies");
        }
    }
}
