use std::sync::Arc;

use log::debug;

use serde::Deserialize;

use openfaas_runtime::Error;

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

/// The form of requests we are handling
#[derive(Deserialize, Debug)]
struct Person {
    name: String,
}

async fn handler(req: Person, greeter: Arc<Greeter>) -> Result<String, Error> {
    debug!("Received request: {:?}", req);
    Ok(greeter.greet(&req.name))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
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
    let handler = move |req: Person| handler(req, greeter.clone());

    // Invoke the runtime
    openfaas_runtime::run(handler).await?;

    Ok(())
}
