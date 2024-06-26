use nom::error::{ErrorKind, ParseError};
use std::str;

#[derive(Debug)]
pub struct ParseSipError<'err> {
    code: u32,
    message: &'err str
}

impl<'err> ParseSipError<'err> {
    pub fn new(code: u32, message: &'err str) -> Self {
        ParseSipError { code, message }
    }
}


impl<'err> ParseError<&'err str> for ParseSipError<'err> {
    fn from_error_kind(input: &'err str, kind: ErrorKind) -> Self {
        ParseSipError { code: kind as u32, message: input }
    }

    fn append(input: &'err str, kind: ErrorKind, _other: Self) -> Self {
        ParseSipError { code: kind as u32, message: input }
    }
}

impl<'err> ParseError<&'err [u8]> for ParseSipError<'err> {
    fn from_error_kind(input: &'err [u8], kind: ErrorKind) -> Self {
        let message = match str::from_utf8(input) {
            Ok(err_msg) =>  err_msg,    
            Err(_) => "Parser error: invalid utf-8 string",
        };

        let msg = format!("{:?}:\t{:?}\n", kind, input);
        println!("{}", msg);
        
        ParseSipError { code: kind as u32, message }
        
    }

    fn append(input: &'err [u8], kind: ErrorKind, _other: Self) -> Self {
        let message = match str::from_utf8(input) {
            Ok(err_msg) =>  err_msg,
            Err(_) => "Parser error: invalid utf-8 string",
        };
        ParseSipError { code: kind as u32, message }
    }
}