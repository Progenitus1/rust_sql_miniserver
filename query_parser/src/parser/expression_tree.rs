use super::{errors::ParseError, lexer::LexerToken};

#[derive(Debug, PartialEq)]
pub enum Node {
    Leaf(LexerToken),
    Binary {
        left: Box<Node>,
        op: LexerToken,
        right: Box<Node>,
    },
    Unary {
        op: LexerToken,
        node: Box<Node>,
    },
}

#[allow(dead_code)]
impl Node {
    pub fn new_binary(left: Node, op: LexerToken, right: Node) -> Self {
        Node::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    pub fn new_unary(op: LexerToken, node: Node) -> Self {
        Node::Unary {
            op,
            node: Box::new(node),
        }
    }

    pub fn collect_identifiers(&self, identifiers: &mut Vec<String>) {
        match self {
            Node::Leaf(LexerToken::Identifier(identifier)) => identifiers.push(identifier.clone()),
            Node::Leaf(_) => {}
            Node::Binary { left, right, .. } => {
                left.collect_identifiers(identifiers);
                right.collect_identifiers(identifiers);
            }
            Node::Unary { node, .. } => node.collect_identifiers(identifiers),
        }
    }
}

pub fn parse_tree(expression: Vec<LexerToken>) -> Result<Option<Node>, ParseError> {
    let expression = fix_operator_precedence(expression);
    let mut parser = ExpressionTreeParser::from(expression);
    parser.parse()
}

struct ExpressionTreeParser {
    tokens: Vec<LexerToken>,
    index: usize,
}

impl ExpressionTreeParser {
    fn from(tokens: Vec<LexerToken>) -> Self {
        ExpressionTreeParser { tokens, index: 0 }
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn head(&self) -> Option<&LexerToken> {
        self.tokens.get(self.index)
    }

    fn eof(&self) -> bool {
        self.index >= self.tokens.len()
    }

    fn expect_head(&self) -> Result<&LexerToken, ParseError> {
        self.head().ok_or(ParseError::UnexpectedQueryEnding)
    }

    fn parse(&mut self) -> Result<Option<Node>, ParseError> {
        if self.tokens.is_empty() {
            return Ok(None);
        }

        let mut node = self.parse_start(false)?;
        while !self.eof() {
            node = self.parse_leaf_or_binary(node)?;
        }
        Ok(Some(node))
    }

    fn parse_start(&mut self, parenthesised: bool) -> Result<Node, ParseError> {
        let head = self.expect_head()?.clone();
        let node = match head {
            LexerToken::Minus | LexerToken::Not | LexerToken::ExclamationMark => {
                self.advance();
                let node = self.parse_unary(head)?;
                Ok(node)
            }
            LexerToken::Null
            | LexerToken::StringLiteral(_)
            | LexerToken::NumberLiteral(_)
            | LexerToken::BoolLiteral(_)
            | LexerToken::FloatNumberLiteral(_)
            | LexerToken::Identifier(_) => {
                self.advance();
                self.parse_leaf_or_binary(Node::Leaf(head))
            }
            LexerToken::ParOpen => {
                self.advance();
                let par_node = self.parse_start(true)?;
                self.parse_leaf_or_binary(par_node)
            }
            _ => Err(ParseError::UnexpectedToken(
                "identifier, literal, unary operator, (".into(),
                head,
            )),
        }?;

        if parenthesised {
            match self.head() {
                Some(LexerToken::ParClose) => self.advance(),
                _ => return Err(ParseError::UnfinishedParenthesis),
            }
        }

        Ok(node)
    }

    fn parse_leaf_or_binary(&mut self, left: Node) -> Result<Node, ParseError> {
        if self.eof() {
            return Ok(left);
        }

        let head = self.expect_head()?.clone();

        match head {
            LexerToken::CompareOp(_)
            | LexerToken::LogicalOp(_)
            | LexerToken::Star
            | LexerToken::Plus
            | LexerToken::Minus
            | LexerToken::Slash
            | LexerToken::Percent => {
                self.advance();
                let right = self.parse_start(false)?;

                Ok(Node::Binary {
                    left: Box::new(left),
                    op: head,
                    right: Box::new(right),
                })
            }
            // todo: check other variants
            _ => Ok(left),
        }
    }

    fn parse_unary(&mut self, unary_op: LexerToken) -> Result<Node, ParseError> {
        let node = self.parse_start(false)?;

        Ok(Node::Unary {
            op: unary_op,
            node: Box::new(node),
        })
    }
}

// See Alternative methods in https://en.wikipedia.org/wiki/Operator-precedence_parser
#[allow(clippy::vec_init_then_push)]
fn fix_operator_precedence(expression: Vec<LexerToken>) -> Vec<LexerToken> {
    if expression.len() <= 2 {
        return expression;
    }

    let mut result = Vec::new();
    result.push(LexerToken::ParOpen);
    result.push(LexerToken::ParOpen);
    result.push(LexerToken::ParOpen);
    result.push(LexerToken::ParOpen);

    for token in expression {
        match &token {
            // operator_precedence: 4
            LexerToken::LogicalOp(_) => {
                result.push(LexerToken::ParClose);
                result.push(LexerToken::ParClose);
                result.push(LexerToken::ParClose);
                result.push(LexerToken::ParClose);
                result.push(token);
                result.push(LexerToken::ParOpen);
                result.push(LexerToken::ParOpen);
                result.push(LexerToken::ParOpen);
                result.push(LexerToken::ParOpen);
            }
            // operator_precedence: 3
            LexerToken::CompareOp(_) => {
                result.push(LexerToken::ParClose);
                result.push(LexerToken::ParClose);
                result.push(LexerToken::ParClose);
                result.push(token);
                result.push(LexerToken::ParOpen);
                result.push(LexerToken::ParOpen);
                result.push(LexerToken::ParOpen);
            }
            // operator_precedence: 2
            LexerToken::Plus | LexerToken::Minus => {
                if result.last() == Some(&LexerToken::ParOpen) {
                    result.push(token);
                    continue;
                }
                result.push(LexerToken::ParClose);
                result.push(LexerToken::ParClose);
                result.push(token);
                result.push(LexerToken::ParOpen);
                result.push(LexerToken::ParOpen);
            }
            // operator_precedence: 1
            LexerToken::Star | LexerToken::Slash | LexerToken::Percent => {
                result.push(LexerToken::ParClose);
                result.push(token);
                result.push(LexerToken::ParOpen);
            }
            LexerToken::ParOpen => {
                result.push(token);
                result.push(LexerToken::ParOpen);
                result.push(LexerToken::ParOpen);
                result.push(LexerToken::ParOpen);
                result.push(LexerToken::ParOpen);
            }
            LexerToken::ParClose => {
                result.push(LexerToken::ParClose);
                result.push(LexerToken::ParClose);
                result.push(LexerToken::ParClose);
                result.push(LexerToken::ParClose);
                result.push(token);
            }
            _ => result.push(token),
        }
    }

    result.push(LexerToken::ParClose);
    result.push(LexerToken::ParClose);
    result.push(LexerToken::ParClose);
    result.push(LexerToken::ParClose);

    result
}

#[test]
fn test_basic_stuff() {
    // let expression = lex("not (x = (1 + 2))").unwrap();
    // let expression = lex("2 + 3 + 1").unwrap();
    let expression = crate::parser::lexer::lex(stringify!((x = 100) and (abc = "abc"))).unwrap();

    let mut parser = ExpressionTreeParser::from(expression);
    let tree = parser.parse().unwrap().unwrap();

    dbg!(tree);
}
