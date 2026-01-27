use std::fs;
use std::path::Path;
use tiny_http::{Header, Response, Server};
use crate::managers::appdata::AppData;

static HTTP_ADDRESS: &str = "127.0.0.1:8081";

pub fn handle_http() {
    let server = Server::http("127.0.0.1:8081").unwrap();
    println!("Http server running on http://127.0.0.1:8081");

    for request in server.incoming_requests() {
        let url = request.url();

        if url.starts_with("/logos/") {
            let url_path = Path::new(url);
            if url_path.file_name().is_none() {
                return;
            }

            let os_filename = url_path.file_name().unwrap();

            let try_filename = os_filename.to_str();
            if try_filename.is_none() {
                return;
            }

            let filename = try_filename.unwrap();
            let filepath = AppData::get_data_path().join("logos").join(filename);

            match fs::read(&filepath) {
                Ok(content) => {
                    let content_type = if filename.ends_with(".png") {
                        "image/png"
                    } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
                        "image/jpeg"
                    } else {
                        "application/octet-stream"
                    };

                    let header = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap();
                    let response = Response::from_data(content).with_header(header);
                    let _ = request.respond(response);
                }
                Err(e) => {
                    let response = Response::from_string("404 - File not found")
                        .with_status_code(404);
                    let _ = request.respond(response);
                }
            }
        } else {
            let response = Response::from_string("404 - Not found")
                .with_status_code(404);
            let _ = request.respond(response);
        }
    }
}