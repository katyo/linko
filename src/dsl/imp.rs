use dsl::def::{NodeKind, InputKind, OutputKind};
use decimate::{Decimal};
use futures::{Stream, Sink};
//use future_pubsub::unsync::unbounded::{UnboundedReceiver, UnboundedSender};

pub type Value = Decimal<i32>;
//pub type SinkValue = Sink<SinkItem = Value, SinkError = ()>;
pub type ValueStream = Box<Stream<Item = Value, Error = ()>>;
pub type ValueStreams = Vec<ValueStream>;

/// Instantiatiate node
pub type NodeInst = fn(ValueStreams) -> ValueStreams;

pub struct NodeDecl {
    pub def: NodeKind,
    pub imp: NodeInst,
}

impl NodeDecl {
    pub fn new(def: NodeKind, imp: NodeInst) -> Self {
        Self { def, imp }
    }
}

#[cfg(test)]
mod test {
    use super::{ValueStreams, NodeDecl, NodeKind, InputKind, OutputKind};
    use futures::{Future, Sink, Stream};
    use futures::future::{Either, lazy};
    use futures::unsync::mpsc::{unbounded};
    use tokio::executor::current_thread::{block_on_all, spawn};

    #[test]
    fn test_adder_decl() {
        fn adder_impl(mut ins: ValueStreams) -> ValueStreams {
            let bi = ins.pop().unwrap();
            let ai = ins.pop().unwrap();
            
            let mut a = None;
            let mut b = None;
            
            let ro = Box::new(ai.map(Either::A).select(bi.map(Either::B)).map(move |v| {
                match v {
                    Either::A(na) => a = Some(na),
                    Either::B(nb) => b = Some(nb),
                }
                match (a, b) {
                    (Some(a), Some(b)) => Some(a + b),
                    _ => None,
                }
            }).skip_while(|opt| Ok(opt.is_none())).map(Option::unwrap));
            
            vec![ro]
        }
        
        let adder_decl = NodeDecl::new(
            NodeKind::new("+")
                .with_in(InputKind::new("a"))
                .with_in(InputKind::new("b"))
                .with_out(OutputKind::new("=")),
            adder_impl,
        );

        let (sa, a) = unbounded();
        let (sb, b) = unbounded();

        let r = adder_impl(vec![Box::new(a), Box::new(b)]).pop().unwrap();

        block_on_all(lazy(|| {
            spawn(sb.send(1.into()).map(|_sb| ()).map_err(|_| ()));
            
            spawn(r.collect().map(|rv| {
                assert_eq!(rv, vec![1.into(), 2.into()]);
            }));

            spawn(sa.send(0.into()).and_then(|sa| sa.send(1.into())).map(|_sa| ()).map_err(|_| ()));

            Ok::<_, ()>(())
        })).unwrap();
    }
}
