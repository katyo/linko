use std::rc::{Rc};
use std::cell::{RefCell};
use std::collections::{HashMap};

use dsl::{NodeKind, InputKind, OutputKind, Value};
use futures::{Stream};
use future_pubsub::unsync::{Cloneable, into_cloneable};

pub type ValueCell = Rc<RefCell<Option<Value>>>;
pub type ValueStream = Box<Stream<Item = Value, Error = ()>>;

#[derive(Clone)]
pub struct Observable {
    value: ValueCell,
    stream: Cloneable<ValueStream>,
}

impl Into<(ValueCell, ValueStream)> for Observable {
    fn into(self) -> (ValueCell, ValueStream) {
        (self.value, Box::new(self.stream.map(|rc| *rc).map_err(|_| ())))
    }
}

impl<S> From<S> for Observable
where S: Stream<Item = Value, Error = ()> + 'static
{
    fn from(stream: S) -> Self {
        let value = Rc::new(RefCell::new(Option::None));
        let value2 = value.clone();
        let stream = Box::new(stream.map(move |val| {
            *value2.borrow_mut() = Some(val);
            val
        }));
        Self { value, stream: into_cloneable(stream) }
    }
}

pub struct Observables {
    map: HashMap<String, Observable>,
}

impl Observables {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    /// use stream of values
    pub fn put<K: AsRef<str>, O: Into<Observable>>(mut self, name: K, observable: O) -> Self {
        self.map.insert(name.as_ref().into(), observable.into());
        self
    }

    /// check parameter exising
    pub fn has<K: AsRef<str>>(&self, name: K) -> bool {
        self.map.contains_key(name.as_ref().into())
    }

    /// get parameter
    pub fn get<K: AsRef<str>>(&mut self, name: K) -> Observable {
        self.map.remove(name.as_ref().into()).unwrap()
    }
}

/// Instantiatiate node
pub type NodeInst = fn(Observables) -> Observables;

pub struct NodeDecl {
    pub def: NodeKind,
    pub imp: NodeInst,
}

impl NodeDecl {
    pub fn new(def: NodeKind, imp: NodeInst) -> Self {
        Self { def, imp }
    }

    pub fn imp(&self, ins: Observables) -> Observables {
        (self.imp)(ins)
    }
}

/// Nodes declarations registry
pub struct NodeDecls {
    decls: HashMap<String, NodeDecl>,
}

impl NodeDecls {
    pub fn new() -> Self {
        Self { decls: HashMap::new() }
    }

    pub fn with(mut self, add: fn(&mut NodeDecls)) -> Self {
        add(&mut self);
        self
    }

    pub fn add(mut self, decl: NodeDecl) -> Self {
        self.decls.insert(decl.def.name.clone(), decl);
        self
    }

    pub fn put(&mut self, decl: NodeDecl) {
        self.decls.insert(decl.def.name.clone(), decl);
    }

    pub fn has<S: AsRef<str>>(&self, name: S) -> bool {
        self.decls.contains_key(name.as_ref().into())
    }

    pub fn get<S: AsRef<str>>(&self, name: S) -> Option<&NodeDecl> {
        self.decls.get(name.as_ref().into())
    }
}

#[cfg(test)]
mod test {
    use super::{Observables, NodeDecl, NodeKind, InputKind, OutputKind, NodeDecls};
    use futures::{Future, Sink, Stream};
    use futures::future::{lazy};
    use futures::unsync::mpsc::{unbounded};
    use tokio::executor::current_thread::{block_on_all, spawn};

    #[test]
    fn test_adder_decl() {        
        fn adder_impl(mut ins: Observables) -> Observables {
            let (av, ai) = ins.get("a").into();
            let (bv, bi) = ins.get("b").into();
            
            let ro = Box::new(ai.map(|_| ()).select(bi.map(|_| ())).map(move |_| {
                match (*av.borrow(), *bv.borrow()) {
                    (Some(a), Some(b)) => Some(a + b),
                    _ => None,
                }
            }).skip_while(|opt| Ok(opt.is_none())).map(Option::unwrap));

            Observables::new().put("=", ro)
        }

        let decls = NodeDecls::new()
            .add(NodeDecl::new(
                NodeKind::new("+")
                    .with_in(InputKind::new("a"))
                    .with_in(InputKind::new("b"))
                    .with_out(OutputKind::new("=")),
                adder_impl));

        let (sa, a) = unbounded();
        let (sb, b) = unbounded();

        assert_eq!(decls.has("+"), true);

        let adder_decl = decls.get("+").unwrap();

        let (v, r) = adder_decl.imp(Observables::new().put("a", a).put("b", b)).get("=").into();

        assert_eq!(*v.borrow(), None);

        block_on_all(lazy(|| {
            println!("send b: 1");
            spawn(sb.send(1.into()).map(|_sb| { println!("sent b: 1"); () }).map_err(|_| ()));

            let v2 = v.clone();
            spawn(r.map(move |r| {
                println!("recv =: {}", v2.borrow().unwrap());
                r
            }).collect().map(move |rv| {
                assert_eq!(rv, vec![1.into(), 2.into()]);
                assert_eq!(*v.borrow(), Some(2.into()));
            }));

            println!("send a: 0");
            spawn(sa.send(0.into()).and_then(|sa| { println!("sent a: 0"); println!("send a: 1"); sa.send(1.into()) }).map(|_sa| { println!("sent a: 1"); () }).map_err(|_| ()));

            Ok::<_, ()>(())
        })).unwrap();
    }
}
