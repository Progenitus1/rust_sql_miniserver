use super::errors::{ParseError, ParseResult};
use super::tokenizer::tokenize;

use std::fmt;

#[derive(Debug, PartialEq, Clone, Default)]
pub enum LexerToken {
    Select,
    Insert,
    Delete,
    Create,
    Drop,
    Table,
    Index,
    Where,
    From,
    Into,
    On,
    Values,
    #[default]
    Null,
    StringLiteral(String),
    NumberLiteral(i32),
    FloatNumberLiteral(f64), // does not impl Eq
    BoolLiteral(bool),
    Identifier(String),
    Comma,
    Semicolon,
    Star,
    Plus,
    Minus,
    Slash,
    // <, >, =, ...
    CompareOp(String),
    ParOpen,
    ParClose,
    // Same as CompareOp, I would maybe change this to enum value per data type
    DataType(String),
    LogicalOp(String),
    Not,
    ExclamationMark,
    Percent,
}

impl fmt::Display for LexerToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerToken::Select => write!(f, "select"),
            LexerToken::Insert => write!(f, "insert"),
            LexerToken::Delete => write!(f, "delete"),
            LexerToken::Create => write!(f, "create"),
            LexerToken::Drop => write!(f, "drop"),
            LexerToken::Table => write!(f, "table"),
            LexerToken::Index => write!(f, "index"),
            LexerToken::Where => write!(f, "where"),
            LexerToken::From => write!(f, "from"),
            LexerToken::Into => write!(f, "into"),
            LexerToken::On => write!(f, "on"),
            LexerToken::Values => write!(f, "values"),
            LexerToken::Null => write!(f, "null"),
            LexerToken::StringLiteral(s) => write!(f, "{}", s),
            LexerToken::NumberLiteral(i) => write!(f, "{}", i),
            LexerToken::FloatNumberLiteral(fl) => write!(f, "{}", fl),
            LexerToken::BoolLiteral(b) => write!(f, "{}", b),
            LexerToken::Identifier(s) => write!(f, "{}", s),
            LexerToken::Comma => write!(f, ","),
            LexerToken::Semicolon => write!(f, ";"),
            LexerToken::Star => write!(f, "*"),
            LexerToken::Plus => write!(f, "+"),
            LexerToken::Minus => write!(f, "-"),
            LexerToken::Slash => write!(f, "/"),
            LexerToken::CompareOp(s) => write!(f, "{}", s),
            LexerToken::ParOpen => write!(f, "("),
            LexerToken::ParClose => write!(f, ")"),
            LexerToken::DataType(s) => write!(f, "{}", s),
            LexerToken::LogicalOp(s) => write!(f, "{}", s),
            LexerToken::Not => write!(f, "not"),
            LexerToken::ExclamationMark => write!(f, "!"),
            LexerToken::Percent => write!(f, "%"),
        }
    }
}

