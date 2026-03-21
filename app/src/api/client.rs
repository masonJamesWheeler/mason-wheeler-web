use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: u16,
    pub message: String,
    pub code: Option<String>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(feature = "hydrate")]
async fn parse_error(resp: gloo_net::http::Response) -> ApiError {
    let status = resp.status();
    let message = resp
        .json::<serde_json::Value>()
        .await
        .ok()
        .and_then(|v| {
            v.get("error")
                .or(v.get("message"))
                .and_then(|e| e.as_str().map(String::from))
        })
        .unwrap_or_else(|| format!("Request failed ({})", status));
    let code = None;
    ApiError { status, message, code }
}

#[cfg(feature = "hydrate")]
pub async fn api_get<T: DeserializeOwned>(url: &str) -> Result<T, ApiError> {
    let resp = gloo_net::http::Request::get(url)
        .send()
        .await
        .map_err(|e| ApiError {
            status: 0,
            message: format!("Network error: {}", e),
            code: None,
        })?;

    if resp.ok() {
        resp.json::<T>().await.map_err(|e| ApiError {
            status: 0,
            message: format!("Parse error: {}", e),
            code: None,
        })
    } else {
        Err(parse_error(resp).await)
    }
}

#[cfg(feature = "hydrate")]
pub async fn api_post<T: DeserializeOwned>(
    url: &str,
    body: &impl Serialize,
) -> Result<T, ApiError> {
    let resp = gloo_net::http::Request::post(url)
        .json(body)
        .map_err(|e| ApiError {
            status: 0,
            message: format!("Serialize error: {}", e),
            code: None,
        })?
        .send()
        .await
        .map_err(|e| ApiError {
            status: 0,
            message: format!("Network error: {}", e),
            code: None,
        })?;

    if resp.ok() {
        resp.json::<T>().await.map_err(|e| ApiError {
            status: 0,
            message: format!("Parse error: {}", e),
            code: None,
        })
    } else {
        Err(parse_error(resp).await)
    }
}

#[cfg(feature = "hydrate")]
pub async fn api_post_no_body(
    url: &str,
    body: &impl Serialize,
) -> Result<(), ApiError> {
    let resp = gloo_net::http::Request::post(url)
        .json(body)
        .map_err(|e| ApiError {
            status: 0,
            message: format!("Serialize error: {}", e),
            code: None,
        })?
        .send()
        .await
        .map_err(|e| ApiError {
            status: 0,
            message: format!("Network error: {}", e),
            code: None,
        })?;

    if resp.ok() {
        Ok(())
    } else {
        Err(parse_error(resp).await)
    }
}

#[cfg(feature = "hydrate")]
pub async fn api_patch<T: DeserializeOwned>(
    url: &str,
    body: &impl Serialize,
) -> Result<T, ApiError> {
    let resp = gloo_net::http::Request::patch(url)
        .json(body)
        .map_err(|e| ApiError {
            status: 0,
            message: format!("Serialize error: {}", e),
            code: None,
        })?
        .send()
        .await
        .map_err(|e| ApiError {
            status: 0,
            message: format!("Network error: {}", e),
            code: None,
        })?;

    if resp.ok() {
        resp.json::<T>().await.map_err(|e| ApiError {
            status: 0,
            message: format!("Parse error: {}", e),
            code: None,
        })
    } else {
        Err(parse_error(resp).await)
    }
}

#[cfg(feature = "hydrate")]
pub async fn api_delete(url: &str) -> Result<(), ApiError> {
    let resp = gloo_net::http::Request::delete(url)
        .send()
        .await
        .map_err(|e| ApiError {
            status: 0,
            message: format!("Network error: {}", e),
            code: None,
        })?;

    if resp.ok() {
        Ok(())
    } else {
        Err(parse_error(resp).await)
    }
}
