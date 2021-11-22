use log::debug;
use std::sync::Arc;

use openfaas_runtime::http;
use openfaas_runtime::http::Request;

use hyper::body::to_bytes;

struct Greeter {
    greet: String,
}

impl Greeter {
    /// Create a new Greeter object defining
    /// the greet
    fn new(greet: &str) -> Self {
        Self {
            greet: greet.to_string(),
        }
    }

    /// Greet someone
    fn greet(&self, person: &str) -> String {
        format!("{} {}", self.greet, person)
    }
}

#[derive(Debug)]
enum GreeterError {
    /// Hyper Error
    Transport(String),
    /// UTF8 Error
    Encoding(String),
}

impl From<hyper::Error> for GreeterError {
    fn from(error: hyper::Error) -> Self {
        GreeterError::Transport(format!("Transport error: {}", error).to_string())
    }
}

impl From<std::string::FromUtf8Error> for GreeterError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        GreeterError::Encoding(format!("Encoding error: {}", error).to_string())
    }
}

impl std::fmt::Display for GreeterError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GreeterError::Transport(error) => write!(fmt, "Transport error: {}", error),
            GreeterError::Encoding(error) => write!(fmt, "Encoding error: {}", error),
        }
    }
}

impl std::error::Error for GreeterError {}

async fn handler(req: Request, greeter: Arc<Greeter>) -> Result<String, GreeterError> {
    let vec = to_bytes(req.into_body()).await?.to_vec();
    let name = String::from_utf8(vec)?;

    debug!("Received request: '{}'", name);

    Ok(greeter.greet(&name))
}

#[tokio::main]
async fn main() {
    env_logger::init();

    // Create the object that will be shared across invocation calls
    let greeter = Arc::new(Greeter::new("Hello"));

    // Define our handler closure.
    //
    // The runtime expects an `FnOnce(serde_json::Value) -> Resp where Resp: Serialize`.
    //
    // So what we do here is create a closure with this type
    // which captures the a reference to the Greeter object
    // and calls the actual handler passing the reference as
    // an argument.
    let handler = move |req: Request| handler(req, greeter.clone());

    // Invoke the runtime
    http::run(handler).await;
}