pub fn lex(input: &str) -> ParseResult<Vec<LexerToken>> {
    let mut tokens = Vec::new();

    for token_str in tokenize(input)? {
        let token_lower = token_str.to_lowercase();
        match token_lower.as_str() {
            // todo: "as" ???
            "select" => tokens.push(LexerToken::Select),
            "insert" => tokens.push(LexerToken::Insert),
            "delete" => tokens.push(LexerToken::Delete),
            "create" => tokens.push(LexerToken::Create),
            "drop" => tokens.push(LexerToken::Drop),
            "table" => tokens.push(LexerToken::Table),
            "index" => tokens.push(LexerToken::Index),
            // we do not need to have 'update' implemented
            "where" => tokens.push(LexerToken::Where),
            "from" => tokens.push(LexerToken::From),
            "into" => tokens.push(LexerToken::Into),
            "on" => tokens.push(LexerToken::On),
            "values" => tokens.push(LexerToken::Values),
            "null" => tokens.push(LexerToken::Null),
            "true" => tokens.push(LexerToken::BoolLiteral(true)),
            "false" => tokens.push(LexerToken::BoolLiteral(false)),
            "=" | "!=" | ">" | "<" | "<=" | ">=" | "<>" => {
                tokens.push(LexerToken::CompareOp(token_str.into()))
            }
            "(" => tokens.push(LexerToken::ParOpen),
            ")" => tokens.push(LexerToken::ParClose),
            // TODO: which data types we want to have ?
            "int" | "varchar" | "float" | "boolean" => {
                tokens.push(LexerToken::DataType(token_lower.clone()))
            }
            "and" | "or" | "xor" => tokens.push(LexerToken::LogicalOp(token_lower.clone())),
            "not" => tokens.push(LexerToken::Not),
            "*" => tokens.push(LexerToken::Star),
            "+" => tokens.push(LexerToken::Plus),
            "-" => tokens.push(LexerToken::Minus),
            "/" => tokens.push(LexerToken::Slash),
            "%" => tokens.push(LexerToken::Percent),
            "," => tokens.push(LexerToken::Comma),
            ";" => tokens.push(LexerToken::Semicolon),
            "!" => tokens.push(LexerToken::ExclamationMark),
            _ => {
                if (token_str.starts_with('"') && token_str.ends_with('"'))
                    || (token_str.starts_with('\'') && token_str.ends_with('\''))
                {
                    tokens.push(LexerToken::StringLiteral(
                        token_str[1..token_str.len() - 1].into(),
                    ));
                } else if let Ok(number) = token_lower.parse::<i32>() {
                    // token_lower is already String, use it for num parsing
                    tokens.push(LexerToken::NumberLiteral(number));
                } else if let Ok(number) = token_lower.parse::<f64>() {
                    tokens.push(LexerToken::FloatNumberLiteral(number));
                } else {
                    for token_char in token_str.chars() {
                        if !(token_char.is_alphanumeric() || ['.', '_', '-'].contains(&token_char))
                        {
                            return Err(ParseError::InvalidIdentifier(
                                token_char,
                                token_str.into(),
                            ));
                        }
                    }
                    tokens.push(LexerToken::Identifier(token_str.into()));
                }
            }
        };
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separator_in_string_literal() {
        let expr = stringify!(insert "ahoj, dobry; vecer" "hello \" world");
        println!("{:?}", lex(expr));
    }

    #[test]
    fn test_lex_select() {
        let expr = "select * from table_id;";
        assert_eq!(
            vec![
                LexerToken::Select,
                LexerToken::Star,
                LexerToken::From,
                LexerToken::Identifier("table_id".into()),
                LexerToken::Semicolon
            ],
            lex(expr).unwrap()
        );
    }

    #[test]
    fn test_lex_insert() {
        let expr = "insert \"ahoj\", -3, nUlL, 3.0 into table_name";
        assert_eq!(
            vec![
                LexerToken::Insert,
                LexerToken::StringLiteral("ahoj".into()),
                LexerToken::Comma,
                LexerToken::Minus,
                LexerToken::NumberLiteral(3),
                LexerToken::Comma,
                LexerToken::Null,
                LexerToken::Comma,
                LexerToken::FloatNumberLiteral(3.0),
                LexerToken::Into,
                LexerToken::Identifier("table_name".into())
            ],
            lex(expr).unwrap()
        );
    }

    #[test]
    fn test_insert() {
        let expr = "INSERT INTO films VALUES ('UA502', 'Bananas', 105, '1971-07-13', 'Comedy', '82 minutes')";
        assert_eq!(
            vec![
                LexerToken::Insert,
                LexerToken::Into,
                LexerToken::Identifier("films".into()),
                LexerToken::Values,
                LexerToken::ParOpen,
                LexerToken::StringLiteral("UA502".into()),
                LexerToken::Comma,
                LexerToken::StringLiteral("Bananas".into()),
                LexerToken::Comma,
                LexerToken::NumberLiteral(105),
                LexerToken::Comma,
                LexerToken::StringLiteral("1971-07-13".into()),
                LexerToken::Comma,
                LexerToken::StringLiteral("Comedy".into()),
                LexerToken::Comma,
                LexerToken::StringLiteral("82 minutes".into()),
                LexerToken::ParClose,
            ],
            lex(expr).unwrap()
        );
    }

    #[test]
    fn test_insert_specify_columns() {
        let expr = "INSERT INTO films (code, title, did, date_prod, kind) VALUES ('T_601', 'Yojimbo', 106, '1961-06-16', 'Drama');";
        assert_eq!(
            vec![
                LexerToken::Insert,
                LexerToken::Into,
                LexerToken::Identifier("films".into()),
                LexerToken::ParOpen,
                LexerToken::Identifier("code".into()),
                LexerToken::Comma,
                LexerToken::Identifier("title".into()),
                LexerToken::Comma,
                LexerToken::Identifier("did".into()),
                LexerToken::Comma,
                LexerToken::Identifier("date_prod".into()),
                LexerToken::Comma,
                LexerToken::Identifier("kind".into()),
                LexerToken::ParClose,
                LexerToken::Values,
                LexerToken::ParOpen,
                LexerToken::StringLiteral("T_601".into()),
                LexerToken::Comma,
                LexerToken::StringLiteral("Yojimbo".into()),
                LexerToken::Comma,
                LexerToken::NumberLiteral(106),
                LexerToken::Comma,
                LexerToken::StringLiteral("1961-06-16".into()),
                LexerToken::Comma,
                LexerToken::StringLiteral("Drama".into()),
                LexerToken::ParClose,
                LexerToken::Semicolon,
            ],
            lex(expr).unwrap()
        );
    }

    #[test]
    fn test_delete() {
        let expr = "delete from table_name where x = 1";
        assert_eq!(
            vec![
                LexerToken::Delete,
                LexerToken::From,
                LexerToken::Identifier("table_name".to_string()),
                LexerToken::Where,
                LexerToken::Identifier("x".to_string()),
                LexerToken::CompareOp("=".to_string()),
                LexerToken::NumberLiteral(1),
            ],
            lex(expr).unwrap()
        );
    }

    #[test]
    fn test_create_table() {
        let expr = "create table table_name x int, y varchar";
        assert_eq!(
            vec![
                LexerToken::Create,
                LexerToken::Table,
                LexerToken::Identifier("table_name".to_string()),
                LexerToken::Identifier("x".to_string()),
                LexerToken::DataType("int".to_string()),
                LexerToken::Comma,
                LexerToken::Identifier("y".to_string()),
                LexerToken::DataType("varchar".to_string()),
            ],
            lex(expr).unwrap()
        );
    }

    #[test]
    fn test_drop_table() {
        let expr = "drop table table_name";
        assert_eq!(
            vec![
                LexerToken::Drop,
                LexerToken::Table,
                LexerToken::Identifier("table_name".to_string()),
            ],
            lex(expr).unwrap()
        );
    }

    #[test]
    fn test_create_index() {
        let expr = "create index column_name on table_name";
        assert_eq!(
            vec![
                LexerToken::Create,
                LexerToken::Index,
                LexerToken::Identifier("column_name".to_string()),
                LexerToken::On,
                LexerToken::Identifier("table_name".to_string()),
            ],
            lex(expr).unwrap()
        );
    }
}
