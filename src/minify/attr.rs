use std::cmp::{min, Ordering};

use aho_corasick::{AhoCorasickBuilder, MatchKind};
use lazy_static::lazy_static;

use crate::gen::attrs::ATTRS;
use crate::gen::codepoints::DIGIT;
use crate::pattern::Replacer;
use crate::spec::entity::encode::encode_entities;
use crate::spec::script::JAVASCRIPT_MIME_TYPES;
use crate::spec::tag::ns::Namespace;
use crate::whitespace::{collapse_whitespace, left_trim, right_trim};

fn build_double_quoted_replacer() -> Replacer {
    let mut patterns = Vec::<Vec<u8>>::new();
    let mut replacements = Vec::<Vec<u8>>::new();

    // Replace all `"` with `&#34`, unless the quote is followed by a digit or semicolon,
    // in which case add a semicolon to the encoded entity.
    for c in "0123456789;".bytes() {
        patterns.push(vec![b'"', c]);
        replacements.push(vec![b'&', b'#', b'3', b'4', b';', c]);
    }
    patterns.push(b"\"".to_vec());
    replacements.push(b"&#34".to_vec());

    Replacer::new(
        AhoCorasickBuilder::new()
            .dfa(true)
            .match_kind(MatchKind::LeftmostLongest)
            .build(patterns),
        replacements,
    )
}

fn build_single_quoted_replacer() -> Replacer {
    let mut patterns = Vec::<Vec<u8>>::new();
    let mut replacements = Vec::<Vec<u8>>::new();

    // Replace all `'` with `&#39`, unless the quote is followed by a digit or semicolon,
    // in which case add a semicolon to the encoded entity.
    for c in "0123456789;".bytes() {
        patterns.push(vec![b'\'', c]);
        replacements.push(vec![b'&', b'#', b'3', b'9', b';', c]);
    }
    patterns.push(b"'".to_vec());
    replacements.push(b"&#39".to_vec());

    Replacer::new(
        AhoCorasickBuilder::new()
            .dfa(true)
            .match_kind(MatchKind::LeftmostLongest)
            .build(patterns),
        replacements,
    )
}

// TODO Sync with WHITESPACE definition.
static WS: &[(u8, &[u8])] = &[
    (b'\x09', b"&#9"),
    (b'\x0a', b"&#10"),
    (b'\x0c', b"&#12"),
    (b'\x0d', b"&#13"),
    (b'\x20', b"&#32"),
];

fn build_unquoted_replacer() -> Replacer {
    let mut patterns = Vec::<Vec<u8>>::new();
    let mut replacements = Vec::<Vec<u8>>::new();

    // Replace all whitespace with a numeric entity, unless the whitespace is followed by a digit or semicolon,
    // in which case add a semicolon to the encoded entity.
    for c in "0123456789;".bytes() {
        for &(ws, rep) in WS {
            patterns.push(vec![ws, c]);
            replacements.push({
                let mut ent = rep.to_vec();
                ent.push(b';');
                ent.push(c);
                ent
            });
        }
    }
    for &(ws, rep) in WS {
        patterns.push(vec![ws]);
        replacements.push(rep.to_vec());
    }

    // Replace all `>` with `&GT`, unless the chevron is followed by a semicolon,
    // in which case add a semicolon to the encoded entity.
    // Use `&GT` instead of `&gt` as `&gt` has more conflicting entities e.g. `&gtcc;`, `&gtdot;`.
    patterns.push(b">;".to_vec());
    replacements.push(b"&GT;;".to_vec());
    patterns.push(b">".to_vec());
    replacements.push(b"&GT".to_vec());

    Replacer::new(
        AhoCorasickBuilder::new()
            .dfa(true)
            .match_kind(MatchKind::LeftmostLongest)
            .build(patterns),
        replacements,
    )
}

lazy_static! {
    static ref DOUBLE_QUOTED_REPLACER: Replacer = build_double_quoted_replacer();
    static ref SINGLE_QUOTED_REPLACER: Replacer = build_single_quoted_replacer();
    static ref UNQUOTED_QUOTED_REPLACER: Replacer = build_unquoted_replacer();
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum AttrType {
    Redundant,
    NoValue,
    Quoted,
    Unquoted,
}

pub struct AttrValMinified {
    typ: AttrType,
    prefix: &'static [u8],
    data: Vec<u8>,
    start: usize,
    suffix: &'static [u8],
}

impl Eq for AttrValMinified {}

impl PartialEq<Self> for AttrValMinified {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len()
    }
}

