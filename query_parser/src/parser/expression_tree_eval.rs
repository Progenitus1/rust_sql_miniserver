#[derive(Debug, Clone, PartialEq)]
pub enum NodeValue {
    Bool(bool),
    String(String),
    Int(i32),
    Float(f64),
    Null,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum NumberBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Equal,
    NotEqual,
}
impl TryFrom<&LexerToken> for NumberBinOp {
    type Error = ParseError;

    fn try_from(value: &LexerToken) -> Result<Self, Self::Error> {
        match &value {
            LexerToken::Plus => Ok(NumberBinOp::Add),
            LexerToken::Minus => Ok(NumberBinOp::Sub),
            LexerToken::Star => Ok(NumberBinOp::Mul),
            LexerToken::Slash => Ok(NumberBinOp::Div),
            LexerToken::Percent => Ok(NumberBinOp::Mod),
            LexerToken::CompareOp(op) => match op.as_str() {
                ">" => Ok(NumberBinOp::Greater),
                "<" => Ok(NumberBinOp::Less),
                ">=" => Ok(NumberBinOp::GreaterEqual),
                "<=" => Ok(NumberBinOp::LessEqual),
                "=" => Ok(NumberBinOp::Equal),
                "!=" | "<>" => Ok(NumberBinOp::NotEqual),
                _ => Err(ParseError::InvalidOperator(
                    "binary operator".into(),
                    value.clone(),
                )),
            },
            _ => Err(ParseError::InvalidOperator(
                "binary operator".into(),
                value.clone(),
            )),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum BoolBinOp {
    And,
    Or,
    Xor,
    Equal,
    NotEqual,
}

impl TryFrom<&LexerToken> for BoolBinOp {
    type Error = ParseError;

    fn try_from(value: &LexerToken) -> Result<Self, Self::Error> {
        match &value {
            LexerToken::LogicalOp(op) => match op.as_str() {
                "and" => Ok(BoolBinOp::And),
                "or" => Ok(BoolBinOp::Or),
                "xor" => Ok(BoolBinOp::Xor),
                _ => Err(ParseError::InvalidOperator(
                    "and, or, xor".into(),
                    value.clone(),
                )),
            },
            LexerToken::CompareOp(op) => match op.as_str() {
                "=" => Ok(BoolBinOp::Equal),
                "!=" | "<>" => Ok(BoolBinOp::NotEqual),
                _ => Err(ParseError::InvalidOperator(
                    "=, !=, <>".into(),
                    value.clone(),
                )),
            },
            _ => Err(ParseError::InvalidOperator(
                "binary bool operator".into(),
                value.clone(),
            )),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum StringOp {
    Concat,
    Equal,
    NotEqual,
}

impl TryFrom<&LexerToken> for StringOp {
    type Error = ParseError;

    fn try_from(value: &LexerToken) -> Result<Self, Self::Error> {
        match &value {
            LexerToken::Plus => Ok(StringOp::Concat),
            LexerToken::CompareOp(op) => match op.as_str() {
                "=" => Ok(StringOp::Equal),
                "!=" | "<>" => Ok(StringOp::NotEqual),
                _ => Err(ParseError::InvalidOperator(
                    "=, !=, <>".into(),
                    value.clone(),
                )),
            },
            _ => Err(ParseError::InvalidOperator(
                "binary string operator".into(),
                value.clone(),
            )),
        }
    }
}

use std::collections::HashMap;

use super::errors::ParseError;
use crate::parser::expression_tree::Node;
use crate::parser::lexer::LexerToken;

pub fn evaluate_binary_node(
    node: &Node,
    identifier_map: &HashMap<String, NodeValue>,
) -> Result<bool, ParseError> {
    let val = evaluate_node(node, identifier_map)?;
    match val {
        NodeValue::Bool(b) => Ok(b),
        // when NULL is the result for WHERE condition, then it is false
        NodeValue::Null => Ok(false),
        _ => Err(ParseError::InvalidType("bool".into(), val)),
    }
}

pub fn evaluate_node(
    node: &Node,
    identifier_map: &HashMap<String, NodeValue>,
) -> Result<NodeValue, ParseError> {
    match node {
        Node::Leaf(token) => evaluate_leaf(token, identifier_map),
        Node::Unary { op, node } => {
            let node_value = evaluate_node(node, identifier_map)?;

            evaluate_unary(op, node_value)
        }
        Node::Binary { left, op, right } => {
            let left_value = evaluate_node(left, identifier_map)?;
            let right_value = evaluate_node(right, identifier_map)?;

            // let the left type decide, which type of operation is expected
            match left_value {
                NodeValue::Bool(_) => evaluate_bool_op(&left_value, &right_value, op.try_into()?),
                NodeValue::String(_) => {
                    evaluate_string_op(&left_value, &right_value, op.try_into()?)
                }
                NodeValue::Int(_) | NodeValue::Float(_) => {
                    evaluate_binary_number_op(&left_value, &right_value, op.try_into()?)
                }
                NodeValue::Null => match op {
                    LexerToken::CompareOp(op) if op == "=" => {
                        Ok(NodeValue::Bool(right_value == NodeValue::Null))
                    }
                    // notice: != behaves differently than <> for null values
                    LexerToken::CompareOp(op) if op == "<>" => {
                        Ok(NodeValue::Bool(right_value != NodeValue::Null))
                    }
                    // all other operations with null result to null
                    _ => Ok(NodeValue::Null),
                },
            }
        }
    }
}

fn evaluate_leaf(
    token: &LexerToken,
    identifier_map: &HashMap<String, NodeValue>,
) -> Result<NodeValue, ParseError> {
    match token {
        LexerToken::BoolLiteral(value) => Ok(NodeValue::Bool(*value)),
        LexerToken::StringLiteral(value) => Ok(NodeValue::String(value.clone())),
        LexerToken::NumberLiteral(value) => Ok(NodeValue::Int(*value)),
        LexerToken::FloatNumberLiteral(value) => Ok(NodeValue::Float(*value)),
        LexerToken::Identifier(id) => match identifier_map.get(id) {
            None => Err(ParseError::IdentifierNotFound(id.clone())),
            Some(value) => Ok(value.clone()),
        },
        LexerToken::Null => Ok(NodeValue::Null),
        _ => Err(ParseError::UnexpectedToken(
            "leaf token".into(),
            token.clone(),
        )),
    }
}

fn evaluate_unary(op: &LexerToken, node_value: NodeValue) -> Result<NodeValue, ParseError> {
    match op {
        LexerToken::Not | LexerToken::ExclamationMark => match node_value {
            NodeValue::Bool(value) => Ok(NodeValue::Bool(!value)),
            NodeValue::Null => Ok(NodeValue::Null),
            _ => Err(ParseError::InvalidType("bool".into(), node_value)),
        },
        LexerToken::Minus => match node_value {
            NodeValue::Int(value) => Ok(NodeValue::Int(-value)),
            NodeValue::Float(value) => Ok(NodeValue::Float(-value)),
            NodeValue::Null => Ok(NodeValue::Null),
            _ => Err(ParseError::InvalidType("int, float".into(), node_value)),
        },
        _ => unreachable!("unary operator should be one of !, not, -"),
    }
}

fn evaluate_binary_number_op(
    left_value: &NodeValue,
    right_value: &NodeValue,
    op: NumberBinOp,
) -> Result<NodeValue, ParseError> {
    match (&left_value, &right_value) {
        (NodeValue::Int(i1), NodeValue::Int(i2)) => evaluate_int_number_op(*i1, *i2, op),
        (NodeValue::Float(f1), NodeValue::Float(f2)) => evaluate_float_number_op(*f1, *f2, op),
        (NodeValue::Int(i1), NodeValue::Float(f2)) => evaluate_float_number_op(*i1 as f64, *f2, op),
        (NodeValue::Float(f1), NodeValue::Int(i2)) => evaluate_float_number_op(*f1, *i2 as f64, op),
        (NodeValue::Int(_), NodeValue::Null) | (NodeValue::Float(_), NodeValue::Null) => {
            Ok(NodeValue::Null)
        }
        _ => Err(ParseError::InvalidType(
            "int, float".into(),
            right_value.clone(),
        )),
    }
}

fn evaluate_float_number_op(f1: f64, f2: f64, op: NumberBinOp) -> Result<NodeValue, ParseError> {
    match op {
        NumberBinOp::Add => Ok(NodeValue::Float(f1 + f2)),
        NumberBinOp::Sub => Ok(NodeValue::Float(f1 - f2)),
        NumberBinOp::Mul => Ok(NodeValue::Float(f1 * f2)),
        NumberBinOp::Div => Ok(NodeValue::Float(f1 / f2)),
        NumberBinOp::Mod => Ok(NodeValue::Float(f1 % f2)),
        NumberBinOp::Greater => Ok(NodeValue::Bool(f1 > f2)),
        NumberBinOp::Less => Ok(NodeValue::Bool(f1 < f2)),
        NumberBinOp::GreaterEqual => Ok(NodeValue::Bool(f1 >= f2)),
        NumberBinOp::LessEqual => Ok(NodeValue::Bool(f1 <= f2)),
        NumberBinOp::Equal => Ok(NodeValue::Bool(f1 == f2)),
        NumberBinOp::NotEqual => Ok(NodeValue::Bool(f1 != f2)),
    }
}

fn evaluate_int_number_op(i1: i32, i2: i32, op: NumberBinOp) -> Result<NodeValue, ParseError> {
    match op {
        NumberBinOp::Add => Ok(NodeValue::Int(i1 + i2)),
        NumberBinOp::Sub => Ok(NodeValue::Int(i1 - i2)),
        NumberBinOp::Mul => Ok(NodeValue::Int(i1 * i2)),
        NumberBinOp::Div => Ok(NodeValue::Int(i1 / i2)),
        NumberBinOp::Mod => Ok(NodeValue::Int(i1 % i2)),
        NumberBinOp::Greater => Ok(NodeValue::Bool(i1 > i2)),
        NumberBinOp::Less => Ok(NodeValue::Bool(i1 < i2)),
        NumberBinOp::GreaterEqual => Ok(NodeValue::Bool(i1 >= i2)),
        NumberBinOp::LessEqual => Ok(NodeValue::Bool(i1 <= i2)),
        NumberBinOp::Equal => Ok(NodeValue::Bool(i1 == i2)),
        NumberBinOp::NotEqual => Ok(NodeValue::Bool(i1 != i2)),
    }
}

fn evaluate_string_op(
    left_value: &NodeValue,
    right_value: &NodeValue,
    op: StringOp,
) -> Result<NodeValue, ParseError> {
    match (&left_value, &right_value) {
        (NodeValue::String(s1), NodeValue::String(s2)) => match op {
            StringOp::Concat => Ok(NodeValue::String(format!("{}{}", s1, s2))),
            StringOp::Equal => Ok(NodeValue::Bool(s1 == s2)),
            StringOp::NotEqual => Ok(NodeValue::Bool(s1 != s2)),
        },
        (NodeValue::String(_), NodeValue::Null) => Ok(NodeValue::Null),
        _ => Err(ParseError::InvalidType(
            "string".into(),
            right_value.clone(),
        )),
    }
}

fn evaluate_bool_op(
    left_value: &NodeValue,
    right_value: &NodeValue,
    op: BoolBinOp,
) -> Result<NodeValue, ParseError> {
    match (&left_value, &right_value) {
        (NodeValue::Bool(b1), NodeValue::Bool(b2)) => match op {
            BoolBinOp::And => Ok(NodeValue::Bool(*b1 && *b2)),
            BoolBinOp::Or => Ok(NodeValue::Bool(*b1 || *b2)),
            BoolBinOp::Xor => Ok(NodeValue::Bool(*b1 ^ *b2)),
            BoolBinOp::Equal => Ok(NodeValue::Bool(*b1 == *b2)),
            BoolBinOp::NotEqual => Ok(NodeValue::Bool(*b1 != *b2)),
        },
        (NodeValue::Bool(_), NodeValue::Null) => Ok(NodeValue::Null),
        _ => Err(ParseError::InvalidType("bool".into(), right_value.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::super::expression_tree::parse_tree;
    use super::super::lexer::lex;
    use super::*;

    fn evaluate_expression(expr: &str) -> Result<NodeValue, ParseError> {
        let expr = lex(expr).unwrap();
        let tree = parse_tree(expr).unwrap().unwrap();

        let mut map = HashMap::new();
        map.insert("x".to_string(), NodeValue::Int(100));
        map.insert("abc".to_string(), NodeValue::String("abc".into()));
        map.insert("nil".to_string(), NodeValue::Null);

        evaluate_node(&tree, &map)
    }

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(evaluate_expression("1 + 2").unwrap(), NodeValue::Int(3));
        assert_eq!(evaluate_expression("1 - 2").unwrap(), NodeValue::Int(-1));
        assert_eq!(evaluate_expression("1 * 2").unwrap(), NodeValue::Int(2));
        assert_eq!(evaluate_expression("1 / 2").unwrap(), NodeValue::Int(0));
        assert_eq!(evaluate_expression("1 % 2").unwrap(), NodeValue::Int(1));
    }

    #[test]
    fn test_int_float_arithmetic() {
        assert_eq!(
            evaluate_expression("1 + 2.0").unwrap(),
            NodeValue::Float(3.0)
        );
    }

    #[test]
    fn test_bool_expressions() {
        assert_eq!(
            evaluate_expression("x = 100").unwrap(),
            NodeValue::Bool(true)
        );
        assert_eq!(
            evaluate_expression("x != 10").unwrap(),
            NodeValue::Bool(true)
        );
        assert_eq!(
            evaluate_expression("10 < x").unwrap(),
            NodeValue::Bool(true)
        );
        assert_eq!(
            evaluate_expression("10 > x").unwrap(),
            NodeValue::Bool(false)
        );
        assert_eq!(
            evaluate_expression("100 <= x").unwrap(),
            NodeValue::Bool(true)
        );
        assert_eq!(
            evaluate_expression("100 >= x").unwrap(),
            NodeValue::Bool(true)
        );
        assert_eq!(
            evaluate_expression("x <> 100").unwrap(),
            NodeValue::Bool(false)
        );
    }

    #[test]
    fn test_string_ops() {
        assert_eq!(
            evaluate_expression(stringify!("foo" + "bar")).unwrap(),
            NodeValue::String("foobar".into())
        );

        assert_eq!(
            evaluate_expression(stringify!("foo" != "bar")).unwrap(),
            NodeValue::Bool(true)
        );

        assert_eq!(
            evaluate_expression(stringify!("foo" = "foo")).unwrap(),
            NodeValue::Bool(true)
        );
    }

    #[test]
    fn test_compound_ops() {
        assert_eq!(
            evaluate_expression("(x - 100) = (10 + 100 - 110)").unwrap(),
            NodeValue::Bool(true)
        );

        assert_eq!(
            evaluate_expression(stringify!((abc + "def") = ("abcd" + "ef"))).unwrap(),
            NodeValue::Bool(true)
        );

        assert_eq!(
            evaluate_expression(stringify!((x = 100) and (abc = "abc"))).unwrap(),
            NodeValue::Bool(true)
        );

        assert_eq!(
            evaluate_expression(stringify!((x - 101) = -1)).unwrap(),
            NodeValue::Bool(true)
        );

        assert_eq!(
            evaluate_expression("((2 * x) = (3 + (122 * 300)))").unwrap(),
            NodeValue::Bool(false)
        );
    }

    #[test]
    fn test_null_bubble_up() {
        assert_eq!(
            evaluate_expression("(nil * 3) + 2").unwrap(),
            NodeValue::Null
        );
        assert_eq!(
            evaluate_expression("(nil + \"aa\") = NULL").unwrap(),
            NodeValue::Bool(true)
        );
    }

    #[test]
    fn null_eq_null() {
        assert_eq!(
            evaluate_expression("nil = NULL").unwrap(),
            NodeValue::Bool(true)
        );

        assert_eq!(evaluate_expression("nil != NULL").unwrap(), NodeValue::Null);

        assert_eq!(
            evaluate_expression("nil <> NULL").unwrap(),
            NodeValue::Bool(false)
        );
    }

    #[test]
    fn test_minus_eq() {
        assert_eq!(
            evaluate_expression(stringify!(0 = -1)).unwrap(),
            NodeValue::Bool(false)
        );
    }

    #[test]
    fn test_unary_op() {
        assert_eq!(
            evaluate_expression(stringify!(-x)).unwrap(),
            NodeValue::Int(-100)
        );

        assert_eq!(
            evaluate_expression("not(x = 100)").unwrap(),
            NodeValue::Bool(false)
        );

        assert_eq!(
            evaluate_expression("!false").unwrap(),
            NodeValue::Bool(true)
        );

        assert_eq!(
            evaluate_expression(stringify!(-2 * -3)).unwrap(),
            NodeValue::Int(6)
        );
    }

    #[test]
    fn test_operator_precedence() {
        assert_eq!(
            evaluate_expression(stringify!(3 = 2 + 1)).unwrap(),
            NodeValue::Bool(true)
        );
        assert_eq!(
            evaluate_expression(stringify!(false = true xor true)).unwrap(),
            NodeValue::Bool(true)
        );
        assert_eq!(
            evaluate_expression(stringify!(-1 * -6)).unwrap(),
            NodeValue::Int(6)
        );
        assert_eq!(
            evaluate_expression(stringify!("abc" + abc = abc + "abc")).unwrap(),
            NodeValue::Bool(true)
        );
        assert_eq!(
            evaluate_expression(stringify!(100 >= 30 + 10)).unwrap(),
            NodeValue::Bool(true)
        );
        assert_eq!(
            evaluate_expression(stringify!((11 + 1) / 2 + 6)).unwrap(),
            NodeValue::Int(12)
        );
        assert_eq!(
            evaluate_expression(stringify!(x > 4 and x <= 130.5)).unwrap(),
            NodeValue::Bool(true)
        );
    }
}
