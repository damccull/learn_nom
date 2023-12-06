use nom::{error::ParseError, IResult, Parser};

fn main() -> Result<(), anyhow::Error> {
    let msg = "Hello, world!";
    dbg!(parse_hello(msg)?);
    dbg!(parse_tag("Hello").parse(msg)?);
    dbg!(parse_comma_tags("Hello", "world").parse(msg)?);
    dbg!(parse_separated(parse_tag("Hello"), parse_tag(", "), parse_tag("world")).parse(msg)?);
    dbg!(parse_bool("true, 12345")?);
    dbg!(parse_either(parse_tag("true"), parse_tag("false")).parse("false, 54321")?);
    Ok(())
}

fn parse_separated<I, O1, S, O2, E>(
    mut parse_tag1: impl Parser<I, O1, E>,
    mut parse_separator: impl Parser<I, S, E>,
    mut parse_tag2: impl Parser<I, O2, E>,
) -> impl Parser<I, (O1, O2), E> {
    move |input: I| {
        let (tail, value1) = parse_tag1.parse(input)?;
        let (tail, _) = parse_separator.parse(tail)?;
        let (tail, value2) = parse_tag2.parse(tail)?;

        Ok((tail, (value1, value2)))
    }
}

fn parse_bool(input: &str) -> IResult<&str, bool, ()> {
    match parse_tag("true").parse(input) {
        Ok((tail, _)) => Ok((tail, true)),
        Err(nom::Err::Error(_err)) => match parse_tag("false").parse(input) {
            Ok((tail, _)) => Ok((tail, false)),
            Err(err) => Err(err),
        },
        Err(err) => Err(err),
    }
}

fn parse_either<I: Clone, O, E: ParseError<I>>(
    mut choice1: impl Parser<I, O, E>,
    mut choice2: impl Parser<I, O, E>,
) -> impl Parser<I, O, E> {
    move |input: I| match choice1.parse(input.clone()) {
        Ok((tail, value)) => Ok((tail, value)),
        Err(nom::Err::Error(err1)) => match choice2.parse(input) {
            Ok((tail, value)) => Ok((tail, value)),
            Err(nom::Err::Error(err2)) => Err(nom::Err::Error(err1.or(err2))),
            Err(err) => Err(err),
        },
        Err(err) => Err(err),
    }
}

fn parse_comma_tags<'i: 't, 't>(
    tag1: &'t str,
    tag2: &'t str,
) -> impl Parser<&'i str, (&'i str, &'i str), ()> + 't {
    let mut parse_tag1 = parse_tag(tag1);
    let mut parse_separator = parse_tag(", ");
    let mut parse_tag2 = parse_tag(tag2);
    move |input: &'i str| {
        let (tail, value1) = parse_tag1.parse(input)?;
        let (tail, _) = parse_separator.parse(tail)?;
        let (tail, value2) = parse_tag2.parse(tail)?;

        Ok((tail, (value1, value2)))
    }
}

fn parse_tag<'i: 't, 't>(tag: &'t str) -> impl Parser<&'i str, &'i str, ()> + 't {
    move |input: &'i str| match input.strip_prefix(tag) {
        Some(tail) => Ok((tail, &input[..tag.len()])),
        None => Err(nom::Err::Error(())),
    }
}

fn parse_hello(input: &str) -> IResult<&str, &str, ()> {
    match input.strip_prefix("Hello") {
        Some(tail) => Ok((tail, "Hello")),
        None => Err(nom::Err::Error(())),
    }
}

#[cfg(test)]
mod tests {
    use nom::Parser;

    use crate::{
        parse_bool, parse_comma_tags, parse_either, parse_hello, parse_separated, parse_tag,
    };

    #[test]
    fn parse_either_works_for_valid_input() {
        let mut parse_bool_str = parse_either(parse_tag("true"), parse_tag("false"));
        assert_eq!(
            parse_bool_str.parse("true, 12345").unwrap(),
            (", 12345", "true")
        );
        assert_eq!(
            parse_bool_str.parse("false, 12345").unwrap(),
            (", 12345", "false")
        );
    }

    #[test]
    fn parse_either_fails_for_invalid_input() {
        let mut parse_bool_str = parse_either(parse_tag("true"), parse_tag("false"));
        assert_eq!(parse_bool_str.parse("borked"), Err(nom::Err::Error(())));
    }

    #[test]
    fn parse_bool_works_for_valid_input() {
        assert_eq!(parse_bool("true, 12345").unwrap(), (", 12345", true));
        assert_eq!(parse_bool("false, 12345").unwrap(), (", 12345", false));
    }
    #[test]
    fn parse_separated_works_for_valid_input() {
        let mut parse_hello_world =
            parse_separated(parse_tag("Hello"), parse_tag(", "), parse_tag("world"));
        assert_eq!(
            parse_hello_world.parse("Hello, world!").unwrap(),
            ("!", ("Hello", "world"))
        );
    }

    #[test]
    fn parse_comma_tags_works_for_valid_input() {
        assert_eq!(
            parse_comma_tags("Hello", "world")
                .parse("Hello, world!")
                .unwrap(),
            ("!", ("Hello", "world"))
        );
    }

    #[test]
    fn parse_tag_workds_for_valid_input() {
        assert_eq!(
            parse_tag("Hello").parse("Hello, world!").unwrap(),
            (", world!", "Hello")
        );
    }

    #[test]
    fn parse_hello_works_for_valid_input() {
        assert_eq!(parse_hello("Hello, world!").unwrap(), (", world!", "Hello"));
    }
}
