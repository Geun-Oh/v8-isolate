use lambda_http::{run, service_fn, tracing, Body, Error, Request, Response};
use v8;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    // let who = event
    //     .query_string_parameters_ref()
    //     .and_then(|params| params.first("name"))
    //     .unwrap_or("world");
    let message = create_isolate();

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(message.into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}

fn create_isolate() -> String {
    let create_params = v8::Isolate::create_params().heap_limits(0, 10 * 1024 * 1024);
    let isolate = &mut v8::Isolate::new(create_params);
    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope, Default::default());
    let scope = &mut v8::ContextScope::new(scope, context);

    let code = v8::String::new(scope, "'Hello' + ' World!'").unwrap();
    let script = v8::Script::compile(scope, code, None).unwrap();
    let result = script.run(scope).unwrap();
    let result = result.to_string(scope).unwrap();
    format!("result: {}", result.to_rust_string_lossy(scope))
}