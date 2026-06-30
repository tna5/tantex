use tantivy::tokenizer::{Token, TokenStream, Tokenizer};

pub fn register_custom_tokenizers(index: &tantivy::Index) {
    let mgr = index.tokenizers();
    mgr.register("sorted", SortedTokenizer);
    mgr.register("raw_lower", RawLowerTokenizer);
}

// ---------------------------------------------------------------------------
// SortedTokenizer — splits on Unicode whitespace only, lowercases, sorts
// alphabetically, re-emits with sequential positions 0..n. Phrase matching
// becomes order-independent ("Jean Dupont" matches "Dupont Jean"). Hyphenated
// tokens are preserved as a single token ("Dupont-Leroux" is not split), so
// "Jean Leroux" does not match "Jean Dupont-Leroux". Cross-element false
// positives are prevented by tantivy's per-value position gap.
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct SortedTokenizer;

pub struct SortedTokenStream {
    tokens: Vec<Token>,
    index: usize,
}

impl TokenStream for SortedTokenStream {
    fn advance(&mut self) -> bool {
        self.index += 1;
        self.index <= self.tokens.len()
    }
    fn token(&self) -> &Token {
        &self.tokens[self.index - 1]
    }
    fn token_mut(&mut self) -> &mut Token {
        &mut self.tokens[self.index - 1]
    }
}

impl Tokenizer for SortedTokenizer {
    type TokenStream<'a> = SortedTokenStream;

    fn token_stream<'a>(&mut self, text: &'a str) -> SortedTokenStream {
        let mut tokens: Vec<Token> = text
            .split(|c: char| c.is_whitespace())
            .filter(|s| !s.is_empty() && s.len() <= 40)
            .map(|s| Token {
                offset_from: 0,
                offset_to: 0,
                position: 0,
                text: s.to_lowercase(),
                position_length: 1,
            })
            .collect();

        tokens.sort_by(|a, b| a.text.cmp(&b.text));
        for (i, t) in tokens.iter_mut().enumerate() {
            t.position = i;
        }

        SortedTokenStream { tokens, index: 0 }
    }
}

// ---------------------------------------------------------------------------
// RawLowerTokenizer — emits the entire input as a single lowercased token.
// Used for emails, usernames: exact case-insensitive match.
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct RawLowerTokenizer;

pub struct RawLowerTokenStream {
    token: Token,
    done: bool,
}

impl TokenStream for RawLowerTokenStream {
    fn advance(&mut self) -> bool {
        if self.done {
            return false;
        }
        self.done = true;
        true
    }
    fn token(&self) -> &Token {
        &self.token
    }
    fn token_mut(&mut self) -> &mut Token {
        &mut self.token
    }
}

impl Tokenizer for RawLowerTokenizer {
    type TokenStream<'a> = RawLowerTokenStream;

    fn token_stream<'a>(&mut self, text: &'a str) -> RawLowerTokenStream {
        RawLowerTokenStream {
            token: Token {
                offset_from: 0,
                offset_to: text.len(),
                position: 0,
                text: text.to_lowercase(),
                position_length: 1,
            },
            done: false,
        }
    }
}
