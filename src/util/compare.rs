pub trait CompareSmallest<T: PartialOrd> {
    fn set_if_smallest(&mut self, value: T);
}

impl<T: PartialOrd> CompareSmallest<T> for Option<T> {
    fn set_if_smallest(&mut self, value: T) {
        let set = match self.as_ref() {
            Some(v) => value.lt(v),
            None => true
        };

        if set {
            *self = Some(value);
        }
    }
}