impl PartialOrd<Self> for AttrValMinified {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.len().partial_cmp(&other.len())
    }
}

impl Ord for AttrValMinified {
    fn cmp(&self, other: &Self) -> Ordering {
        self.len().cmp(&other.len())
    }
}

impl AttrValMinified {
    pub fn len(&self) -> usize {
        self.prefix.len() + (self.data.len() - self.start) + self.suffix.len()
    }

    pub fn out(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.prefix);
        out.extend_from_slice(&self.data[self.start..]);
        out.extend_from_slice(self.suffix);
    }

    #[cfg(test)]
    pub fn str(&self) -> String {
        let mut out = Vec::with_capacity(self.len());
        self.out(&mut out);
        String::from_utf8(out).unwrap()
    }

    pub fn typ(&self) -> AttrType {
        self.typ
    }
}

pub fn encode_using_double_quotes(val: &[u8]) -> AttrValMinified {
    AttrValMinified {
        typ: AttrType::Quoted,
        prefix: b"\"",
        data: DOUBLE_QUOTED_REPLACER.replace_all(val),
        start: 0,
        suffix: b"\"",
    }
}

pub fn encode_using_single_quotes(val: &[u8]) -> AttrValMinified {
    AttrValMinified {
        typ: AttrType::Quoted,
        prefix: b"'",
        data: SINGLE_QUOTED_REPLACER.replace_all(val),
        start: 0,
        suffix: b"'",
    }
}

pub fn encode_unquoted(val: &[u8]) -> AttrValMinified {
    let data = UNQUOTED_QUOTED_REPLACER.replace_all(val);
    let prefix: &'static [u8] = match data.get(0) {
        Some(b'"') => match data.get(1) {
            Some(&s) if DIGIT[s] || s == b';' => b"&#34;",
            _ => b"&#34",
        },
        Some(b'\'') => match data.get(1) {
            Some(&s) if DIGIT[s] || s == b';' => b"&#39;",
            _ => b"&#39",
        },
        _ => b"",
    };
    let start = if !prefix.is_empty() { 1 } else { 0 };
    AttrValMinified {
        typ: AttrType::Unquoted,
        prefix,
        data,
        start,
        suffix: b"",
    }
}

pub fn minify_attr_val(
    ns: Namespace,
    tag: &[u8],
    name: &[u8],
    mut value_raw: Vec<u8>,
) -> AttrValMinified {
    let attr_cfg = ATTRS.get(ns, tag, name);

    let should_collapse_and_trim = attr_cfg.filter(|attr| attr.collapse_and_trim).is_some();
    let is_boolean = attr_cfg.filter(|attr| attr.boolean).is_some();
    // An attribute can have both redundant_if_empty and default_value, which means it has two default values: "" and default_value.
    let redundant_if_empty = attr_cfg.filter(|attr| attr.redundant_if_empty).is_some();
    let default_value = attr_cfg.and_then(|attr| attr.default_value);

    // Trim before checking is_boolean as the entire attribute could be redundant post-minification.
    if should_collapse_and_trim {
        right_trim(&mut value_raw);
        left_trim(&mut value_raw);
        collapse_whitespace(&mut value_raw);
    };

    if (value_raw.is_empty() && redundant_if_empty)
        || default_value.filter(|dv| dv == &value_raw).is_some()
        // TODO Cfg.
        || (tag == b"script" && JAVASCRIPT_MIME_TYPES.contains(value_raw.as_slice()))
    {
        return AttrValMinified {
            typ: AttrType::Redundant,
            prefix: b"",
            data: Vec::new(),
            start: 0,
            suffix: b"",
        };
    };

    if is_boolean {
        return AttrValMinified {
            typ: AttrType::NoValue,
            prefix: b"",
            data: Vec::new(),
            start: 0,
            suffix: b"",
        };
    };

    let encoded = encode_entities(&value_raw, true);

    // When lengths are equal, prefer double quotes to all and single quotes to unquoted.
    min(
        min(
            encode_using_double_quotes(&encoded),
            encode_using_single_quotes(&encoded),
        ),
        encode_unquoted(&encoded),
    )
}
