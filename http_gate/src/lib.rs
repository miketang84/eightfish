#![allow(unused_assignments)]

use anyhow::Result;
use spin_sdk::{
    http::{Request, Response,},
    http_component, redis
};
use http::Method;
use serde_json::json;
use uuid::Uuid;
use bytes::Bytes;

const REDIS_ADDRESS_ENV: &str = "REDIS_ADDRESS";

/// A simple Spin HTTP component.
#[http_component]
fn http_gate(req: Request) -> Result<Response> {

    let redis_addr= std::env::var(REDIS_ADDRESS_ENV)?;
    
    let uri = req.uri();
    let path = uri.path().to_owned();

    let mut method = String::new();
    let mut reqdata: Option<String> = None;
    match req.method() {
        &Method::GET => {
            method = "query".to_owned();

            // In query mode: data is the url params
            reqdata = match uri.query() {
                Some(query) => {
                    Some(query.to_string())
                }
                None => {
                    None
                }
            }
        }
        &Method::POST => {
            method = "post".to_owned();

            // In post mode: data is the body content of the request
            match req.into_body() {
                Some(body) => {
                    let bo = String::from_utf8_lossy(body.as_ref());
                    reqdata = Some(bo.to_string());
                }
                None => {
                    reqdata = None;
                }   
            }

        }
        _ => {
            // handle cases of other directives

        }
    };

    // We can do the unified authentication for some actions here
    // depends on path, method, and reqdata
    // XXX: 

    // use a unique way to generate a reqid
    let reqid = Uuid::new_v4();

    let payload = json!({
        "reqid": reqid.simple().to_string(),
        "reqdata": reqdata,
    });

    // construct a json, serialize it and send to a redis channel
    // model and action, we can plan a scheme to parse them out
    // here, we just put entire path content to action field, for later cases
    // we can parse it to model and action parts
    let json_to_send = json!({
        "model": path,
        "action": &method,
        "data": payload.to_string().as_bytes().to_vec(),
        "time": 0
    });

    if &method == "post" {
        // send to subxt proxy to handle
        _ = redis::publish(&redis_addr, "spin2proxy", &serde_json::to_vec(&json_to_send).unwrap());
    } else if &method == "query" {
        // send to spin_redis_worker to handle
        _ = redis::publish(&redis_addr, "proxy2spin", &serde_json::to_vec(&json_to_send).unwrap());
    }

    loop {
        let mut loop_count = 1;
        // loop the redis cache key of this procedure request
        let result = redis::get(&redis_addr, &format!("reqid:{reqid}"));
            //.map_err(|_| anyhow!("Error querying Redis"))?;
        match result {
            Ok(raw_result) => {
                // Now we get the raw serialized result from worker, we suppose it use
                // JSON spec to serialized it, so we can directly pass it back
                // to user's response body.
                // clear the redis cache key of the worker result
                // TODO: add del command to host function, pr to spin
                //let _ = redis::del(&redis_addr, &format!("reqid:{reqid}"));

                // jump out this loop, and return the response to user
                return Ok(http::Response::builder()
                          .status(200)
                          .header("openforum_version", "0.1")
                          .body(Some(Bytes::from(raw_result)))?);

            }
            Err(_) => {
                // after 6 seconds, timeout
                if loop_count < 600 {
                    // if not get the result, sleep for a little period
                    let ten_millis = std::time::Duration::from_millis(10);
                    std::thread::sleep(ten_millis);
                    loop_count += 1;
                }
                else {
                    // timeout handler, use which http status code?
                    return Ok(http::Response::builder()
                              .status(500)
                              .body(Some("No data".into()))?);
                }
            }
        }
    }
}

