use super::errors::{ParseError, ParseResult};
use super::expression_tree::{parse_tree, Node};
use super::lexer::{lex, LexerToken};

#[derive(Debug, PartialEq)]
pub enum Query {
    Select {
        body: Vec<LexerToken>,
        table_name: String,
        where_body: Option<Node>,
    },
    Insert {
        values: Vec<LexerToken>,
        columns: Vec<String>,
        table_name: String,
    },
    Delete {
        table_name: String,
        where_body: Option<Node>,
    },
    CreateTable {
        table_name: String,
        columns_definition: Vec<(String, String)>,
    },
    CreateIndex {
        column_name: String,
        table_name: String,
    },
    DropIndex {
        column_name: String,
        table_name: String,
    },
    DropTable {
        table_name: String,
    },
}

struct QueryParser {
    tokens: Vec<LexerToken>,
    index: usize,
}

impl QueryParser {
    fn from(tokens: Vec<LexerToken>) -> Self {
        QueryParser { tokens, index: 0 }
    }

    /// Return the token on current index and advance the index.
    fn next(&mut self) -> Option<&LexerToken> {
        let tok = self.tokens.get(self.index);
        self.index += 1;
        tok
    }

    /// Check if the text token equals the token in function parameter.
    /// If so, then advance the index.
    fn try_next(&mut self, token: LexerToken) -> bool {
        if let Some(current) = self.head() {
            if current == &token {
                self.index += 1;
                return true;
            }
        }
        false
    }

    fn head(&self) -> Option<&LexerToken> {
        self.tokens.get(self.index)
    }

    fn require_expression_body_token(&mut self) -> ParseResult<LexerToken> {
        if let Some(token) = self.next() {
            return match *token {
                LexerToken::Identifier(_)
                | LexerToken::FloatNumberLiteral(_)
                | LexerToken::BoolLiteral(_)
                | LexerToken::NumberLiteral(_)
                | LexerToken::StringLiteral(_)
                | LexerToken::Star
                | LexerToken::ParOpen
                | LexerToken::ParClose
                // | LexerToken::CompareOp(_)
                | LexerToken::Null => Ok(token.clone()),
                _ => Err(ParseError::UnexpectedToken("expression body".into(), token.clone())),
            };
        }

        Err(ParseError::UnexpectedQueryEnding)
    }

    fn require_identifier(&mut self) -> ParseResult<String> {
        if let Some(token) = self.next() {
            return match token.clone() {
                LexerToken::Identifier(id) => Ok(id),
                _ => Err(ParseError::UnexpectedToken(
                    "identifier".into(),
                    token.clone(),
                )),
            };
        }
        Err(ParseError::UnexpectedQueryEnding)
    }

    fn require_datatype(&mut self) -> ParseResult<String> {
        if let Some(token) = self.next() {
            return match token.clone() {
                LexerToken::DataType(datatype) => Ok(datatype),
                _ => Err(ParseError::UnexpectedToken(
                    "data-type".into(),
                    token.clone(),
                )),
            };
        }
        Err(ParseError::UnexpectedQueryEnding)
    }

    fn require_token(&mut self, required: LexerToken) -> ParseResult<()> {
        if let Some(token) = self.next() {
            if *token == required {
                return Ok(());
            }
            return Err(ParseError::UnexpectedToken(
                format!("{:?}", required),
                token.clone(),
            ));
        }

        Err(ParseError::UnexpectedQueryEnding)
    }

    fn require_table_or_index(&mut self) -> ParseResult<LexerToken> {
        if let Some(token) = self.next() {
            if *token == LexerToken::Table || *token == LexerToken::Index {
                return Ok(token.clone());
            }
            return Err(ParseError::UnexpectedToken(
                "table name or identifier".into(),
                token.clone(),
            ));
        }

        Err(ParseError::UnexpectedQueryEnding)
    }

    fn require_eof(&self) -> ParseResult<()> {
        if self.index < self.tokens.len() {
            Err(ParseError::UnexpectedQueryEnding)
        } else {
            Ok(())
        }
    }

