pub struct ToolSet {
    list: &'static [&'static str],
}

#[macro_export]
macro_rules! tool_set {
    ($T:path, $($E:path, )*) => {
        crate::tools::ToolSet::new(&[<$T as crate::tools::Tool>::NAME, tool_set!($($E,)*)])
    };
    () => {}
}

impl ToolSet {
    pub const fn new(list: &'static [&'static str]) -> Self {
        Self { list }
    }

    pub fn names(&self) -> impl Iterator<Item = &'static str> {
        self.list.iter().map(|x| *x)
    }
}
