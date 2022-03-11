//! Adapts a file from a rustc testcase into something testable using gccrs
//! and ftf

pub struct FileAdaptor<I>(I)
where
    I: Iterator;

impl<I> Iterator for FileAdaptor<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub trait FileAdaptorExt: Iterator {
    fn adapt(self) -> FileAdaptor<Self>
    where
        Self: Sized,
    {
        // FIXME: Add transformation from rustc test cases
        FileAdaptor(self)
    }
}

impl<I: Iterator> FileAdaptorExt for I {}
