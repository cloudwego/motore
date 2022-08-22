use crate::{layer::Identity, utils::Either};

/// Convert an `Option<Layer>` into a [`Layer`].
///
/// ```
/// # use std::time::Duration;
/// # use motore::Service;
/// # use motore::builder::ServiceBuilder;
/// use motore::utils::option_layer;
/// # use motore::timeout::TimeoutLayer;
/// # async fn wrap<S>(svc: S) where S: Service<(), (), Error = &'static str> + 'static + Send, {
/// # let timeout = Some(Duration::new(10, 0));
/// // Layer to apply a timeout if configured
/// let maybe_timeout = option_layer(timeout.map(|duration| TimeoutLayer::new(Some(duration))));
///
/// ServiceBuilder::new()
///     .layer(maybe_timeout)
///     .service(svc);
/// # }
/// ```
///
/// [`Layer`]: crate::layer::Layer
pub fn option_layer<L>(layer: Option<L>) -> Either<L, Identity> {
    if let Some(layer) = layer {
        Either::A(layer)
    } else {
        Either::B(Identity::new())
    }
}
