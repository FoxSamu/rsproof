pub type TriResult<T, E> = Result<T, Option<E>>;

pub trait TriRes<T, E> {
    fn ok(t: T) -> Self;
    fn none() -> Self;
    fn err(e: E) -> Self;

    fn with_error(self, e: E) -> Self;
    fn to_result(self, e: E) -> Result<T, E>;
}

impl<T, E> TriRes<T, E> for TriResult<T, E> {
    fn with_error(self, e: E) -> Self {
        if let Err(None) = self {
            return Err(Some(e));
        }

        return self;
    }

    fn to_result(self, e: E) -> Result<T, E> {
        match self {
            Err(None) => Err(e),
            Err(Some(e)) => Err(e),
            Ok(v) => Ok(v)
        }
    }
    
    fn ok(t: T) -> Self {
        Ok(t)
    }
    
    fn none() -> Self {
        Err(None)
    }
    
    fn err(e: E) -> Self {
        Err(Some(e))
    }
}