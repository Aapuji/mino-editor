#[derive(Debug)]
pub struct Language<const N: usize> {
    filetype: &'static str,
    exts: [&'static str; N],

}

const C: Language<3> = Language {
    filetype: "c",
    exts: [".c", ".h", ".cpp" ]
};