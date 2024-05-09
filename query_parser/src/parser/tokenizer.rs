use crate::parser::errors::ParseError;

use super::errors::ParseResult;

fn is_allowed_identifier_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch.is_ascii_punctuation()
}

pub fn tokenize(input: &str) -> ParseResult<Vec<&str>> {
    let mut tokens = Vec::new();

    enum State {
        //                example: SELECT * FROM table WHERE some_column = "hello \"world\"";
        Normal,                 // ^---------------------------------------^                ^
        StrLit,                 // iterator is inside the string literal    ^--------------^
        StrLitEscapedChar,      // current char is to be escaped                   ^      ^
        DoubleCharSizeOperator, // current char is second char of double char size operator - >=, <=, <>, !=
    }
    let mut state = State::Normal;
    let mut token_start_i: usize = 0;
    // fixme: rename to mark that it means 'current end boundary'
    let mut token_current_i: usize = 0;

    for (char_pos, char) in input.chars().enumerate() {
        token_current_i += char.len_utf8();
        match state {
            // basically outside str_literal
            State::Normal => {
                match char {
                    '(' | ')' | ' ' | ',' | ';' | '=' | '+' | '-' => {
                        // end the current token
                        tokens.push(&input[token_start_i..token_current_i - 1]);
                        // add the separator as a separate token
                        if char != ' ' {
                            tokens.push(&input[token_current_i - 1..token_current_i]);
                        }

                        token_start_i = token_current_i;
                    }
                    '"' | '\u{0027}' => state = State::StrLit,
                    '\\' => {
                        return Err(ParseError::InvalidChar('\\', char_pos));
                    }
                    '>' | '<' | '!' => {
                        if let Some(next_char) = input[token_current_i..].chars().next() {
                            match format!("{char}{next_char}").as_str() {
                                "<=" | ">=" | "<>" | "!=" => {
                                    state = State::DoubleCharSizeOperator;
                                }
                                _ => {
                                    // end the current token
                                    tokens.push(&input[token_start_i..token_current_i - 1]);
                                    // add the separator as a separate token
                                    tokens.push(&input[token_current_i - 1..token_current_i]);
                                    token_start_i = token_current_i;
                                }
                            }
                        }
                    }
                    _ => {
                        if !is_allowed_identifier_char(char) {
                            return Err(ParseError::InvalidChar(char, char_pos));
                        }
                    }
                }
            }
            State::StrLit => {
                match char {
                    // escape symbol
                    '\\' => state = State::StrLitEscapedChar,
                    // unescaped " char, end the string_literal state and add a token
                    '"' | '\u{0027}' => {
                        state = State::Normal;
                        tokens.push(&input[token_start_i..token_current_i]);
                        token_start_i = token_current_i;

                        // check that the next char is either of [' ', ',', ';', '=']
                        // e.g. "hello"-1 is invalid
                        if let Some(next_char) = input[token_current_i..].chars().next() {
                            if ![' ', ',', ';', '=', ')'].contains(&next_char) {
                                return Err(ParseError::InvalidChar(next_char, char_pos + 1));
                            }
                        }
                    }
                    _ => {}
                }
            }
            State::StrLitEscapedChar => {
                state = State::StrLit;
            }
            State::DoubleCharSizeOperator => {
                // end the current token
                tokens.push(&input[token_start_i..token_current_i - 2]);
                // add the double size operator as a separate token
                tokens.push(&input[token_current_i - 2..token_current_i]);
                token_start_i = token_current_i;
                state = State::Normal;
            }
        }

        // end of the input
        if token_current_i == input.len() {
            match state {
                State::Normal => tokens.push(&input[token_start_i..token_current_i]),
                _ => {
                    return Err(ParseError::UnfinishedStringLiteral(
                        input[token_start_i..token_current_i].to_string(),
                    ))
                }
            }
        }
    }

    Ok(tokens.into_iter().filter(|&tok| !tok.is_empty()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        assert_eq!(
            vec!["select", "2", "from", "table"],
            tokenize(stringify!(select 2 from table)).unwrap()
        );

        assert_eq!(
            vec!["select", "2", ",", "3", ","],
            tokenize(stringify!(select 2, 3,)).unwrap()
        );

        assert_eq!(
            vec!["select", "\"ahoj\""],
            tokenize(stringify!(select "ahoj")).unwrap()
        );

        assert_eq!(
            vec!["insert", "\"😎\"", ",", "2", "into", "my_table"],
            tokenize(stringify!(insert "😎", 2 into my_table)).unwrap()
        );

        assert_eq!(
            // the escape character itself should be also included here, it will be stripped away in lexer
            // .. or won't be?
            vec!["select", "\"ahoj\\\"\""],
            tokenize(stringify!(select "ahoj\"")).unwrap()
        );
    }

    #[test]
    fn test_tokenize_errors() {
        assert!(tokenize("insert \"").is_err());
    }

    #[test]
    fn test_delete() {
        assert_eq!(
            vec!["delete", "from", "my_table"],
            tokenize(stringify!(delete from my_table)).unwrap()
        );

        assert_eq!(
            vec!["delete", "from", "my_table", "where", "x", "=", "40.0"],
            tokenize(stringify!(delete from my_table where x = 40.0)).unwrap()
        );
    }

    #[test]
    fn test_plus_minus() {
        assert_eq!(
            vec!["select", "x", "-", "a", "from", "my_table"],
            tokenize(stringify!(select x - a from my_table)).unwrap()
        );

        assert_eq!(
            vec!["select", "x", "+", "a", "as", "res", "from", "my_table"],
            tokenize(stringify!(select x + a as res from my_table)).unwrap()
        );

        assert_eq!(vec!["where", "x", "+", "4"], tokenize("where x+4").unwrap());

        assert_eq!(
            vec!["where", "x", "=", "-", "4"],
            tokenize("where x=-4").unwrap()
        );

        assert_eq!(
            vec!["where", "x", "-", "4", "=", "5"],
            tokenize("where x-4=5").unwrap()
        );
    }

    #[test]
    fn test_multiple_spaces() {
        assert_eq!(
            vec!["select", "ahoj"],
            tokenize(stringify!(select      ahoj)).unwrap()
        );

        assert_eq!(
            vec!["select", "ahoj"],
            tokenize("select      ahoj").unwrap()
        );
    }

    #[test]
    fn test_parenthesis() {
        assert_eq!(
            vec!["where", "(", "x", "=", "-", "4", ")"],
            tokenize("where (x = -4)").unwrap()
        );
    }

    #[test]
    fn test_tokenize_without_spaces() {
        assert_eq!(vec!["where", "x", "=", "4"], tokenize("where x=4").unwrap());

        assert_eq!(
            vec!["where", "x", ">=", "4"],
            tokenize("where x>=4").unwrap()
        );

        assert_eq!(
            vec!["where", "x", ">=", ">", "44"],
            tokenize("where x >=> 44").unwrap()
        );

        assert_eq!(
            vec!["where", "x", "!", ">", "44"],
            tokenize("where x !> 44").unwrap()
        );

        assert_eq!(
            vec!["where", "x", "<>", "44"],
            tokenize("where x<>44").unwrap()
        );
    }

    #[test]
    fn test_exclamation_mark() {
        assert_eq!(vec!["!", "abc"], tokenize("!abc").unwrap());
        assert_eq!(vec!["!", "abc"], tokenize("! abc").unwrap());
    }

    // TODO: what about combining quotes?
    // #[test]
    // fn test_string_literals_combined() {
    //     assert_eq!(
    //         vec!["select", "\"ahoj\"", "\"zdar'\""],
    //         tokenize(stringify!(select "ahoj" "zdar'")).unwrap()
    //     );
    // }

    #[test]
    fn test_insert() {
        let expr = "INSERT INTO films VALUES ('UA502', 'Bananas', 105, '1971-07-13', 'Comedy', '82 minutes')";
        let expected = vec![
            "INSERT",
            "INTO",
            "films",
            "VALUES",
            "(",
            "'UA502'",
            ",",
            "'Bananas'",
            ",",
            "105",
            ",",
            "'1971-07-13'",
            ",",
            "'Comedy'",
            ",",
            "'82 minutes'",
            ")",
        ];

        assert_eq!(expected, tokenize(expr).unwrap());
    }

    #[test]
    fn test_insert_specified_columns() {
        let expr = "INSERT INTO films (code, title, did, date_prod, kind) VALUES ('T_601', 'Yojimbo', 106, '1961-06-16', 'Drama');";
        let expected = vec![
            "INSERT",
            "INTO",
            "films",
            "(",
            "code",
            ",",
            "title",
            ",",
            "did",
            ",",
            "date_prod",
            ",",
            "kind",
            ")",
            "VALUES",
            "(",
            "'T_601'",
            ",",
            "'Yojimbo'",
            ",",
            "106",
            ",",
            "'1961-06-16'",
            ",",
            "'Drama'",
            ")",
            ";",
        ];

        assert_eq!(expected, tokenize(expr).unwrap());
    }
}
