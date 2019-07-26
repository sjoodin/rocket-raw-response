/*!
# Raw Response for Rocket Framework

This crate provides a response struct used for responding raw data.

See `examples`.
*/

pub extern crate mime;
extern crate mime_guess;
extern crate percent_encoding;
extern crate rocket;
#[macro_use]
extern crate educe;

use std::io::{Read, Cursor, ErrorKind};
use std::fs::File;
use std::path::Path;
use std::rc::Rc;

use mime::Mime;

use rocket::response::{self, Response, Responder};
use rocket::request::Request;
use rocket::http::Status;

#[derive(Educe)]
#[educe(Debug)]
enum RawResponseData {
    Vec(Vec<u8>),
    Reader {
        #[educe(Debug(ignore))]
        data: Box<dyn Read + 'static>,
        content_length: Option<u64>,
    },
    File(Rc<Path>),
}

#[derive(Debug)]
pub struct RawResponse {
    file_name: Option<String>,
    content_type: Option<Mime>,
    data: RawResponseData,
}

impl RawResponse {
    /// Create a `RawResponse` instance from a `Vec<u8>`.
    pub fn from_vec<S: Into<String>>(vec: Vec<u8>, file_name: Option<S>, content_type: Option<Mime>) -> RawResponse {
        let file_name = file_name.map(|file_name| file_name.into());

        let data = RawResponseData::Vec(vec);

        RawResponse {
            file_name,
            content_type,
            data,
        }
    }

    /// Create a `RawResponse` instance from a reader.
    pub fn from_reader<R: Read + 'static, S: Into<String>>(reader: R, file_name: Option<S>, content_type: Option<Mime>, content_length: Option<u64>) -> RawResponse {
        let file_name = file_name.map(|file_name| file_name.into());

        let data = RawResponseData::Reader {
            data: Box::new(reader),
            content_length,
        };

        RawResponse {
            file_name,
            content_type,
            data,
        }
    }

    /// Create a `RawResponse` instance from a path of a file.
    pub fn from_file<P: Into<Rc<Path>>, S: Into<String>>(path: P, file_name: Option<S>, content_type: Option<Mime>) -> RawResponse {
        let path = path.into();
        let file_name = file_name.map(|file_name| file_name.into());

        let data = RawResponseData::File(path);

        RawResponse {
            file_name,
            content_type,
            data,
        }
    }
}

macro_rules! file_name {
    ($s:expr, $res:expr) => {
        if let Some(file_name) = $s.file_name {
            if !file_name.is_empty() {
                $res.raw_header("Content-Disposition", format!("inline; filename*=UTF-8''{}", percent_encoding::percent_encode(file_name.as_bytes(), percent_encoding::QUERY_ENCODE_SET)));
            }
        }
    };
}

macro_rules! content_type {
    ($s:expr, $res:expr) => {
        if let Some(content_type) = $s.content_type {
            $res.raw_header("Content-Type", content_type.to_string());
        }
    };
}

impl<'a> Responder<'a> for RawResponse {
    fn respond_to(self, _: &Request) -> response::Result<'a> {
        let mut response = Response::build();

        match self.data {
            RawResponseData::Vec(data) => {
                file_name!(self, response);
                content_type!(self, response);

                response.sized_body(Cursor::new(data));
            }
            RawResponseData::Reader { data, content_length } => {
                file_name!(self, response);
                content_type!(self, response);

                if let Some(content_length) = content_length {
                    response.raw_header("Content-Length", content_length.to_string());
                }

                response.streamed_body(data);
            }
            RawResponseData::File(path) => {
                if let Some(file_name) = self.file_name {
                    if !file_name.is_empty() {
                        response.raw_header("Content-Disposition", format!("inline; filename*=UTF-8''{}", percent_encoding::percent_encode(file_name.as_bytes(), percent_encoding::QUERY_ENCODE_SET)));
                    }
                } else {
                    if let Some(file_name) = path.file_name().map(|file_name| file_name.to_string_lossy()) {
                        response.raw_header("Content-Disposition", format!("inline; filename*=UTF-8''{}", percent_encoding::percent_encode(file_name.as_bytes(), percent_encoding::QUERY_ENCODE_SET)));
                    }
                }

                if let Some(content_type) = self.content_type {
                    response.raw_header("Content-Type", content_type.to_string());
                } else {
                    if let Some(extension) = path.extension() {
                        if let Some(extension) = extension.to_str() {
                            let content_type = mime_guess::get_mime_type(extension);

                            response.raw_header("Content-Type", content_type.to_string());
                        }
                    }
                }

                let file = File::open(path).map_err(|err| if err.kind() == ErrorKind::NotFound {
                    Status::NotFound
                } else {
                    Status::InternalServerError
                })?;

                response.sized_body(file);
            }
        }

        response.ok()
    }
}