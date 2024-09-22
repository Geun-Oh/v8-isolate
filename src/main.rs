mod isolate_pool;

use lambda_http::{run, service_fn, tracing, Body, Error, Request, Response};
use isolate_pool::{IsolatePool, IsolateWithIdx};
use std::env;
use v8::HandleScope;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let pool = IsolatePool::new(5);

    match pool.get_isolate() {
        Some(isolate_with_idx) => {
            let return_resp;
            let idx = isolate_with_idx.idx;
            let mut isolate = isolate_with_idx.isolate;

            {
                let scope = &mut v8::HandleScope::new(&mut isolate);
                let context = v8::Context::new(scope, Default::default());
                let scope = &mut v8::ContextScope::new(scope, context);
            
                // let who = event
                //     .query_string_parameters_ref()
                //     .and_then(|params| params.first("name"))
                //     .unwrap_or("world");
                let message = execute_script(scope);

                let resp = Response::builder()
                .status(200)
                .header("content-type", "text/html")
                .body(message.into())
                .map_err(Box::new)?;

                return_resp = resp;
            }

            pool.return_isolate(IsolateWithIdx { isolate, idx });
            return Ok(return_resp)
        },
        None => {
            return Err(Error::from("Failed to get isolate"));
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}

fn execute_script(scope: &mut HandleScope) -> String {
    let code = v8::String::new(scope, "'Hello' + ' World!'").unwrap();
    let script = v8::Script::compile(scope, code, None).unwrap();
    let result = script.run(scope).unwrap();
    let result = result.to_string(scope).unwrap();
    if is_warm_start() {
        format!("result: {} with {}", result.to_rust_string_lossy(scope), "provisioned concurrency")
    } else {
        format!("result: {} with {}", result.to_rust_string_lossy(scope), "new instance")
    }
}

fn is_warm_start() -> bool {
    if let Ok(initialization_type) = env::var("AWS_LAMBDA_INITIALIZATION_TYPE") {
        initialization_type == "provisioned-concurrency"
    } else {
        false
    }
}