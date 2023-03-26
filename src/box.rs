#[macro_export]
macro_rules! Box {
    ($type:ty) => {{
        let mut _box = Box::new(unsafe { mem::zeroed::<$type>() });
        let _box_ptr = addr_of_mut!(*_box);
        (_box, _box_ptr)
    }};
}