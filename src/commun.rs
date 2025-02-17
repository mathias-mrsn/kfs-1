#[const_trait]
pub trait ConstFrom<T>: Sized
{
    fn from_const(value: T) -> Self;
}

#[const_trait]
pub trait ConstInto<T>: Sized
{
    fn into_const(self) -> T;
}

#[const_trait]
pub trait ConstDefault: Sized
{
    fn default_const() -> Self;
}
