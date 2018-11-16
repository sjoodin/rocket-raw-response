#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

extern crate rocket_raw_response;

use std::path::Path;

use rocket_raw_response::RawResponse;

#[get("/")]
fn view() -> RawResponse<'static> {
    let path = Path::join(Path::new("examples"), Path::join(Path::new("images"), "image(貓).jpg"));

    RawResponse::from_file(path, None::<String>, None).unwrap()
}

fn main() {
    rocket::ignite().mount("/", routes![view]).launch();
}