#[derive(Debug, Clone, Copy)]
pub struct ToolSet {
    list: &'static [&'static str],
}

#[macro_export]
macro_rules! tool_set {
    () => {
        crate::tools::ToolSet::new(&[])
    };
    ($($E:path),*) => {
        crate::tools::ToolSet::new(&[tool_set!(@ $($E),*)])
    };

    (@ $P:path, $($E:path),+) => {tool_set!(@ $P), tool_set!(@ $($E,)*)};
    (@ $P:path) => {<$P as crate::tools::Tool>::NAME};
    (@)=>{}
}

impl ToolSet {
    pub const fn new(list: &'static [&'static str]) -> Self {
        Self { list }
    }

    pub fn toold(&self) -> impl Iterator<Item = &'static str> {
        self.list.iter().map(|x| *x)
    }
}
