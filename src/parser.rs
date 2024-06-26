use nom::{
    bytes::complete::{tag, take, take_till, take_while},
    character::{
        complete::{digit1, space1},
        is_digit,
    },
    combinator::map_res,
    sequence::{preceded, tuple},
    IResult,
};

use nom::character::complete::char;

use crate::util::ParseSipError;

const EMPTY: &[u8] = &[];

pub trait Parser<'a> {
    fn parse(input: &'a [u8]) -> Result<Self, nom::Err<ParseSipError>> where Self: Sized;
}

fn until_eof(input: &[u8]) -> IResult<&[u8], &[u8], ParseSipError> {
    take_till(|c| c == 0x0D || c == 0x0A)(input)
}

fn parse_int(input: &[u8]) -> nom::IResult<&[u8], u32> {
    map_res(digit1, |digits: &[u8]| {
        std::str::from_utf8(digits)
            .map_err(|_| "Invalid UTF-8")
            .and_then(|s| s.parse::<u32>().map_err(|_| "Parse error"))
    })(input)
}

pub(crate) fn status_line(i: &[u8]) -> nom::IResult<&[u8], (&[u8], &[u8]), ParseSipError> {
    let (input, (_, _, _, _, _, code, _, reason_phrase)) = tuple((
        tag("SIP/"),
        take_while(is_digit),
        char('.'),
        take_while(is_digit),
        space1,
        digit1,
        space1,
        until_eof,
    ))(i)?;

    Ok((input, (code, reason_phrase)))
}

fn uri_params(i: &[u8]) -> IResult<&[u8], &[u8], ParseSipError> {
    if i.len() == 0 || i[0] != b';' {
        Ok((i, EMPTY))
    } else {
        preceded(tag(";"), take_till(|c| c == b'?' || c == b' '))(i)
    }
}

fn uri_headers(i: &[u8]) -> IResult<&[u8], &[u8], ParseSipError> {
    if i.len() == 0 || i[0] != b'?' {
        Ok((i, EMPTY))
    } else {
        preceded(tag("?"), take_till(|c| c == b' '))(i)
    }
}

fn uri_host(i: &[u8]) -> IResult<&[u8], &[u8], ParseSipError> {
    take_till(|c| c == b';' || c == b'?' || c == b' ')(i)
}

fn rl_after_ampersat(i: &[u8]) -> nom::IResult<&[u8], (&[u8], &[u8], &[u8]), ParseSipError> {
    let (input, host) = uri_host(i)?;
    let (input, uri_params) = uri_params(input)?;
    let (input, uri_headers) = uri_headers(input)?;

    Ok((input, (host, uri_params, uri_headers)))
}

pub(crate) fn request_line(i: &[u8]) -> nom::IResult<&[u8], (&[u8], &[u8], &[u8], &[u8], &[u8]), ParseSipError> {
    let scheme = take_while(|c| c != b':');
    let user = preceded(char(':'), take_till(|c| c == b'@'));
    let (input, (schema, after_schema)) = tuple((scheme, user))(i)?;

    if input.is_empty() {
        // No user info
        let (input, (host, uri_params, uri_headers)) = rl_after_ampersat(after_schema)?;
        return Ok((input, (schema, EMPTY, host, uri_params, uri_headers)));
    }

    // remove the '@'
    let (input, _) = take(1usize)(input)?;
    let (input, (host, uri_params, uri_headers)) = rl_after_ampersat(input)?;

    return Ok((input, (schema, after_schema, host, uri_params, uri_headers)));
}