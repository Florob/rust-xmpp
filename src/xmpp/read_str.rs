// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use std::io;
use std::io::{BufStream, BufRead, Read, Write};
use std::str;
use unicode::str::utf8_char_width;

pub trait ReadString {
    fn read_str(&mut self) -> io::Result<String>;
}

impl<S: Read + Write> ReadString for BufStream<S> {
    fn read_str(&mut self) -> io::Result<String> {
        let (result, last) = {
            let available = try!(self.fill_buf());
            let len = available.len();
            let mut last = if len < 3 { 0 } else { len - 3 };
            while last < len {
                let width = utf8_char_width(available[last]);
                if width == 0 {
                    last += 1;
                    continue;
                }
                if last+width <= len {
                    last += width;
                } else {
                    break;
                }
            }
            (str::from_utf8(&available[..last]).unwrap().to_string(), last)
        };
        self.consume(last);

        Ok(result)
    }
}
