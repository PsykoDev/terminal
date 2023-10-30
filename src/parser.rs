#[derive(Debug)]
pub enum ParseError<'a> {
    Empty,
    InvalidChar(char),
    InvalidSequence(&'a str),
}

fn any_char(input: &str) -> Result<(&str, char), ParseError<'_>> {
    match input.chars().next() {
        None => Err(ParseError::Empty),
        Some(c) => Ok((&input[c.len_utf8()..], c)),
    }
}

fn satisfy<F>(f: F, input: &str) -> Result<(&str, char), ParseError<'_>>
    where
        F: FnOnce(char) -> bool,
{
    match input.chars().next() {
        None => Err(ParseError::Empty),
        Some(c) => {
            if f(c) {
                Ok((&input[c.len_utf8()..], c))
            } else {
                Err(ParseError::InvalidChar(c))
            }
        }
    }
}

fn take_while<F>(f: F, input: &str) -> Result<(&str, &str), ParseError<'_>>
    where
        F: Fn(char) -> bool,
{
    let mut index = 0;
    loop {
        match any_char(&input[index..]) {
            Err(_e) => break Ok((&input[index..], &input[..index])),
            Ok((_rest, c)) => {
                if f(c) {
                    index += c.len_utf8();
                } else {
                    break Ok((&input[index..], &input[..index]));
                }
            }
        }
    }
}

fn skip_ws(input: &str) -> Result<(&str, ()), ParseError<'_>> {
    let (rest, _) = take_while(|c| c.is_whitespace(), input)?;
    Ok((rest, ()))
}

fn parse_i32(input: &str) -> Result<(&str, i32), ParseError<'_>> {
    let (rest, s) = take_while(|c| c.is_digit(10), input)?;
    let n = s
        .parse::<i32>()
        .map_err(|_e| ParseError::InvalidSequence(s))?;
    Ok((rest, n))
}

// fn parse_bool(input: &str) -> Result<(&str, bool), ParseError<'_>> {
//     let (rest, s) = take_while(|c| c.is_alphabetic(), input)?;
//     match s {
//         "true" => Ok((rest, true)),
//         "false" => Ok((rest, false)),
//         _ => Err(ParseError::InvalidSequence(s)),
//     }
// }

#[derive(Debug)]
pub(crate) enum Expr {
    Int(i32),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Var(String),
}


pub fn parse_expr(input: &str) -> Result<(&'_ str, Expr), ParseError<'_>> {

    let (rest, _) = skip_ws(&input)?;

    let (mut i, lhs) = parse_term(rest)?;
    let mut v = lhs;

    loop {
        let (rest, _) = skip_ws(i)?;
        let (rest, operator) = match satisfy(|c| c == '+' || c == '-', rest) {
            Ok(x) => x,
            Err(_) => break Ok((i, v)),
        };
        let (rest, rhs) = parse_term(rest)?;
        let r = rhs;

        match operator {
            '+' => v = Expr::Add(Box::from(v), Box::from(r)),
            '-' => v = Expr::Sub(Box::from(v), Box::from(r)),
            _ => unreachable!(),
        };
        i = rest;
    }
}

fn parse_term(input: &str) -> Result<(&str, Expr), ParseError<'_>> {
    let (rest, lhs) = parse_factor(input)?;
    let mut i = rest;
    let mut v = lhs;

    loop {
        let (rest, _) = skip_ws(i)?;
        let (rest, operator) = match satisfy(|c| c == '*' || c == '/', rest) {
            Ok(x) => x,
            Err(_) => break Ok((i, v)),
        };
        let (rest, rhs) = parse_factor(rest)?;
        let r = rhs;

        match operator {
            '*' => v = Expr::Mul(Box::from(v), Box::from(r)),
            '/' => v = Expr::Div(Box::from(v), Box::from(r)),
            _ => unreachable!(),
        };

        i = rest;
    }
}

fn parse_factor(input: &str) -> Result<(&str, Expr), ParseError<'_>> {
    let (rest, _) = skip_ws(input)?;

    if let Ok((rest, num)) = parse_i32(rest) {
        return Ok((rest, Expr::Int(num)));
    }

    if let Ok((rest, name)) = parse_name(input) {
        return Ok((rest, Expr::Var(name)));
    }

    let (rest, _) = satisfy(|c| c == '(', rest)?;
    let (rest, _) = skip_ws(rest)?;
    let (rest, expr) = parse_expr(rest)?;
    let (rest, _) = skip_ws(rest)?;
    let (rest, _) = satisfy(|c| c == ')', rest)?;
    let (rest, _) = skip_ws(rest)?;

    Ok((rest, expr))
}

fn parse_name(input: &str) -> Result<(&str, String), ParseError<'_>> {
    let (rest, _) = skip_ws(input)?;
    let (rest, first_char) = satisfy(|c| c.is_ascii_alphabetic() || c == '_', rest)?;
    let (rest, _) = skip_ws(rest)?;
    let (rest, name_chars) = take_while(|c| c.is_ascii_alphanumeric() || c == '_', rest)?;
    let (rest, _) = skip_ws(rest)?;
    let name = format!("{}{}", first_char, name_chars);
    Ok((rest, name))
}

#[derive(Debug)]
pub(crate) enum Statement {
    Assign(String, Expr),
    Expr(Expr),
}

pub fn parse_statement(input: &str) -> Result<(&str, Statement), ParseError<'_>> {
    let (rest, _) = skip_ws(input)?;

    if let Ok((rest, name)) = parse_name(rest) {
        let (rest, _) = skip_ws(rest)?;

        if let Ok((rest, operator)) = satisfy(|c| c == '=' , rest) {
            let (rest, _) = skip_ws(rest)?;
            let (rest,expr) = parse_expr(rest)?;
            let (rest, _) = skip_ws(rest)?;

            let statement = match operator {
                '=' => Statement::Assign(name, expr),
                _ => return Err(ParseError::InvalidChar(operator)),
            };
            return Ok((rest, statement));
        }
    }
    let (rest, expr) = parse_expr(rest)?;
    let (rest, _) = skip_ws(rest)?;
    Ok((rest, Statement::Expr(expr)))
}