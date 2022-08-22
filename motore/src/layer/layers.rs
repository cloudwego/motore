use super::{Identity, Layer, Stack};
use crate::utils::Either;

#[derive(Clone, Debug)]
pub struct Layers<L>(pub L);

impl Default for Layers<Identity> {
    fn default() -> Self {
        Layers::new(Identity::new())
    }
}

impl<L> Layers<L> {
    pub fn new(layer: L) -> Self {
        Layers(layer)
    }

    pub fn push<O>(self, outer: O) -> Layers<Stack<L, O>> {
        Layers(Stack::new(self.0, outer))
    }

    pub fn push_optional<O>(self, outer: Option<O>) -> Layers<Stack<L, Either<O, Identity>>> {
        self.push(if let Some(o) = outer {
            Either::A(o)
        } else {
            Either::B(Identity::new())
        })
    }
}

impl<M, L: Layer<M>> Layer<M> for Layers<L> {
    type Service = L::Service;

    fn layer(self, inner: M) -> Self::Service {
        self.0.layer(inner)
    }
}
