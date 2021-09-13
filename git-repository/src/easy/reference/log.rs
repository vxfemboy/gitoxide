//!
use std::{borrow::Borrow, cell::RefMut, marker::PhantomData, ops::DerefMut};

use git_ref::file::ReferenceExt;

use crate::{easy, easy::Reference};

/// A platform to obtain iterators over reference logs.
#[must_use = "Iterators should be obtained from this log platform"]
pub struct Platform<'repo, A: 'repo, R>
where
    R: Borrow<Reference<'repo, A>>,
{
    pub(crate) reference: R,
    pub(crate) buf: RefMut<'repo, Vec<u8>>,
    pub(crate) _phantom: PhantomData<A>,
}

///
pub mod init {
    use crate::easy;

    /// The error returned by [State::iter()][super::State::iter()] and [State::iter_rev()][super::State::iter_rev()].
    #[derive(Debug, thiserror::Error)]
    #[allow(missing_docs)]
    pub enum Error {
        #[error(transparent)]
        Io(#[from] std::io::Error),
        #[error(transparent)]
        BorrowRepo(#[from] easy::borrow::repo::Error),
    }
}

// TODO: use attached types with `access`
/// An iterator over reference logs, most recent to oldest.
pub type ReverseIter<'a> = git_ref::file::log::iter::Reverse<'a, std::fs::File>;
/// An iterator over reference logs, oldest to newest.
pub type ForwardIter<'a> = git_ref::file::log::iter::Forward<'a>;

impl<'repo, A, R> Platform<'repo, A, R>
where
    A: easy::Access + Sized,
    R: Borrow<Reference<'repo, A>>,
{
    /// Return a reverse iterator over reference logs, most recent to oldest.
    ///
    /// This is a typical and efficient way of accessing logs as one is interested in the most recent ones first.
    pub fn iter_rev(&mut self) -> Result<Option<ReverseIter<'_>>, init::Error> {
        let buf = self.buf.deref_mut();
        buf.resize(512, 0);
        Ok(self
            .reference
            .borrow()
            .inner
            .log_iter_rev(&self.reference.borrow().access.repo()?.refs, buf)?)
    }

    // TODO: tests
    /// Return an iterator over reference logs, from oldest to newest.
    ///
    /// The iterator is optimized for rewriting the processing or rewriting the entire log.
    /// For accessing only the most recent entries, see [`iter_rev()`][State::iter_rev()].
    pub fn iter(&mut self) -> Result<Option<ForwardIter<'_>>, init::Error> {
        let buf = self.buf.deref_mut();
        Ok(self
            .reference
            .borrow()
            .inner
            .log_iter(&self.reference.borrow().access.repo()?.refs, buf)?)
    }
}

impl<'repo, A> Reference<'repo, A>
where
    A: easy::Access + Sized,
{
    /// Return a platform for obtaining iterators over reference logs.
    pub fn log(&self) -> Result<Platform<'repo, A, &'_ Reference<'repo, A>>, easy::borrow::state::Error> {
        Ok(Platform {
            reference: self,
            buf: self.access.state().try_borrow_mut_buf()?,
            _phantom: Default::default(),
        })
    }
}
