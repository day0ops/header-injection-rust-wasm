#[macro_use]
extern crate lazy_static;

use std::{time::Duration, collections::HashMap};

use proxy_wasm::traits::*;
use proxy_wasm::types::*;

use url::{Url, Position, ParseError};
use log::{debug, info, error};
use serde::{Deserialize};
use anyhow::{bail, Result};

// TODO: Definitely not necessary to lazy load but just defining this for global use
lazy_static! {
    static ref POSTMAN_URL: Result<Url, ParseError> = Url::parse("https://postman-echo.com/get");
    static ref POSTMAN_URL_PARAMS: HashMap<String, String> = {
        let mut map = HashMap::new();
        map.insert(String::from("filter_name"), String::from("rust"));
        map.insert(String::from("author"), String::from("kasunt"));
        map
    };
}

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { 
        Box::new(HeaderInjectionRoot {
        }) 
    });
}}

struct HeaderInjectionRoot;

impl Context for HeaderInjectionRoot {}

impl RootContext for HeaderInjectionRoot {
    // To pass configuration to the filter
    fn on_configure(&mut self, _: usize) -> bool {
        true
    }

    // Main HTTP context
    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(ProcessHeadersHttpFilter {
            context_id: _context_id.to_string(),
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

struct ProcessHeadersHttpFilter {
    context_id: String,
}

impl HttpContext for ProcessHeadersHttpFilter {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        info!("Request intercepted by the header-injection filter");
        let path = self.get_http_request_header(":path").unwrap();

        // We only care about this path. Everything else is passed through.
        // TODO composite filters might be preferred instead of this
        if path.contains("/get") {
            match self.call_external_service() {
                Ok(()) => {
                    debug!("Called external service successfully");
                    Action::Pause
                }
                Err(e) => {
                    error!("Looks like we were unable to call the external service {}", e);
                    Action::Pause
                }
            }
        } else {
            Action::Continue
        }
    }

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        Action::Continue
    }

    fn on_log(&mut self) {
        debug!("Context #{} completed in filter", self.context_id);
    }
}

impl Context for ProcessHeadersHttpFilter {
    // Process response from the external service
    fn on_http_call_response(&mut self, _: u32, _: usize, body_size: usize, _: usize) {
        match parse_response_body(self.get_http_call_response_body(0, body_size)) {
            Ok(body) => {
                self.add_http_request_header("author", body.args.author.as_str());
                self.add_http_request_header("filter_name", body.args.filter_name.as_str());
                self.resume_http_request();
            }
            Err(e) => {
                error!("Looks like there was a problem parsing the response body {}", e);
                // Best way to throw a response up the chain
                self.send_http_response(
                    400,
                    vec![],
                    Some(b"Unknown Gateway Error.\n"),
                );
            }
        }
    }
}

impl ProcessHeadersHttpFilter {
    fn call_external_service(&self) -> Result<()> {
        let url = POSTMAN_URL.as_ref().unwrap().as_str();
        let params = POSTMAN_URL_PARAMS.iter();
        let encoded_path = Url::parse_with_params(url, params)?;
        debug!("Encoded path {}", encoded_path.as_str());

        self.dispatch_http_call(
            "postman-echo-service",
            vec![ // Headers
                (":method", "GET"),
                (":path", &encoded_path[Position::BeforePath..]),
                (":authority", "postman-echo.com"),
            ],
            None, // Body
            vec![], // Trailers
            Duration::from_secs(5), // Timeout
        ).unwrap();

        return Ok(())
    }
}

#[derive(Deserialize)]
struct Args {
    author: String,
    filter_name: String
}

#[derive(Deserialize)]
struct DecodedBody {
    args: Args
}

fn parse_response_body(body_op: Option<Bytes>) -> Result<DecodedBody> {
    if let Some(body) = body_op {
        let decoded: DecodedBody = serde_json::from_slice(&body).unwrap();
        return Ok(decoded)
    }
    bail!("Unable to parse body")
}