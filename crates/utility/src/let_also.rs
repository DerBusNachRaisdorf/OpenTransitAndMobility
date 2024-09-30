pub trait LetAlso {
    fn let_ref<R, F>(&self, block: F) -> R
    where
        F: FnOnce(&Self) -> R;
    fn let_mut<R, F>(&mut self, block: F) -> R
    where
        F: FnMut(&mut Self) -> R;
    fn let_owned<R, F>(self, block: F) -> R
    where
        Self: Sized,
        F: FnOnce(Self) -> R;

    fn also_ref<F>(&self, block: F) -> &Self
    where
        F: FnOnce(&Self);
    fn also_mut<F>(&mut self, block: F) -> &mut Self
    where
        F: FnMut(&mut Self);
}

impl<T> LetAlso for T {
    fn let_ref<R, F>(&self, block: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        block(self)
    }
    fn let_mut<R, F>(&mut self, mut block: F) -> R
    where
        F: FnMut(&mut Self) -> R,
    {
        block(self)
    }
    fn let_owned<R, F>(self, block: F) -> R
    where
        Self: Sized,
        F: FnOnce(Self) -> R,
    {
        block(self)
    }

    fn also_ref<F>(&self, block: F) -> &Self
    where
        F: FnOnce(&Self),
    {
        block(self);
        self
    }
    fn also_mut<F>(&mut self, mut block: F) -> &mut Self
    where
        F: FnMut(&mut Self),
    {
        block(self);
        self
    }
}
