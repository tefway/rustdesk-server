use hbb_common::log;
use reqwest;

use serde::Deserialize;

pub(crate) fn require_authorization() -> bool {
    true
}

fn make_url_check_login() -> String {
    return format!("{}/api/check_login", "http://172.17.0.1:8000");
}

fn make_url_check_machine() -> String {
    return format!("{}/api/check_machine", "http://172.17.0.1:8000");
}

#[derive(Deserialize, Debug)]
struct CheckLoginResp {
    success: bool,
}

pub async fn post_request(
    url: String,
    body: String,
    header: &str,
) -> Result<String, reqwest::Error> {
    let mut req = reqwest::Client::new().post(url);
    if !header.is_empty() {
        let tmp: Vec<&str> = header.split(": ").collect();
        if tmp.len() == 2 {
            req = req.header(tmp[0], tmp[1]);
        }
    }
    req = req.header("Content-Type", "application/json");
    let to = std::time::Duration::from_secs(12);
    Ok(req.body(body).timeout(to).send().await?.text().await?)
}

async fn request_authorization(access_token: String, rdeskid: String) -> bool {
    let body = serde_json::json!({ "access_token": access_token, "id" : rdeskid });

    let resp = post_request(make_url_check_login(), body.to_string(), "").await;

    match resp {
        Ok(data) => match serde_json::from_str::<CheckLoginResp>(data.as_str()) {
            Ok(resp) => {
                return resp.success;
            }
            Err(error) => {
                log::error!("Fail to parse response {} {}", error, data);
                return false;
            }
        },
        Err(error) => {
            log::error!("Fail to get response {}", error);
            return false;
        }
    }
}

async fn request_machine_auth(uuid: String, rdeskid: String) -> bool {
    let body = serde_json::json!({ "uuid": uuid, "id" : rdeskid });

    let resp = post_request(make_url_check_machine(), body.to_string(), "").await;

    match resp {
        Ok(data) => match serde_json::from_str::<CheckLoginResp>(data.as_str()) {
            Ok(resp) => {
                return resp.success;
            }
            Err(error) => {
                log::error!("Fail to parse response {} {}", error, data);
                return false;
            }
        },
        Err(error) => {
            log::error!("Fail to get response {}", error);
            return false;
        }
    }
}

pub(crate) async fn check_authorization(access_token: String, rdeskid: String) -> bool {
    if require_authorization() && access_token.is_empty() {
        return false;
    }

    request_authorization(access_token, rdeskid).await
}

pub(crate) async fn check_machine_id(uuid: String, rdeskid: String) -> bool {
    if require_authorization() && uuid.is_empty() {
        return false;
    }

    request_machine_auth(uuid, rdeskid).await
}
