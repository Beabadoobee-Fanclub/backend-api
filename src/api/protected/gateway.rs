use axum::{
    body::Body,
    extract::{Path, Request},
    http::Response,
    Extension,
};
use worker::Env;

#[worker::send]
pub async fn handle_websocket(
    Path(id): Path<String>,
    Extension(env): Extension<Env>,
    req: Request,
) -> Response<Body> {
    let object = match env.durable_object("BOTROOM") {
        Ok(obj) => obj,
        Err(_) => return Response::new(Body::from("Error accessing durable object")),
    };

    let Ok(object_id) = object.id_from_name(&id) else {
        return Response::new(Body::from("Invalid object ID"));
    };

    let Ok(stub) = object_id.get_stub() else {
        return Response::new(Body::from("Error getting durable object stub"));
    };

    let url = req.uri().clone();

    let Ok(mut new_req) = worker::Request::new(&url.to_string(), worker::Method::Get) else {
        return Response::new(Body::from("Error creating request"));
    };

    for (key, value) in req.headers().iter() {
        if new_req
            .headers_mut()
            .expect("Failed to get headers")
            .append(key.as_str(), value.to_str().expect("Invalid header value"))
            .is_err()
        {
            return Response::new(Body::from("Error setting headers"));
        }
    }

    let res = match stub.fetch_with_request(new_req).await {
        Ok(response) => response,
        Err(_) => return Response::new(Body::from("Error fetching durable object")),
    };

    res.into()
}
