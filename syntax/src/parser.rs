use std::iter::Peekable;

use crate::ast::{
    BinaryOp, Choice, Expr, ExternDeclData, Literal, NodeId, Script, Stmt, TextPart, UnaryOp,
    VarBindingData,
};
use crate::diagnostic::{Diagnostic, DiagnosticContext, IntoDiagnostic};
use crate::scanner::LexicalError;
use crate::token::{Span, Token, TokenKind};

#[derive(Debug, Clone)]
pub enum ParseError {
    Lexical(LexicalError),
    Syntax { message: String, span: Span },
}

impl From<LexicalError> for ParseError {
    fn from(err: LexicalError) -> Self {
        ParseError::Lexical(err)
    }
}

impl IntoDiagnostic for ParseError {
    fn into_diagnostic(self, ctx: &DiagnosticContext) -> Diagnostic {
        match self {
            ParseError::Lexical(lex_err) => lex_err.into_diagnostic(ctx),
            ParseError::Syntax { message, span } => {
                Diagnostic::error(format!("syntax error: {}", message), span, &message)
            }
        }
    }
}

pub struct Parser<'a, I: Iterator<Item = Result<Token<'a>, LexicalError>>> {
    tokens: Peekable<I>,
    errors: Vec<ParseError>,
    next_id: usize,
}

