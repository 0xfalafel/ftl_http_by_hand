use nom::{
    bytes::streaming::{tag, take_until}, character::complete::{space0, space1, u16}, combinator::{map_res, opt}, sequence::terminated, IResult, Parser
};

#[derive(Debug)]
#[allow(unused)]
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
    // HTTP version header
    let (i, _) = tag("HTTP/1.1")(i)?;
    let (i, _) = space1(i)?;
    
    // Read the status code
    let (i, status_code) = u16(i)?;
    let (i, _) = space1(i)?;
    
    // Read the status text
    let (i, status_text) = map_res(
        take_until(CRLF),
        |bytes| str::from_utf8(bytes)
    ).parse(i)?;
    let (i, _) = tag(CRLF)(i)?;
    
    let mut res = Response {
        status: status_code,
        status_text: status_text,
        headers: Vec::default(),
    };

    let mut i = i;
    loop {
        if let (i, Some(_)) = opt(tag(CRLF)).parse(i)? {
            println!("We are done !");
            return Ok((i, res))
        }

        let (i2, (name, value)) = header(i)?;
        res.headers.push((name, value));
        i = i2;
    }
}

fn header(i: &[u8]) -> IResult<&[u8], (&str, &str)> {
    // parse the header key
    let (i, name) = map_res(
        terminated(take_until(":"), tag(":")),
        str::from_utf8
    ).parse(i)?;

    let (i, _) = space0(i)?;

    // parse the header value
    let (i, value) = map_res(
        terminated(take_until(CRLF), tag(CRLF)),
        str::from_utf8
    ).parse(i)?;

    Ok((i, (name, value)))
}