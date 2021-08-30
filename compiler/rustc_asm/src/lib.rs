use std::fmt;

use rustc_span::HashStableContext;
use rustc_macros::{Decodable, Encodable, HashStable_Generic};
use rustc_span::symbol::Symbol;
use rustc_span::Span;

/// Inline assembly operand explicit register or register class.
///
/// E.g., `"eax"` as in `asm!("mov eax, 2", out("eax") result)`.
#[derive(Clone, Copy, Encodable, Decodable, Debug)]
pub enum InlineAsmRegOrRegClass {
    Reg(Symbol),
    RegClass(Symbol),
}

bitflags::bitflags! {
    #[derive(Encodable, Decodable, HashStable_Generic)]
    pub struct InlineAsmOptions: u8 {
        const PURE = 1 << 0;
        const NOMEM = 1 << 1;
        const READONLY = 1 << 2;
        const PRESERVES_FLAGS = 1 << 3;
        const NORETURN = 1 << 4;
        const NOSTACK = 1 << 5;
        const ATT_SYNTAX = 1 << 6;
        const RAW = 1 << 7;
    }
}

#[derive(Clone, PartialEq, PartialOrd, Encodable, Decodable, Debug, Hash, HashStable_Generic)]
pub enum InlineAsmTemplatePiece {
    String(String),
    Placeholder { operand_idx: usize, modifier: Option<char>, span: Span },
}

impl fmt::Display for InlineAsmTemplatePiece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => {
                for c in s.chars() {
                    match c {
                        '{' => f.write_str("{{")?,
                        '}' => f.write_str("}}")?,
                        _ => c.fmt(f)?,
                    }
                }
                Ok(())
            }
            Self::Placeholder { operand_idx, modifier: Some(modifier), .. } => {
                write!(f, "{{{}:{}}}", operand_idx, modifier)
            }
            Self::Placeholder { operand_idx, modifier: None, .. } => {
                write!(f, "{{{}}}", operand_idx)
            }
        }
    }
}

impl InlineAsmTemplatePiece {
    /// Rebuilds the asm template string from its pieces.
    pub fn to_string(s: &[Self]) -> String {
        use fmt::Write;
        let mut out = String::new();
        for p in s.iter() {
            let _ = write!(out, "{}", p);
        }
        out
    }
}