    fn parse_query(&mut self) -> ParseResult<Query> {
        let query_type = self.next().ok_or(ParseError::UnexpectedQueryEnding)?;

        let query = match query_type {
            LexerToken::Select => {
                let body = self.parse_query_body()?;
                self.require_token(LexerToken::From)?;
                let table_name = self.require_identifier()?;
                let where_body = self.parse_where_body()?;

                Ok(Query::Select {
                    body,
                    table_name,
                    where_body,
                })
            }
            LexerToken::Insert => {
                self.require_token(LexerToken::Into)?;
                let table_name = self.require_identifier()?;

                let mut columns = Vec::new();

                if self.try_next(LexerToken::ParOpen) {
                    while !self.try_next(LexerToken::ParClose) {
                        columns.push(self.require_identifier()?);
                        self.try_next(LexerToken::Comma);
                    }
                }

                self.require_token(LexerToken::Values)?;
                let is_parenthesised = self.try_next(LexerToken::ParOpen);
                // todo: restrict this to some subset of 'query body' (e.g. star not allowed)
                let mut values = self.parse_query_body()?;
                let last_value = values.last();
                if is_parenthesised {
                    if last_value == Some(&LexerToken::ParClose) {
                        values.pop(); // remove the closing parenthesis from values
                    } else {
                        return Err(ParseError::UnexpectedToken(
                            "closing parenthesis".into(),
                            last_value.unwrap().clone(),
                        ));
                    }
                }

                if !columns.is_empty() && (columns.len() != values.len()) {
                    return Err(ParseError::InsertQueryValuesMismatch);
                }

                Ok(Query::Insert {
                    values,
                    columns,
                    table_name,
                })
            }
            LexerToken::Delete => {
                self.require_token(LexerToken::From)?;
                let table_name = self.require_identifier()?;
                let where_body = self.parse_where_body()?;

                Ok(Query::Delete {
                    table_name,
                    where_body,
                })
            }
            LexerToken::Create => {
                if self.require_table_or_index()? == LexerToken::Table {
                    let table_name = self.require_identifier()?;

                    let is_parenthesised = self.try_next(LexerToken::ParOpen);
                    let columns_definition = self.parse_columns_definition()?;
                    if is_parenthesised {
                        self.require_token(LexerToken::ParClose)?;
                    }
                    Ok(Query::CreateTable {
                        table_name,
                        columns_definition,
                    })
                } else {
                    // index
                    let column_name = self.require_identifier()?;
                    self.require_token(LexerToken::On)?;
                    let table_name = self.require_identifier()?;

                    Ok(Query::CreateIndex {
                        column_name,
                        table_name,
                    })
                }
            }
            LexerToken::Drop => {
                if self.require_table_or_index()? == LexerToken::Table {
                    let table_name = self.require_identifier()?;
                    return Ok(Query::DropTable { table_name });
                } else {
                    // drop index
                    let column_name = self.require_identifier()?;
                    self.require_token(LexerToken::On)?;
                    let table_name = self.require_identifier()?;
                    Ok(Query::DropIndex {
                        column_name,
                        table_name,
                    })
                }
            }
            _ => Err(ParseError::UnexpectedToken(
                "SELECT/INSERT/DELETE".into(),
                query_type.clone(),
            )),
        };

        self.try_next(LexerToken::Semicolon);
        self.require_eof()?;

        query
    }

    fn parse_where_body(&mut self) -> ParseResult<Option<Node>> {
        // where body (the last (optional) part of Query)
        let mut where_body = Vec::new();
        if self.try_next(LexerToken::Where) {
            while let Some(token) = self.next() {
                where_body.push(token.clone());
            }
        }
        parse_tree(where_body)
    }

    fn parse_query_body(&mut self) -> ParseResult<Vec<LexerToken>> {
        let mut body = Vec::new();
        let mut _cont = true;

        while _cont {
            let token = self.require_expression_body_token()?;
            body.push(token);
            self.try_next(LexerToken::Comma); // skip commas (?)
            match self.head() {
                Some(LexerToken::From) | None => _cont = false,
                _ => {}
            }
        }
        Ok(body)
    }

    fn parse_columns_definition(&mut self) -> ParseResult<Vec<(String, String)>> {
        let mut columns = Vec::new();
        let mut _cont = true;
        // comma-separated identifiers
        while _cont {
            let identifier = self.require_identifier()?;
            let datatype = self.require_datatype()?;
            columns.push((identifier, datatype));
            _cont = self.try_next(LexerToken::Comma);
        }
        Ok(columns)
    }

    #[allow(dead_code)]
    fn parse_columns(&mut self) -> ParseResult<Vec<String>> {
        let mut columns = Vec::new();
        let mut _cont = true;
        // comma-separated identifiers
        while _cont {
            let identifier = self.require_identifier()?;
            columns.push(identifier);
            _cont = self.try_next(LexerToken::Comma);
        }
        Ok(columns)
    }
}

