use std::ops::{Deref, DerefMut};

pub trait Apply {
    fn apply<F, R>(self, f: F) -> R 
    where
        Self: Sized,
        F: FnOnce(Self) -> R,
    {
        f(self)
    }

    fn apply_ref<'a, F, R>(&'a self, f: F) -> R 
    where
        F: FnOnce(&'a Self) -> R,
    {
        f(self)
    }

    fn apply_mut<'a, F, R>(&'a mut self, f: F) -> R 
    where
        Self: Deref,
        F: FnOnce(&'a mut Self) -> R,
    {
        f(self)
    }
    
    fn apply_deref<'a, F, R>(&'a self, f: F) -> R 
    where
        Self: Deref,
        F: FnOnce(&'a Self::Target) -> R,
    {
        f(&*self)
    }

    fn apply_deref_mut<'a, F, R>(&'a mut self, f: F) -> R 
    where
        Self: DerefMut,
        F: FnOnce(&'a mut Self::Target) -> R,
    {
        f(&mut*self)
    }
}

impl<T: ?Sized> Apply for T {}

pub trait Also: Sized {
    fn also<F>(self, f: F) -> Self
    where
        F: FnOnce(&Self),
    {
        f(&self);
        self
    }

    fn also_mut<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Self),
    {
        f(&mut self);
        self
    }
    
    fn also_deref<F>(self, f: F) -> Self
    where
        Self: Deref,
        F: FnOnce(&Self::Target),
    {
        f(&*self);
        self
    }

    fn also_deref_mut<F>(mut self, f: F) -> Self
    where
        Self: DerefMut,
        F: FnOnce(&mut Self::Target),
    {
        f(&mut*self);
        self
    }
}

impl<T> Also for T {}
