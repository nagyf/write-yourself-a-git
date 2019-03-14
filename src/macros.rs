macro_rules! path {
    ($($segment:expr),+) => {{
        let mut base = ::std::path::PathBuf::new();
        $(
            base.push($segment);
        )*
        base
    }}
}