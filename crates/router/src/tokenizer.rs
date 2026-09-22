// This Source Code Form is governed by the
// Node-Locked Network Public License v1.0 (NLNPL-1.0).
//
// File-level copyleft applies to this Covered File.
//
// Network-service use may trigger source-disclosure obligations.
//
// Execution may require a valid Licensor-issued Node Key.
//
// See LICENSE for complete terms.
//
// PRIOR ART BADGE: April 14, 2026 — Project inception.
// SPDX-License-Identifier: LicenseRef-NLNPL-1.0

//! Deterministic tokenizer for SnapKitty sovereign agents.
//!
//! Same input always yields the same token sequence, ids, and derived
//! amounts. No RNG, no locale tables, no ML. ASCII is lowercased; other
//! letters are lowercased via Unicode simple mapping. Token ids are FNV-1a
//! 64-bit hashes of `(kind, text)`.

use std::collections::BTreeSet;

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

/// FNV-1a 64 — stable across platforms and Rust versions.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenKind {
    Word,
    Number,
    Amount,
    Ident,
}

impl TokenKind {
    fn tag(self) -> &'static str {
        match self {
            TokenKind::Word => "word",
            TokenKind::Number => "number",
            TokenKind::Amount => "amount",
            TokenKind::Ident => "ident",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub text: String,
    pub kind: TokenKind,
    pub id: u64,
    pub amount: Option<f64>,
}

impl Token {
    fn new(kind: TokenKind, text: String, amount: Option<f64>) -> Self {
        let id = token_id(kind, &text);
        Self {
            text,
            kind,
            id,
            amount,
        }
    }
}

fn token_id(kind: TokenKind, text: &str) -> u64 {
    let mut hash = fnv1a64(kind.tag().as_bytes());
    hash ^= 0xff;
    hash = hash.wrapping_mul(FNV_PRIME);
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Ordered token stream plus derived lookup sets for intent routing.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenStream {
    pub tokens: Vec<Token>,
    words: BTreeSet<String>,
    phrases: BTreeSet<String>,
    amounts: Vec<f64>,
}

impl TokenStream {
    fn from_tokens(tokens: Vec<Token>) -> Self {
        let mut words = BTreeSet::new();
        let mut phrases = BTreeSet::new();
        let mut amounts = Vec::new();
        let mut sequential_words: Vec<String> = Vec::new();

        for token in &tokens {
            if let Some(amount) = token.amount {
                amounts.push(amount);
            }
            match token.kind {
                TokenKind::Word | TokenKind::Ident => {
                    expand_lexeme(&token.text, &mut words, &mut phrases);
                    sequential_words.push(token.text.clone());
                }
                TokenKind::Number | TokenKind::Amount => {
                    sequential_words.push(token.text.clone());
                }
            }
        }

        for window in sequential_words.windows(2) {
            if is_wordish(&window[0]) && is_wordish(&window[1]) {
                phrases.insert(format!("{} {}", window[0], window[1]));
            }
        }
        for window in sequential_words.windows(3) {
            if window.iter().all(|w| is_wordish(w)) {
                phrases.insert(format!("{} {} {}", window[0], window[1], window[2]));
            }
        }

        Self {
            tokens,
            words,
            phrases,
            amounts,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    pub fn has(&self, word: &str) -> bool {
        self.words.contains(word)
    }

    pub fn has_any(&self, words: &[&str]) -> bool {
        words.iter().any(|word| self.has(word))
    }

    pub fn has_phrase(&self, phrase: &str) -> bool {
        self.phrases.contains(phrase)
    }

    pub fn amounts(&self) -> &[f64] {
        &self.amounts
    }

    pub fn first_amount(&self) -> Option<f64> {
        self.amounts.first().copied()
    }

    pub fn idents(&self) -> impl Iterator<Item = &str> {
        self.tokens.iter().filter_map(|token| {
            if token.kind == TokenKind::Ident {
                Some(token.text.as_str())
            } else {
                None
            }
        })
    }

    /// Fingerprint of the full stream — equal streams share equal ids.
    pub fn fingerprint(&self) -> u64 {
        let mut hash = FNV_OFFSET;
        for token in &self.tokens {
            for byte in token.id.to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(FNV_PRIME);
            }
        }
        hash
    }
}

fn is_wordish(text: &str) -> bool {
    text.chars()
        .next()
        .map(|c| c.is_ascii_alphabetic())
        .unwrap_or(false)
}

fn expand_lexeme(text: &str, words: &mut BTreeSet<String>, phrases: &mut BTreeSet<String>) {
    words.insert(text.to_string());
    if text.contains('-') {
        let parts: Vec<&str> = text.split('-').filter(|part| !part.is_empty()).collect();
        for part in &parts {
            words.insert((*part).to_string());
        }
        if parts.len() >= 2 {
            phrases.insert(parts.join(" "));
        }
    }
}

/// Tokenize `input` into a deterministic stream.
pub fn tokenize(input: &str) -> TokenStream {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        if c == '$' {
            chars.next();
            if let Some((text, value)) = read_number(&mut chars) {
                tokens.push(Token::new(TokenKind::Amount, format!("${text}"), Some(value)));
            }
            continue;
        }
        if c.is_ascii_digit() {
            if let Some((text, value)) = read_number(&mut chars) {
                tokens.push(Token::new(TokenKind::Number, text, Some(value)));
            }
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let raw = read_lexeme(&mut chars);
            let text = normalize(&raw);
            let kind = classify_lexeme(&text);
            tokens.push(Token::new(kind, text, None));
            continue;
        }
        chars.next();
    }

    TokenStream::from_tokens(tokens)
}

fn normalize(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        if c.is_ascii_uppercase() {
            out.push((c as u8 - b'A' + b'a') as char);
        } else if c.is_ascii() {
            out.push(c);
        } else {
            for lower in c.to_lowercase() {
                out.push(lower);
            }
        }
    }
    out
}

fn classify_lexeme(text: &str) -> TokenKind {
    let has_digit = text.chars().any(|c| c.is_ascii_digit());
    let has_sep = text.contains('_') || text.contains('-');
    if has_digit || (has_sep && text.chars().any(|c| c.is_ascii_alphabetic())) {
        if text.chars().all(|c| c.is_ascii_digit() || c == '-' || c == '_') && has_digit {
            TokenKind::Ident
        } else if has_digit || text.contains('_') {
            TokenKind::Ident
        } else {
            TokenKind::Word
        }
    } else {
        TokenKind::Word
    }
}

fn read_lexeme(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut buf = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
            buf.push(c);
            chars.next();
        } else {
            break;
        }
    }
    buf
}

