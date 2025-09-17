#[derive(Debug, Clone, Copy)]
pub struct ToolSet {
    list: &'static [&'static str],
}

#[macro_export]
macro_rules! tool_set {
    ($($E:path),*) => {
        crate::tools::ToolSet::new(tool_set!(@ [] $($E),*))
    };

    (@ [$($T:tt)*] $P:path, $($E:path),+) => {tool_set!(@ [$($T)* <$P as crate::tools::Tool>::NAME,] $($E),+)};
    (@ [$($T:tt)*] $P:path) => {tool_set!(@ [$($T)* <$P as crate::tools::Tool>::NAME,])};
    (@ [$($T:tt)*]) => {&[$($T)*]};
}

impl ToolSet {
    pub const fn new(list: &'static [&'static str]) -> Self {
        Self { list }
    }

    pub fn toold(&self) -> impl Iterator<Item = &'static str> {
        self.list.iter().map(|x| *x)
    }
}
