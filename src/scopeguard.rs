// Extracted from the scopeguard crate
use core::ops::{Deref, DerefMut};

// 添加注释: 用于构建范围守卫, 结构体需要传入value和dropfn, 用于在离开范围时进行调用的
pub struct ScopeGuard<T, F>
where
    F: FnMut(&mut T),
{
    dropfn: F,
    value: T,
}

#[cfg_attr(feature = "inline-more", inline)]
pub fn guard<T, F>(value: T, dropfn: F) -> ScopeGuard<T, F>
where
    F: FnMut(&mut T),
{
    ScopeGuard { dropfn, value }
}

impl<T, F> Deref for ScopeGuard<T, F>
where
    F: FnMut(&mut T),
{
    type Target = T;
    #[cfg_attr(feature = "inline-more", inline)]
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T, F> DerefMut for ScopeGuard<T, F>
where
    F: FnMut(&mut T),
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T, F> Drop for ScopeGuard<T, F>
where
    F: FnMut(&mut T),
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn drop(&mut self) {
        (self.dropfn)(&mut self.value)
    }
}