impl<'a, I: Iterator<Item = Result<Token<'a>, LexicalError>>> Parser<'a, I> {
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
            errors: Vec::new(),
            next_id: 0,
        }
    }

    fn next_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Check if the next token has the given kind (without consuming it)
    fn check(&mut self, kind: TokenKind) -> bool {
        matches!(self.tokens.peek(), Some(Ok(t)) if t.kind == kind)
    }

    /// Get the span of the current peeked token, or a zero-span if none
    fn current_span(&mut self) -> Span {
        match self.tokens.peek() {
            Some(Ok(t)) => t.span,
            _ => Span { start: 0, end: 0 },
        }
    }

    /// Consume and return the next token.
    /// Only call when you've already verified a token exists via peek/check.
    fn advance(&mut self) -> Token<'a> {
        self.tokens.next().unwrap().unwrap()
    }

    /// Try to parse a statement from the current token.
    /// Returns None for non-statement tokens (NewLine, Indent, Dedent, Eof, etc.)
    fn try_parse_statement(&mut self) -> Option<Stmt> {
        match self.tokens.peek() {
            Some(Ok(t)) => match t.kind {
                TokenKind::Temp => Some(self.temp_declaration()),
                TokenKind::Save => Some(self.save_declaration()),
                TokenKind::Extern => Some(self.extern_declaration()),
                TokenKind::Set => Some(self.assignment()),
                TokenKind::TextSegment | TokenKind::OpenBrace => Some(self.line_statement()),
                TokenKind::Choice => Some(self.choice_set()),
                TokenKind::If => Some(self.if_statement()),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn parse(mut self) -> Result<Script, Vec<ParseError>> {
        let mut statements = Vec::new();

        loop {
            // Handle errors first
            if matches!(self.tokens.peek(), Some(Err(_))) {
                if let Some(Err(e)) = self.tokens.next() {
                    self.errors.push(e.into());
                }
                self.synchronize();
                continue;
            }

            // Try to parse a statement
            if let Some(stmt) = self.try_parse_statement() {
                statements.push(stmt);
                continue;
            }

            // Handle non-statement tokens
            match self.tokens.peek() {
                None => break,
                Some(Ok(token)) => match token.kind {
                    TokenKind::NewLine | TokenKind::Indent | TokenKind::Dedent => {
                        // Skip newlines and indent/dedent tokens at top level
                        self.advance();
                    }
                    TokenKind::Eof => break,
                    _ => {
                        // Unexpected token at statement level
                        let span = token.span;
                        let kind = token.kind;
                        self.errors.push(ParseError::Syntax {
                            message: format!("Unexpected token: {:?}", kind),
                            span,
                        });
                        self.advance();
                    }
                },
                Some(Err(_)) => unreachable!(), // Handled above
            }
        }

        if self.errors.is_empty() {
            Ok(Script { statements })
        } else {
            Err(self.errors)
        }
    }

    /// Parse a temp declaration: temp name = value
    fn temp_declaration(&mut self) -> Stmt {
        let start_token = self.advance(); // Consume 'temp'
        let data = self.parse_var_binding("temp", start_token.span.start);
        Stmt::TempDecl(data)
    }

    /// Parse a save declaration: save name = value
    fn save_declaration(&mut self) -> Stmt {
        let start_token = self.advance(); // Consume 'save'
        let data = self.parse_var_binding("save", start_token.span.start);
        Stmt::SaveDecl(data)
    }

    /// Parse an extern declaration: extern name (no initializer)
    fn extern_declaration(&mut self) -> Stmt {
        let start_token = self.advance(); // Consume 'extern'
        let id = self.next_id();

        // Expect identifier (no = literal for extern)
        let (name, end) = if self.check(TokenKind::Identifier) {
            let token = self.advance();
            (token.lexeme.to_string(), token.span.end)
        } else {
            let span = self.current_span();
            self.errors.push(ParseError::Syntax {
                message: "Expected identifier after 'extern'".to_string(),
                span,
            });
            self.synchronize();
            (String::new(), start_token.span.end)
        };

        Stmt::ExternDecl(ExternDeclData {
            id,
            name,
            span: Span {
                start: start_token.span.start,
                end,
            },
        })
    }

    /// Parse an assignment: set name = value
    fn assignment(&mut self) -> Stmt {
        let start_token = self.advance(); // Consume 'set'
        let data = self.parse_var_binding("set", start_token.span.start);
        Stmt::Assignment(data)
    }

    /// Parse an expression inside interpolation braces using precedence-climbing.
    /// Entry point for expression parsing. Returns the expression and its span.
    fn parse_interpolation_expr(&mut self, start: usize) -> Option<(Expr, Span)> {
        let (expr, end) = self.parse_logical_or()?;
        let span = Span { start, end };
        Some((expr, span))
    }

    /// Parse logical OR: logical_and ( "or" logical_and )*
    /// Lowest precedence binary operator.
    fn parse_logical_or(&mut self) -> Option<(Expr, usize)> {
        let (mut left, mut end) = self.parse_logical_and()?;

        while let Some(Ok(t)) = self.tokens.peek() {
            if t.kind != TokenKind::Or {
                break;
            }
            let op_token = self.advance();

            match self.parse_logical_and() {
                Some((right, right_end)) => {
                    let span = Span {
                        start: left.span().start,
                        end: right_end,
                    };
                    left = Expr::Binary {
                        id: self.next_id(),
                        left: Box::new(left),
                        op: BinaryOp::Or,
                        right: Box::new(right),
                        span,
                    };
                    end = right_end;
                }
                None => {
                    self.errors.push(ParseError::Syntax {
                        message: "Expected expression after 'or'".to_string(),
                        span: op_token.span,
                    });
                    return None;
                }
            }
        }

        Some((left, end))
    }

    /// Parse logical AND: equality ( "and" equality )*
    /// Higher precedence than OR, lower than equality.
    fn parse_logical_and(&mut self) -> Option<(Expr, usize)> {
        let (mut left, mut end) = self.parse_equality()?;

        while let Some(Ok(t)) = self.tokens.peek() {
            if t.kind != TokenKind::And {
                break;
            }
            let op_token = self.advance();

            match self.parse_equality() {
                Some((right, right_end)) => {
                    let span = Span {
                        start: left.span().start,
                        end: right_end,
                    };
                    left = Expr::Binary {
                        id: self.next_id(),
                        left: Box::new(left),
                        op: BinaryOp::And,
                        right: Box::new(right),
                        span,
                    };
                    end = right_end;
                }
                None => {
                    self.errors.push(ParseError::Syntax {
                        message: "Expected expression after 'and'".to_string(),
                        span: op_token.span,
                    });
                    return None;
                }
            }
        }

        Some((left, end))
    }

    /// Parse equality: comparison ( ( "==" | "!=" ) comparison )*
    fn parse_equality(&mut self) -> Option<(Expr, usize)> {
        let (mut left, mut end) = self.parse_comparison()?;

        while let Some(Ok(t)) = self.tokens.peek() {
            let op = match t.kind {
                TokenKind::EqualEqual => BinaryOp::Equal,
                TokenKind::BangEqual => BinaryOp::NotEqual,
                _ => break,
            };
            let op_token = self.advance();

            match self.parse_comparison() {
                Some((right, right_end)) => {
                    let span = Span {
                        start: left.span().start,
                        end: right_end,
                    };
                    left = Expr::Binary {
                        id: self.next_id(),
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                        span,
                    };
                    end = right_end;
                }
                None => {
                    self.errors.push(ParseError::Syntax {
                        message: "Expected expression after equality operator".to_string(),
                        span: op_token.span,
                    });
                    return None;
                }
            }
        }

        Some((left, end))
    }

    /// Parse comparison: term ( ( "<" | "<=" | ">" | ">=" ) term )*
    fn parse_comparison(&mut self) -> Option<(Expr, usize)> {
        let (mut left, mut end) = self.parse_term()?;

        while let Some(Ok(t)) = self.tokens.peek() {
            let op = match t.kind {
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                _ => break,
            };
            let op_token = self.advance();

            match self.parse_term() {
                Some((right, right_end)) => {
                    let span = Span {
                        start: left.span().start,
                        end: right_end,
                    };
                    left = Expr::Binary {
                        id: self.next_id(),
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                        span,
                    };
                    end = right_end;
                }
                None => {
                    self.errors.push(ParseError::Syntax {
                        message: "Expected expression after comparison operator".to_string(),
                        span: op_token.span,
                    });
                    return None;
                }
            }
        }

        Some((left, end))
    }

    /// Parse term: factor ( ( "+" | "-" ) factor )*
    fn parse_term(&mut self) -> Option<(Expr, usize)> {
        let (mut left, mut end) = self.parse_factor()?;

        while let Some(Ok(t)) = self.tokens.peek() {
            let op = match t.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,
                _ => break,
            };
            let op_token = self.advance();

            match self.parse_factor() {
                Some((right, right_end)) => {
                    let span = Span {
                        start: left.span().start,
                        end: right_end,
                    };
                    left = Expr::Binary {
                        id: self.next_id(),
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                        span,
                    };
                    end = right_end;
                }
                None => {
                    self.errors.push(ParseError::Syntax {
                        message: "Expected expression after arithmetic operator".to_string(),
                        span: op_token.span,
                    });
                    return None;
                }
            }
        }

        Some((left, end))
    }

    /// Parse factor: unary ( ( "*" | "/" | "%" ) unary )*
    fn parse_factor(&mut self) -> Option<(Expr, usize)> {
        let (mut left, mut end) = self.parse_unary()?;

        while let Some(Ok(t)) = self.tokens.peek() {
            let op = match t.kind {
                TokenKind::Star => BinaryOp::Multiply,
                TokenKind::Slash => BinaryOp::Divide,
                TokenKind::Percent => BinaryOp::Modulo,
                _ => break,
            };
            let op_token = self.advance();

            match self.parse_unary() {
                Some((right, right_end)) => {
                    let span = Span {
                        start: left.span().start,
                        end: right_end,
                    };
                    left = Expr::Binary {
                        id: self.next_id(),
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                        span,
                    };
                    end = right_end;
                }
                None => {
                    self.errors.push(ParseError::Syntax {
                        message: "Expected expression after arithmetic operator".to_string(),
                        span: op_token.span,
                    });
                    return None;
                }
            }
        }

        Some((left, end))
    }

    /// Parse unary: ( "-" | "not" )* primary
    /// Unary is right-recursive for right associativity (--x parses as -(-x), not not x parses as not (not x))
    fn parse_unary(&mut self) -> Option<(Expr, usize)> {
        if let Some(Ok(t)) = self.tokens.peek() {
            if t.kind == TokenKind::Minus {
                let op_token = self.advance();
                let (expr, end) = self.parse_unary()?;
                let span = Span {
                    start: op_token.span.start,
                    end,
                };
                return Some((
                    Expr::Unary {
                        id: self.next_id(),
                        op: UnaryOp::Negate,
                        expr: Box::new(expr),
                        span,
                    },
                    end,
                ));
            }
            if t.kind == TokenKind::Not {
                let op_token = self.advance();
                // Right-recursive for right-associativity: "not not x" = "not (not x)"
                let (expr, end) = self.parse_unary()?;
                let span = Span {
                    start: op_token.span.start,
                    end,
                };
                return Some((
                    Expr::Unary {
                        id: self.next_id(),
                        op: UnaryOp::Not,
                        expr: Box::new(expr),
                        span,
                    },
                    end,
                ));
            }
        }
        self.parse_primary()
    }

    /// Parse primary: identifier | literal | "(" expression ")"
    fn parse_primary(&mut self) -> Option<(Expr, usize)> {
        match self.tokens.peek() {
            Some(Ok(t)) => match t.kind {
                // Parenthesized expression
                TokenKind::OpenParen => {
                    self.advance(); // consume '('
                    let (expr, _) = self.parse_logical_or()?;
                    if self.check(TokenKind::CloseParen) {
                        let close = self.advance();
                        Some((expr, close.span.end))
                    } else {
                        let span = self.current_span();
                        self.errors.push(ParseError::Syntax {
                            message: "Expected ')' after expression".to_string(),
                            span,
                        });
                        None
                    }
                }
                TokenKind::Identifier => {
                    let token = self.advance();
                    Some((
                        Expr::VarRef {
                            id: self.next_id(),
                            name: token.lexeme.to_string(),
                            span: token.span,
                        },
                        token.span.end,
                    ))
                }
                TokenKind::String => {
                    let token = self.advance();
                    let s = unescape_string(&token.lexeme[1..token.lexeme.len() - 1]);
                    Some((
                        Expr::Literal {
                            value: Literal::String(s),
                            span: token.span,
                        },
                        token.span.end,
                    ))
                }
                TokenKind::Number => {
                    let token = self.advance();
                    let n: f64 = token.lexeme.parse().unwrap_or(0.0);
                    Some((
                        Expr::Literal {
                            value: Literal::Number(n),
                            span: token.span,
                        },
                        token.span.end,
                    ))
                }
                TokenKind::True => {
                    let token = self.advance();
                    Some((
                        Expr::Literal {
                            value: Literal::Bool(true),
                            span: token.span,
                        },
                        token.span.end,
                    ))
                }
                TokenKind::False => {
                    let token = self.advance();
                    Some((
                        Expr::Literal {
                            value: Literal::Bool(false),
                            span: token.span,
                        },
                        token.span.end,
                    ))
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// Parse a variable binding: identifier = expression
    /// Used by temp declarations, save declarations, and assignments.
    /// The keyword token should already be consumed.
    fn parse_var_binding(&mut self, keyword: &str, start: usize) -> VarBindingData {
        let id = self.next_id();

        // Expect identifier
        let name = if self.check(TokenKind::Identifier) {
            let token = self.advance();
            token.lexeme.to_string()
        } else {
            let span = self.current_span();
            self.errors.push(ParseError::Syntax {
                message: format!("Expected identifier after '{}'", keyword),
                span,
            });
            self.synchronize();
            return VarBindingData {
                id,
                name: String::new(),
                init_expr: Expr::Literal {
                    value: Literal::Bool(false),
                    span: Span { start, end: start },
                },
                span: Span { start, end: start },
            };
        };

        // Expect '='
        if self.check(TokenKind::Equals) {
            self.advance();
        } else {
            let span = self.current_span();
            self.errors.push(ParseError::Syntax {
                message: format!("Expected '=' in {} statement", keyword),
                span,
            });
            self.synchronize();
            return VarBindingData {
                id,
                name,
                init_expr: Expr::Literal {
                    value: Literal::Bool(false),
                    span: Span { start, end: start },
                },
                span: Span { start, end: start },
            };
        }

        // Parse expression value
        let (init_expr, end) = match self.parse_logical_or() {
            Some((expr, end)) => (expr, end),
            None => {
                let span = self.current_span();
                self.errors.push(ParseError::Syntax {
                    message: format!("Expected expression after '=' in {} statement", keyword),
                    span,
                });
                (
                    Expr::Literal {
                        value: Literal::Bool(false),
                        span: Span { start, end: start },
                    },
                    start,
                )
            }
        };

        VarBindingData {
            id,
            name,
            init_expr,
            span: Span { start, end },
        }
    }

    /// Parse a line statement (text content with possible interpolation)
    fn line_statement(&mut self) -> Stmt {
        let (parts, span) = self.parse_text_parts();
        Stmt::Line { parts, span }
    }

    /// Parse text parts until newline (TextSegment, interpolations)
    fn parse_text_parts(&mut self) -> (Vec<TextPart>, Span) {
        let mut parts = Vec::new();
        let mut start: Option<usize> = None;
        let mut end: usize = 0;

        loop {
            match self.tokens.peek() {
                Some(Ok(t)) => match t.kind {
                    TokenKind::TextSegment => {
                        let token = self.advance();
                        if start.is_none() {
                            start = Some(token.span.start);
                        }
                        end = token.span.end;
                        parts.push(TextPart::Literal {
                            text: token.lexeme.to_string(),
                            span: token.span,
                        });
                    }
                    TokenKind::OpenBrace => {
                        let open = self.advance();
                        if start.is_none() {
                            start = Some(open.span.start);
                        }

                        // Parse expression inside braces using the new expression grammar
                        match self.parse_interpolation_expr(open.span.start) {
                            Some((expr, expr_span)) => {
                                // Expect close brace
                                if self.check(TokenKind::CloseBrace) {
                                    let close = self.advance();
                                    end = close.span.end;
                                    let full_span = Span {
                                        start: open.span.start,
                                        end: close.span.end,
                                    };
                                    parts.push(TextPart::Expr {
                                        expr,
                                        span: full_span,
                                    });
                                } else {
                                    let span = self.current_span();
                                    self.errors.push(ParseError::Syntax {
                                        message: "Expected '}' after expression".to_string(),
                                        span,
                                    });
                                    end = expr_span.end;
                                }
                            }
                            None => {
                                self.errors.push(ParseError::Syntax {
                                    message: "Expected expression after '{'".to_string(),
                                    span: open.span,
                                });
                                end = open.span.end;
                            }
                        }
                    }
                    TokenKind::NewLine | TokenKind::Eof | TokenKind::Dedent => {
                        // End of text content
                        break;
                    }
                    _ => {
                        // Unexpected token in text
                        break;
                    }
                },
                Some(Err(_)) => {
                    if let Some(Err(e)) = self.tokens.next() {
                        self.errors.push(e.into());
                    }
                    break;
                }
                None => break,
            }
        }

        let span = Span {
            start: start.unwrap_or(0),
            end,
        };
        (parts, span)
    }

    fn choice_set(&mut self) -> Stmt {
        let mut choices = Vec::new();

        loop {
            // Consume the Choice token ("- ")
            let choice_token = self.advance();
            let start = choice_token.span.start;

            // Parse the choice text (may contain interpolation)
            let (parts, text_span) = self.parse_text_parts();
            let end = if text_span.end > 0 {
                text_span.end
            } else {
                choice_token.span.end
            };

            // Expect newline after choice text
            if !matches!(self.tokens.peek(), Some(Ok(t)) if t.kind == TokenKind::NewLine) {
                self.errors.push(ParseError::Syntax {
                    message: "Expected newline after choice".to_string(),
                    span: Span { start, end },
                });
                self.synchronize();
                break;
            }

            self.advance(); // Consume the NewLine

            // Parse any nested content under this choice
            let nested = self.parse_nested_content();

            choices.push(Choice {
                parts,
                span: Span { start, end },
                nested,
            });

            if !matches!(self.tokens.peek(), Some(Ok(t)) if t.kind == TokenKind::Choice) {
                break;
            }
        }
        Stmt::ChoiceSet { choices }
    }

    /// Parse if statement: if expr NEWLINE INDENT stmts DEDENT [elseif...] [else...]
    fn if_statement(&mut self) -> Stmt {
        let start_token = self.advance(); // Consume 'if'
        let start = start_token.span.start;
        let id = self.next_id();

        // Parse condition expression (reuse existing expression parsing)
        let condition = match self.parse_logical_or() {
            Some((expr, _)) => expr,
            None => {
                let span = self.current_span();
                self.errors.push(ParseError::Syntax {
                    message: "Expected condition expression after 'if'".to_string(),
                    span,
                });
                self.synchronize();
                return self.error_if_stmt(id, start);
            }
        };

        // Expect NewLine, Indent, parse block
        let then_branch = self.parse_indented_block("if");
        if then_branch.is_empty() {
            let span = self.current_span();
            self.errors.push(ParseError::Syntax {
                message: "if block cannot be empty".to_string(),
                span,
            });
        }

        // Parse elseif branches
        let mut elseif_branches = Vec::new();
        while self.check(TokenKind::Elseif) {
            self.advance(); // Consume 'elseif'
            let elseif_cond = match self.parse_logical_or() {
                Some((expr, _)) => expr,
                None => {
                    let span = self.current_span();
                    self.errors.push(ParseError::Syntax {
                        message: "Expected condition expression after 'elseif'".to_string(),
                        span,
                    });
                    continue;
                }
            };
            let elseif_stmts = self.parse_indented_block("elseif");
            if elseif_stmts.is_empty() {
                let span = self.current_span();
                self.errors.push(ParseError::Syntax {
                    message: "elseif block cannot be empty".to_string(),
                    span,
                });
            }
            elseif_branches.push((elseif_cond, elseif_stmts));
        }

        // Parse optional else branch
        let else_branch = if self.check(TokenKind::Else) {
            self.advance(); // Consume 'else'
            let else_stmts = self.parse_indented_block("else");
            if else_stmts.is_empty() {
                let span = self.current_span();
                self.errors.push(ParseError::Syntax {
                    message: "else block cannot be empty".to_string(),
                    span,
                });
            }
            Some(else_stmts)
        } else {
            None
        };

        let end = self.current_span().end;
        Stmt::If {
            id,
            condition,
            then_branch,
            elseif_branches,
            else_branch,
            span: Span { start, end },
        }
    }

    /// Parse indented block: NEWLINE INDENT stmts DEDENT
    /// Returns empty vec on error (caller should report appropriate error)
    fn parse_indented_block(&mut self, context: &str) -> Vec<Stmt> {
        // Skip NewLine tokens
        while self.check(TokenKind::NewLine) {
            self.advance();
        }

        // Expect Indent
        if !self.check(TokenKind::Indent) {
            let span = self.current_span();
            self.errors.push(ParseError::Syntax {
                message: format!("Expected indented block after '{}'", context),
                span,
            });
            return Vec::new();
        }
        self.advance(); // Consume Indent

        // Parse statements until Dedent
        let mut statements = Vec::new();
        loop {
            // Handle errors
            if matches!(self.tokens.peek(), Some(Err(_))) {
                if let Some(Err(e)) = self.tokens.next() {
                    self.errors.push(e.into());
                }
                self.synchronize();
                continue;
            }

            // Try to parse a statement
            if let Some(stmt) = self.try_parse_statement() {
                statements.push(stmt);
                continue;
            }

            // Handle structural tokens
            match self.tokens.peek() {
                Some(Ok(t)) => match t.kind {
                    TokenKind::NewLine => {
                        self.advance();
                    }
                    TokenKind::Dedent => {
                        self.advance();
                        break;
                    }
                    TokenKind::Eof => break,
                    // Elseif/Else at same indent level ends the block
                    TokenKind::Elseif | TokenKind::Else => break,
                    _ => {
                        let span = t.span;
                        self.errors.push(ParseError::Syntax {
                            message: format!("Unexpected token in {} block", context),
                            span,
                        });
                        self.advance();
                    }
                },
                None => break,
                Some(Err(_)) => unreachable!(),
            }
        }

        statements
    }

    /// Create error placeholder for if statement
    fn error_if_stmt(&self, id: NodeId, start: usize) -> Stmt {
        Stmt::If {
            id,
            condition: Expr::Literal {
                value: Literal::Bool(false),
                span: Span { start, end: start },
            },
            then_branch: Vec::new(),
            elseif_branches: Vec::new(),
            else_branch: None,
            span: Span { start, end: start },
        }
    }

    /// Parse nested content under a choice (after Indent, before Dedent).
    /// Returns empty Vec if no nested content.
    fn parse_nested_content(&mut self) -> Vec<Stmt> {
        // Check if there's an Indent token
        if !matches!(self.tokens.peek(), Some(Ok(t)) if t.kind == TokenKind::Indent) {
            return Vec::new();
        }

        self.advance(); // Consume the Indent

        let mut statements = Vec::new();

        loop {
            // Handle errors first
            if matches!(self.tokens.peek(), Some(Err(_))) {
                if let Some(Err(e)) = self.tokens.next() {
                    self.errors.push(e.into());
                }
                self.synchronize();
                continue;
            }

            // Try to parse a statement
            if let Some(stmt) = self.try_parse_statement() {
                statements.push(stmt);
                continue;
            }

            // Handle non-statement tokens
            match self.tokens.peek() {
                None => break,
                Some(Ok(token)) => match token.kind {
                    TokenKind::Dedent => {
                        self.advance(); // Consume Dedent
                        break;
                    }
                    TokenKind::NewLine | TokenKind::Indent => {
                        self.advance();
                    }
                    TokenKind::Eof => break,
                    _ => {
                        // Skip unexpected tokens
                        self.advance();
                    }
                },
                Some(Err(_)) => unreachable!(), // Handled above
            }
        }

        statements
    }

    fn synchronize(&mut self) {
        loop {
            match self.tokens.peek() {
                None => return,
                Some(Err(_)) => {
                    if let Some(Err(e)) = self.tokens.next() {
                        self.errors.push(e.into());
                    }
                }
                Some(Ok(token)) => match token.kind {
                    TokenKind::NewLine => {
                        self.tokens.next();
                        return;
                    }
                    TokenKind::Eof => return,
                    _ => {
                        self.tokens.next();
                    }
                },
            }
        }
    }
}

/// Unescape a string literal (handle \n, \t, \", \\)
fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;

    fn parse_source(source: &str) -> Result<Script, Vec<ParseError>> {
        let scanner = Scanner::new(source);
        let parser = Parser::new(scanner.tokens());
        parser.parse()
    }

    fn get_text_parts(source: &str) -> Vec<TextPart> {
        let script = parse_source(source).expect("Parse failed");
        match &script.statements[0] {
            Stmt::Line { parts, .. } => parts.clone(),
            _ => panic!("Expected Line statement"),
        }
    }

    // === Success Cases: Equality Expressions ===

    #[test]
    fn parse_equality_two_variables() {
        let parts = get_text_parts("{x == y}");
        assert!(matches!(
            &parts[0],
            TextPart::Expr {
                expr: Expr::Binary { op: BinaryOp::Equal, .. },
                ..
            }
        ));
    }

    #[test]
    fn parse_inequality_two_variables() {
        let parts = get_text_parts("{x != y}");
        assert!(matches!(
            &parts[0],
            TextPart::Expr {
                expr: Expr::Binary { op: BinaryOp::NotEqual, .. },
                ..
            }
        ));
    }

    #[test]
    fn parse_equality_with_string() {
        let parts = get_text_parts("{x == \"hello\"}");
        assert!(matches!(
            &parts[0],
            TextPart::Expr {
                expr: Expr::Binary { op: BinaryOp::Equal, .. },
                ..
            }
        ));
        // Also verify right operand is a string literal
        if let TextPart::Expr { expr: Expr::Binary { right, .. }, .. } = &parts[0] {
            assert!(matches!(right.as_ref(), Expr::Literal { value: Literal::String(_), .. }));
        }
    }

    #[test]
    fn parse_equality_with_number() {
        let parts = get_text_parts("{x == 42}");
        if let TextPart::Expr { expr: Expr::Binary { right, .. }, .. } = &parts[0] {
            assert!(matches!(right.as_ref(), Expr::Literal { value: Literal::Number(_), .. }));
        } else {
            panic!("Expected Binary expression");
        }
    }

    #[test]
    fn parse_equality_with_negative_number() {
        let parts = get_text_parts("{x == -5}");
        if let TextPart::Expr { expr: Expr::Binary { right, .. }, .. } = &parts[0] {
            // -5 is now parsed as Unary(Negate, Literal(5)), not Literal(-5)
            assert!(matches!(right.as_ref(), Expr::Unary { op: UnaryOp::Negate, .. }));
        } else {
            panic!("Expected Binary expression");
        }
    }

    #[test]
    fn parse_equality_with_true() {
        let parts = get_text_parts("{x == true}");
        if let TextPart::Expr { expr: Expr::Binary { right, .. }, .. } = &parts[0] {
            assert!(matches!(right.as_ref(), Expr::Literal { value: Literal::Bool(true), .. }));
        } else {
            panic!("Expected Binary expression");
        }
    }

    #[test]
    fn parse_equality_with_false() {
        let parts = get_text_parts("{x == false}");
        if let TextPart::Expr { expr: Expr::Binary { right, .. }, .. } = &parts[0] {
            assert!(matches!(right.as_ref(), Expr::Literal { value: Literal::Bool(false), .. }));
        } else {
            panic!("Expected Binary expression");
        }
    }

    // === Simple Variable References ===

    #[test]
    fn parse_simple_var_ref_still_works() {
        let parts = get_text_parts("{name}");
        assert!(matches!(
            &parts[0],
            TextPart::Expr { expr: Expr::VarRef { name, .. }, .. } if name == "name"
        ));
    }

    #[test]
    fn parse_text_with_var_ref() {
        let parts = get_text_parts("Hello, {name}!");
        assert_eq!(parts.len(), 3);
        assert!(matches!(&parts[0], TextPart::Literal { .. }));
        assert!(matches!(&parts[1], TextPart::Expr { expr: Expr::VarRef { .. }, .. }));
        assert!(matches!(&parts[2], TextPart::Literal { .. }));
    }

    // === Edge Cases ===

    #[test]
    fn parse_equality_no_spaces() {
        let parts = get_text_parts("{x==y}");
        assert!(matches!(&parts[0], TextPart::Expr { expr: Expr::Binary { .. }, .. }));
    }

    #[test]
    fn parse_equality_extra_spaces() {
        let parts = get_text_parts("{x  ==  y}");
        assert!(matches!(&parts[0], TextPart::Expr { expr: Expr::Binary { .. }, .. }));
    }

    #[test]
    fn parse_mixed_content() {
        let parts = get_text_parts("Result: {x == y} done");
        assert_eq!(parts.len(), 3);
        assert!(matches!(&parts[1], TextPart::Expr { expr: Expr::Binary { .. }, .. }));
    }

    // === Error Cases ===

    #[test]
    fn parse_error_missing_right_operand() {
        let result = parse_source("{x ==}");
        assert!(result.is_err());
    }

    #[test]
    fn parse_arithmetic_addition() {
        let parts = get_text_parts("{x + y}");
        assert!(matches!(
            &parts[0],
            TextPart::Expr {
                expr: Expr::Binary { op: BinaryOp::Add, .. },
                ..
            }
        ));
    }

    #[test]
    fn parse_arithmetic_subtraction() {
        let parts = get_text_parts("{x - y}");
        assert!(matches!(
            &parts[0],
            TextPart::Expr {
                expr: Expr::Binary { op: BinaryOp::Subtract, .. },
                ..
            }
        ));
    }

    #[test]
    fn parse_arithmetic_multiplication() {
        let parts = get_text_parts("{x * y}");
        assert!(matches!(
            &parts[0],
            TextPart::Expr {
                expr: Expr::Binary { op: BinaryOp::Multiply, .. },
                ..
            }
        ));
    }

    #[test]
    fn parse_arithmetic_division() {
        let parts = get_text_parts("{x / y}");
        assert!(matches!(
            &parts[0],
            TextPart::Expr {
                expr: Expr::Binary { op: BinaryOp::Divide, .. },
                ..
            }
        ));
    }

    #[test]
    fn parse_arithmetic_modulo() {
        let parts = get_text_parts("{x % y}");
        assert!(matches!(
            &parts[0],
            TextPart::Expr {
                expr: Expr::Binary { op: BinaryOp::Modulo, .. },
                ..
            }
        ));
    }

    #[test]
    fn parse_unary_negation() {
        let parts = get_text_parts("{-x}");
        assert!(matches!(
            &parts[0],
            TextPart::Expr {
                expr: Expr::Unary { op: UnaryOp::Negate, .. },
                ..
            }
        ));
    }

    #[test]
    fn parse_parenthesized_expression() {
        let parts = get_text_parts("{(x + y)}");
        assert!(matches!(
            &parts[0],
            TextPart::Expr {
                expr: Expr::Binary { op: BinaryOp::Add, .. },
                ..
            }
        ));
    }

    #[test]
    fn parse_precedence_mul_over_add() {
        // 2 + 3 * 4 should parse as 2 + (3 * 4)
        let parts = get_text_parts("{2 + 3 * 4}");
        if let TextPart::Expr { expr: Expr::Binary { left, op, right, .. }, .. } = &parts[0] {
            assert_eq!(*op, BinaryOp::Add);
            // Left should be literal 2
            assert!(matches!(left.as_ref(), Expr::Literal { value: Literal::Number(n), .. } if *n == 2.0));
            // Right should be 3 * 4
            assert!(matches!(right.as_ref(), Expr::Binary { op: BinaryOp::Multiply, .. }));
        } else {
            panic!("Expected Binary expression");
        }
    }

    #[test]
    fn parse_parentheses_override_precedence() {
        // (2 + 3) * 4 should parse as (2 + 3) * 4
        let parts = get_text_parts("{(2 + 3) * 4}");
        if let TextPart::Expr { expr: Expr::Binary { left, op, right, .. }, .. } = &parts[0] {
            assert_eq!(*op, BinaryOp::Multiply);
            // Left should be 2 + 3
            assert!(matches!(left.as_ref(), Expr::Binary { op: BinaryOp::Add, .. }));
            // Right should be literal 4
            assert!(matches!(right.as_ref(), Expr::Literal { value: Literal::Number(n), .. } if *n == 4.0));
        } else {
            panic!("Expected Binary expression");
        }
    }

    #[test]
    fn parse_error_unclosed_parenthesis() {
        let result = parse_source("{(x + y}");
        assert!(result.is_err());
    }

    #[test]
    fn parse_error_missing_right_operand_add() {
        let result = parse_source("{x +}");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod choice_else_tests {
    use super::*;
    use crate::scanner::Scanner;

    #[test]
    fn test_choice_in_if_with_else() {
        let source = r#"temp condition = true
if condition
    - Choice A
        Action A
    - Choice B
        Action B
else
    Alternative."#;

        let scanner = Scanner::new(source);
        let parser = Parser::new(scanner.tokens());
        let result = parser.parse();

        match result {
            Ok(script) => {
                println!("Parse successful! Statements: {:#?}", script.statements);
                // Should have 2 statements: temp declaration and if statement
                assert_eq!(script.statements.len(), 2);
            }
            Err(errors) => {
                for e in &errors {
                    println!("Parse error: {:?}", e);
                }
                panic!("Expected parse to succeed but got {} errors", errors.len());
            }
        }
    }
}

#[cfg(test)]
mod token_debug_tests {
    use crate::scanner::Scanner;
    use crate::token::TokenKind;

    #[test]
    fn debug_token_stream() {
        let source = r#"temp condition = true
if condition
    - Choice A
        Action A
    - Choice B
        Action B
else
    Alternative."#;

        println!("\n=== TOKEN STREAM ===");
        let scanner = Scanner::new(source);
        for (i, tok) in scanner.tokens().enumerate() {
            match tok {
                Ok(t) => {
                    let lexeme_display = if t.lexeme.contains('\n') {
                        "\n".to_string()
                    } else if t.lexeme.is_empty() {
                        "<empty>".to_string()
                    } else {
                        format!("{:?}", t.lexeme)
                    };
                    println!("{:3}: {:15?} {}", i, t.kind, lexeme_display);
                }
                Err(e) => println!("{:3}: ERROR {:?}", i, e),
            }
        }
    }
}
