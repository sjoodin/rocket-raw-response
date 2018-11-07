//! # Raw Response for Rocket Framework
//! This crate provides a response struct used for responding raw data.

extern crate mime_guess;
extern crate rocket;

use std::io::{self, Read, ErrorKind, Cursor};
use std::fs::{self, File};
use std::path::Path;

use mime_guess::get_mime_type_str;

use rocket::response::{self, Response, Responder};
use rocket::request::Request;

const RESPONSE_CHUNK_SIZE: u64 = 4096;

/// The response struct used for responding raw data.
pub struct RawResponse {
    pub data: Box<Read>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
}

impl<'a> Responder<'a> for RawResponse {
    fn respond_to(self, _: &Request) -> response::Result<'a> {
        let mut response = Response::build();

        if let Some(content_type) = self.content_type {
            response.raw_header("Content-Type", content_type);
        }

        if let Some(content_length) = self.content_length {
            response.raw_header("Content-Length", content_length.to_string());
        }

        response.chunked_body(self.data, RESPONSE_CHUNK_SIZE);

        response.ok()
    }
}

impl RawResponse {
    /// Create a RawResponse instance from a path of a file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<RawResponse> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(io::Error::from(ErrorKind::NotFound));
        }

        if !path.is_file() {
            return Err(io::Error::from(ErrorKind::InvalidInput));
        }

        let file_size = match fs::metadata(&path) {
            Ok(metadata) => {
                Some(metadata.len())
            }
            Err(e) => return Err(e)
        };

        let content_type = match path.extension() {
            Some(extension) => {
                get_mime_type_str(&extension.to_str().unwrap().to_lowercase()).map(|t| { String::from(t) })
            }
            None => None
        };

        let data = Box::from(File::open(&path)?);

        Ok(RawResponse {
            data,
            content_type,
            content_length: file_size,
        })
    }

    /// Create a RawResponse instance from a Vec<u8> instance.
    pub fn from_bytes<S: AsRef<str>>(data: Vec<u8>, content_type: Option<S>) -> io::Result<RawResponse> {
        let content_length = data.len();

        let content_type = match content_type {
            Some(s) => Some(s.as_ref().to_string()),
            None => None
        };

        let data = Box::from(Cursor::new(data));

        Ok(RawResponse {
            data,
            content_type,
            content_length: Some(content_length as u64),
        })
    }
}