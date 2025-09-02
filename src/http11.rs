use nom::{
    bytes::streaming::{tag, take_until, take_while1},
    IResult,
};

#[derive(Debug)]
pub struct Response<'a> {
    pub status: u16,
    pub status_text: &'a str,

    // header/name values could be non-UTF-8, but let's not care for this sample.
    // we are however careful to not use a HashMap since headers can repeat.
    pub headers: Vec<(&'a str, &'a str)>
}

const CRLF: &str = "\r\n";

// Looks like `HTTP/1.1 200 OK\r\n` or `HTTP/1.1 404 Not Found\r\n`
pub fn response(i: &[u8]) -> IResult<&[u8], Response<'_>> {
    let (i, _) = tag("HTTP/1.1 ")(i)?;

    
}