fn read_number(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<(String, f64)> {
    let mut raw = String::new();
    let mut saw_digit = false;
    let mut saw_dot = false;

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            raw.push(c);
            saw_digit = true;
            chars.next();
        } else if c == ',' {
            chars.next();
        } else if c == '.' && !saw_dot {
            raw.push('.');
            saw_dot = true;
            chars.next();
        } else {
            break;
        }
    }

    if !saw_digit {
        return None;
    }
    let value = raw.parse::<f64>().ok()?;
    Some((raw, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determinism() {
        let a = tokenize("Approve PO #po_002 for $75,000 — vendor v_001");
        let b = tokenize("Approve PO #po_002 for $75,000 — vendor v_001");
        assert_eq!(a, b);
        assert_eq!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn extracts_amount_and_idents() {
        let stream = tokenize("PO po_002 $75,000 vendor v_001");
        assert_eq!(stream.first_amount(), Some(75_000.0));
        let idents: Vec<&str> = stream.idents().collect();
        assert!(idents.contains(&"po_002"));
        assert!(idents.contains(&"v_001"));
    }

    #[test]
    fn exact_token_match_not_substring() {
        let stream = tokenize("point of sale");
        assert!(!stream.has("po"));
        assert!(stream.has("point"));
        assert!(stream.has_phrase("point of") || stream.has("point"));
    }

    #[test]
    fn hyphen_expands_to_phrase() {
        let stream = tokenize("triple-entry ledger");
        assert!(stream.has("triple"));
        assert!(stream.has("entry"));
        assert!(stream.has_phrase("triple entry"));
        assert!(stream.has("ledger"));
    }

    #[test]
    fn cash_flow_bigram() {
        let stream = tokenize("Show me the cash flow");
        assert!(stream.has_phrase("cash flow"));
        assert!(stream.has("cash"));
        assert!(stream.has("flow"));
    }

    #[test]
    fn case_fold_ascii() {
        let a = tokenize("Cash FLOW");
        let b = tokenize("cash flow");
        assert_eq!(a.words, b.words);
        assert_eq!(a.phrases, b.phrases);
    }

    #[test]
    fn token_ids_stable() {
        let stream = tokenize("vendor");
        let id = stream.tokens[0].id;
        assert_eq!(id, token_id(TokenKind::Word, "vendor"));
        assert_ne!(id, 0);
    }
}
