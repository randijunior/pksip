use nom::{
    bytes::complete::{take_till, take_while}, character::complete::one_of, sequence::preceded, IResult
};

fn until_eof(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_till(|c| c == 0x0D || c == 0x0A)(input)
}

fn parse_int(input: &[u8]) -> nom::IResult<&[u8], u32> {
    nom::combinator::map_res(nom::character::complete::digit1, |digits: &[u8]| {
        std::str::from_utf8(digits)
            .map_err(|_| "Invalid UTF-8")
            .and_then(|s| s.parse::<u32>().map_err(|_| "Parse error"))
    })(input)
}

pub(crate) fn status_line(i: &[u8]) -> nom::IResult<&[u8], (&[u8], &[u8])> {
    let (input, (_, _, _, _, _, code, _, reason_phrase)) = nom::sequence::tuple((
        nom::bytes::complete::tag("SIP/"),
        take_while(nom::character::is_digit),
        nom::character::complete::char('.'),
        take_while(nom::character::is_digit),
        nom::character::complete::space1,
        nom::character::complete::digit1,
        nom::character::complete::space1,
        until_eof,
    ))(i)?;

    Ok((input, (code, reason_phrase)))
}

// SIP URI: sip:user:password@host:port;uri-parameters?headers
/*
Example SIP and SIPS URIs

   sip:alice@atlanta.com
   sip:alice:secretword@atlanta.com;transport=tcp
   sips:alice@atlanta.com?subject=project%20x&priority=urgent
   sip:+1-212-555-1212:1234@gateway.com;user=phone
   sips:1212@gateway.com
   sip:alice@192.0.2.4
   sip:atlanta.com;method=REGISTER?to=alice%40atlanta.com
   sip:alice;day=tuesday@atlanta.com
*/




pub(crate) fn request_line(i: &[u8]) -> nom::IResult<&[u8], (&[u8], &[u8])> {
    let schema = take_while(|c| c != b':');
    let user = preceded(nom::character::complete::char(':'), take_till(|c| c == b'@'));
    let (input, (schema, user_info, _)) = nom::sequence::tuple((
        schema,
        user,
        nom::character::complete::char('@')
    ))(i)?;

    

    Ok((input, (schema, user_info)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_line_parser() {
        let (input, (schema, user_info)) = request_line("sip:alice;day=tuesday@atlanta.com".as_bytes()).unwrap();

        println!("INPUT {:#?}", std::str::from_utf8(input).unwrap());

        println!("SCHEMA {:#?}", std::str::from_utf8(schema).unwrap());
        println!("USER INFO {:#?}", std::str::from_utf8(user_info).unwrap());

        // assert_eq!(schema, "sip".as_bytes());
        // assert_eq!(user_info, "alice".as_bytes());
    }
}
