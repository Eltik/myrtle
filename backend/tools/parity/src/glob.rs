use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Seg {
    Literal(String),
    One,
    Many,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerGlob {
    source: String,
    segs: Vec<Seg>,
}

impl PointerGlob {
    pub fn parse(source: &str) -> Result<Self, String> {
        if source.is_empty() {
            return Ok(Self {
                source: String::new(),
                segs: Vec::new(),
            });
        }
        let Some(rest) = source.strip_prefix('/') else {
            return Err(format!(
                "pointer glob {source:?} must be empty or start with '/'"
            ));
        };
        let segs = rest
            .split('/')
            .map(|s| match s {
                "*" => Seg::One,
                "**" => Seg::Many,
                lit => Seg::Literal(unescape(lit)),
            })
            .collect();
        Ok(Self {
            source: source.to_owned(),
            segs,
        })
    }

    pub fn matches<S: AsRef<str>>(&self, path: &[S]) -> bool {
        match_segs(&self.segs, path)
    }
}

impl fmt::Display for PointerGlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.source)
    }
}

fn match_segs<S: AsRef<str>>(pat: &[Seg], path: &[S]) -> bool {
    match pat.split_first() {
        None => path.is_empty(),
        Some((Seg::Many, rest)) => (0..=path.len()).any(|skip| match_segs(rest, &path[skip..])),
        Some((Seg::One, rest)) => !path.is_empty() && match_segs(rest, &path[1..]),
        Some((Seg::Literal(lit), rest)) => path
            .split_first()
            .is_some_and(|(head, tail)| head.as_ref() == lit && match_segs(rest, tail)),
    }
}

fn unescape(seg: &str) -> String {
    seg.replace("~1", "/").replace("~0", "~") // RFC 6901 order
}

pub fn escape_segment(seg: &str) -> String {
    seg.replace('~', "~0").replace('/', "~1")
}

pub fn format_pointer<S: AsRef<str>>(path: &[S]) -> String {
    let mut out = String::new();
    for seg in path {
        out.push('/');
        out.push_str(&escape_segment(seg.as_ref()));
    }
    out
}

pub fn any_matches<S: AsRef<str>>(globs: &[PointerGlob], path: &[S]) -> bool {
    globs.iter().any(|g| g.matches(path))
}
