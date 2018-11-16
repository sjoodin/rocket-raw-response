/*!
# Raw Response for Rocket Framework

This crate provides a response struct used for responding raw data.

See `examples`.
*/

pub extern crate mime;
extern crate mime_guess;
extern crate percent_encoding;
extern crate rocket;

use std::io::{self, Read, ErrorKind, Cursor};
use std::fs::{self, File};
use std::path::Path;
use std::fmt::{self, Debug, Formatter};

use mime::Mime;

use rocket::response::{Response, Responder, Result};
use rocket::request::Request;

/// The response struct used for responding raw data.
pub struct RawResponse<'a> {
    pub data: Box<Read + 'a>,
    pub file_name: String,
    pub content_type: Option<Mime>,
    pub content_length: Option<u64>,
}

impl<'a> Debug for RawResponse<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!("RawResponse {{file_name: {:?}, content_type: {:?}, content_length: {:?}}}", self.file_name, self.content_type, self.content_length))
    }
}

impl<'a> Responder<'a> for RawResponse<'a> {
    fn respond_to(self, _: &Request) -> Result<'a> {
        let mut response = Response::build();

        if !self.file_name.is_empty() {
            response.raw_header("Content-Disposition", format!("inline; filename*=UTF-8''{}", percent_encoding::percent_encode(self.file_name.as_bytes(), percent_encoding::QUERY_ENCODE_SET)));
        }

        if let Some(content_type) = self.content_type {
            response.raw_header("Content-Type", content_type.to_string());
        }

        if let Some(content_length) = self.content_length {
            response.raw_header("Content-Length", content_length.to_string());
        }

        response.streamed_body(self.data);

        response.ok()
    }
}

impl<'a> RawResponse<'a> {
    /// Create a `RawResponse` instance from a path of a file.
    pub fn from_file<P: AsRef<Path>, S: Into<String>>(path: P, file_name: Option<S>, content_type: Option<Mime>) -> io::Result<RawResponse<'static>> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(io::Error::from(ErrorKind::NotFound));
        }

        if !path.is_file() {
            return Err(io::Error::from(ErrorKind::InvalidInput));
        }

        let file_name = match file_name {
            Some(file_name) => {
                let file_name = file_name.into();
                file_name
            }
            None => {
                path.file_name().unwrap().to_str().unwrap().to_string()
            }
        };

        let file_size = match fs::metadata(&path) {
            Ok(metadata) => {
                Some(metadata.len())
            }
            Err(e) => return Err(e)
        };

        let content_type = match content_type {
            Some(content_type) => content_type,
            None => match path.extension() {
                Some(extension) => {
                    mime_guess::get_mime_type(extension.to_str().unwrap())
                }
                None => mime::APPLICATION_OCTET_STREAM
            }
        };

        let data = Box::from(File::open(&path)?);

        Ok(RawResponse {
            data,
            file_name,
            content_type: Some(content_type),
            content_length: file_size,
        })
    }

    /// Create a `RawResponse` instance from a Vec<u8>.
    pub fn from_vec<S: Into<String>>(vec: Vec<u8>, file_name: S, content_type: Option<Mime>) -> RawResponse<'static> {
        let file_name = file_name.into();

        let content_length = vec.len();

        RawResponse {
            data: Box::from(Cursor::new(vec)),
            file_name,
            content_type,
            content_length: Some(content_length as u64),
        }
    }

    /// Create a `RawResponse` instance from a reader.
    pub fn from_reader<R: Read + 'a, S: Into<String>>(reader: R, file_name: S, content_type: Option<Mime>, content_length: Option<u64>) -> RawResponse<'a> {
        let file_name = file_name.into();

        RawResponse {
            data: Box::from(reader),
            file_name,
            content_type,
            content_length,
        }
    }
}