pub fn parse(query: &str) -> ParseResult<Query> {
    let tokens = lex(query)?;
    let mut parser = QueryParser::from(tokens);

    parser.parse_query()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select() {
        let expr = "select id, name, lastname from person";
        let expected = Query::Select {
            body: vec![
                LexerToken::Identifier("id".to_string()),
                LexerToken::Identifier("name".to_string()),
                LexerToken::Identifier("lastname".to_string()),
            ],
            table_name: "person".to_string(),
            where_body: None,
        };

        let result = parse(expr).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_select_with_where() {
        let expr = "select * from person where id = 3";
        let expected = Query::Select {
            body: vec![LexerToken::Star],
            table_name: "person".to_string(),
            where_body: Some(Node::new_binary(
                Node::Leaf(LexerToken::Identifier("id".into())),
                LexerToken::CompareOp("=".into()),
                Node::Leaf(LexerToken::NumberLiteral(3)),
            )),
        };

        let result = parse(expr).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_insert() {
        let expr = "insert into mira values 'Mira', 24";
        let expected = Query::Insert {
            values: vec![
                LexerToken::StringLiteral("Mira".to_string()),
                LexerToken::NumberLiteral(24),
            ],
            columns: Vec::new(),
            table_name: "mira".into(),
        };

        let result = parse(expr).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_insert_parentheses() {
        let expr = "insert into mira values ('Mira', 24)";
        let expected = Query::Insert {
            values: vec![
                LexerToken::StringLiteral("Mira".to_string()),
                LexerToken::NumberLiteral(24),
            ],
            columns: Vec::new(),
            table_name: "mira".into(),
        };

        let result = parse(expr).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_insert_selected_columns() {
        let expr = "insert into mira (abc, def, ijk) values ('Mira', 24, 33)";
        let expected = Query::Insert {
            values: vec![
                LexerToken::StringLiteral("Mira".to_string()),
                LexerToken::NumberLiteral(24),
                LexerToken::NumberLiteral(33),
            ],
            columns: vec!["abc".into(), "def".into(), "ijk".into()],
            table_name: "mira".into(),
        };

        let result = parse(expr).unwrap();
        assert_eq!(expected, result);
    }

    // currently, we don't support this
    // #[test]
    // fn test_select_with_expr() {
    //     let expr = "select (app_resets - pda_resets), lastname from person";
    //     let expected =
    //         Query::Select {
    //             body: vec![
    //                 LexerToken::ParOpen,
    //                 LexerToken::Identifier("app_resets".to_string()),
    //                 LexerToken::Minus,
    //                 LexerToken::Identifier("pda_resets".to_string()),
    //                 LexerToken::ParClose,
    //                 LexerToken::Identifier("lastname".to_string())
    //             ],
    //             table_name: "person".to_string(),
    //             where_body: Vec::new() };

    //     let result = parse(expr).unwrap();
    //     assert_eq!(expected, result);
    // }

    #[test]
    fn test_delete() {
        let expr = "delete from table_name where x > 1";
        let expected = Query::Delete {
            table_name: "table_name".to_string(),
            where_body: Some(Node::new_binary(
                Node::Leaf(LexerToken::Identifier("x".into())),
                LexerToken::CompareOp(">".into()),
                Node::Leaf(LexerToken::NumberLiteral(1)),
            )),
        };

        let result = parse(expr).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_create_table() {
        let expr = "create table table_name x int, y varchar, bool_column boolean";
        let expected = Query::CreateTable {
            table_name: "table_name".to_string(),
            columns_definition: vec![
                ("x".to_string(), "int".to_string()),
                ("y".to_string(), "varchar".to_string()),
                ("bool_column".to_string(), "boolean".to_string()),
            ],
        };

        let result = parse(expr).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_create_table_parenthesised() {
        let expr = "create table table_name (x int, y varchar, bool_column boolean)";
        let expected = Query::CreateTable {
            table_name: "table_name".to_string(),
            columns_definition: vec![
                ("x".to_string(), "int".to_string()),
                ("y".to_string(), "varchar".to_string()),
                ("bool_column".to_string(), "boolean".to_string()),
            ],
        };

        let result = parse(expr).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_drop_table() {
        let expr = "drop table table_name";
        let expected = Query::DropTable {
            table_name: "table_name".to_string(),
        };

        let result = parse(expr).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_drop_index() {
        let expr = "drop index column_name on table_name";
        let expected = Query::DropIndex {
            column_name: "column_name".to_string(),
            table_name: "table_name".to_string(),
        };

        let result = parse(expr).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_create_index() {
        let expr = "create index column_name on table_name";
        let expected = Query::CreateIndex {
            column_name: "column_name".to_string(),
            table_name: "table_name".to_string(),
        };

        let result = parse(expr).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_create_index_fails_multiple_columns() {
        let expr = "create index index_name on table_name (column1, column2)";
        let result = parse(expr);
        assert!(result.is_err());
    }
}
