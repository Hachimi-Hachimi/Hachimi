/*
    Simple templating language parser/evaluator for localization strings.

    Syntax:
    - Filter: $(filter_name arg1 arg2 arg3 ...)
      Loosely based on Bash command substitution syntax.

    More expression types might be added later, but the filter expression
    is already suitable for most if not all use cases.
*/
use fnv::FnvHashMap;

pub enum Token {
    Identifier(String),
    NumberLit(f64),
    StringLit(String)
}

pub type Filter = fn(args: &[Token]) -> Option<String>;

pub trait Context {
    fn on_filter_eval(&mut self, name: &str, args: &[Token]) -> Option<String>;
}

struct EmptyContext();

impl Context for EmptyContext {
    fn on_filter_eval(&mut self, _name: &str, _args: &[Token]) -> Option<String> {
        None
    }
}

struct FilterRemovalContext();

impl Context for FilterRemovalContext {
    fn on_filter_eval(&mut self, _name: &str, _args: &[Token]) -> Option<String> {
        Some(String::new())
    }
}

pub struct Parser {
    filters: FnvHashMap<String, Filter>
}

impl Parser {
    pub fn new(filters_: &[(&str, Filter)]) -> Parser {
        let mut filters = FnvHashMap::default();
        for (name, filter) in filters_ {
            filters.insert(name.to_string(), filter.to_owned());
        }

        Parser { filters }
    }

    fn eval_filter(&self, tokens: &Vec<Token>, context: &mut impl Context) -> Option<String> {
        if tokens.is_empty() { return None; }

        if let Token::Identifier(filter_name) = tokens.first().unwrap() {
            let args = &tokens.as_slice()[1..];
            let context_res = context.on_filter_eval(filter_name, args);
            if context_res.is_some() {
                return context_res
            }
            else if let Some(filter) = self.filters.get(filter_name) {
                return filter(&tokens.as_slice()[1..]);
            }
        }

        None
    }

    fn parse_token(input: &str) -> Option<Token> {
        let mut iter = input.chars();
        let start_char = iter.next().unwrap(); // guaranteed to have at least one char
        let end_char = iter.last();

        if start_char == '\'' && end_char.is_some() && end_char.unwrap() == '\'' {
            return Some(Token::StringLit(input[1..input.len() - 1].replace("\\'", "'")));
        }

        if start_char.is_numeric() {
            return if let Ok(number) = input.parse::<f64>() {
                Some(Token::NumberLit(number))
            }
            else if let Ok(number) = input.replace(",", "").parse::<f64>() {
                // Allow commas
                // (not doing in the initial parse; the idea being that numbers with commas are not common)
                Some(Token::NumberLit(number))
            }
            else {
                None
            }
        }

        if input.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Some(Token::Identifier(input.to_owned()));
        }

        None
    }

    pub fn eval(&self, input: &str) -> String {
        self.eval_with_context(input, &mut EmptyContext {})
    }

    pub fn eval_with_context(&self, input: &str, context: &mut impl Context) -> String {
        let mut output: Vec<u8> = Vec::with_capacity(input.len());

        let mut start_expr = false;
        let mut in_filter = false;
        let mut checkpoint: usize = 0;
        let mut tokens: Vec<Token> = Vec::new();
        let mut token_start: usize = 0;
        let mut in_string = false;
        let mut start_escape = false;

        // Iterate through the bytes directly for the sake of simplicity
        // (it's also faster than going through char())
        // A caveat is that the "syntax parsing" portion of the parser has
        // no knowledge of Unicode characters; it doesn't need to anyways, UTF-8
        // sequences do not conflict with normal ascii characters.
        for (i, c) in input.bytes().enumerate() {
            output.push(c);

            if in_filter {
                // Continue if string char is escaped
                if start_escape {
                    start_escape = false;
                    continue;
                }

                // Check separator and expr close
                match c {
                    b')' => 'filter_close: {
                        if in_string { break 'filter_close; }

                        // Parse token (if it hasnt been terminated by a trailing separator yet)
                        if token_start != 0 {
                            let res = Self::parse_token(&input[token_start..i]);
                            if let Some(token) = res {
                                tokens.push(token);
                                token_start = 0;
                            }
                            else {
                                warn!("Invalid token in '{}' (at pos {})", input, token_start);
                                token_start = 0;
                                tokens.clear();
                                in_filter = false;
                                break 'filter_close;
                            }
                        }

                        if let Some(res) = self.eval_filter(&tokens, context) {
                            output.truncate(checkpoint);
                            output.extend(res.bytes());
                        }
                        else {
                            warn!("Filter evaluation failed in '{}' (at pos {})", input, i);
                        }

                        tokens.clear();
                        in_filter = false;
                    },

                    b' ' => if !in_string && token_start != 0 {
                        let res = Self::parse_token(&input[token_start..i]);
                        if let Some(token) = res {
                            tokens.push(token);
                        }
                        else {
                            warn!("Invalid token in '{}' (at pos {})", input, token_start);
                            tokens.clear();
                            in_filter = false;
                        }
                        token_start = 0;
                    },

                    b'\'' => {
                        if token_start == 0 {
                            token_start = i;
                            in_string = true;
                        }
                        else {
                            in_string = false;
                        }
                    }

                    b'\\' => if in_string {
                        start_escape = true;
                    }

                    _ => if token_start == 0 {
                        token_start = i;
                    }
                }
                continue;
            }

            if start_expr {
                // Check expression opening
                if c == b'(' { // Filter expression
                    in_filter = true;
                }
                start_expr = false;
                continue;
            }

            // Check for expression start
            if c == b'$' {
                start_expr = true;
                checkpoint = output.len() - 1; // before the starting char
            }
        }

        unsafe { String::from_utf8_unchecked(output) }
    }

    /// Evaluate the template with a context that returns an empty string on any filter expr
    pub fn remove_filters(&self, input: &str) -> String {
        self.eval_with_context(input, &mut FilterRemovalContext {})
    }
}