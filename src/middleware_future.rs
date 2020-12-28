use actix_service::Service;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct GovernorMiddlewareFuture<S: Service> {
    kind: FutureKind<S>,
}

impl<S: Service> GovernorMiddlewareFuture<S> {
    pub(crate) fn from_future(fut: S::Future) -> Self {
        Self {
            kind: FutureKind::ForwardToService(fut)
        }
    }
    pub(crate) fn fail_immediately(err: S::Error) -> Self {
        Self {
            kind: FutureKind::FailImmediately(Some(err))
        }
    }
}

impl<S: Service> Unpin for GovernorMiddlewareFuture<S>
where
    S::Future: Unpin,
{}

enum FutureKind<S: Service> {
    ForwardToService(S::Future),
    FailImmediately(Option<S::Error>),
}

impl<S: Service, B> Future for GovernorMiddlewareFuture<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Output = Result<ServiceResponse<B>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = unsafe { Pin::into_inner_unchecked(self) };

        match &mut me.kind {
            FutureKind::ForwardToService(future) => {
                let future = unsafe { Pin::new_unchecked(future) };
                future.poll(cx)
            },
            FutureKind::FailImmediately(opt) => {
                let error = opt.take().expect("Poll called after completion.");
                Poll::Ready(Err(error))
            },
        }
    }
}
