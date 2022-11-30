use hbb_common::log;
use std::process::Command;

use serde::Deserialize;

pub(crate) fn require_authorization() -> bool {
    true
}

fn make_url() -> String {
    return format!("{}/api/check_login", "http://172.17.0.1:8000");
}

#[derive(Deserialize, Debug)]
struct CheckLoginResp {
    success: bool,
}

pub fn post_request(url: String, body: String, header: &str) -> Result<String, std::io::Error> {
    #[cfg(not(target_os = "linux"))]
    {
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

    #[cfg(target_os = "linux")]
    {
        let mut data = vec![
            "curl",
            "-sS",
            "-X",
            "POST",
            &url,
            "-H",
            "Content-Type: application/json",
            "-d",
            &body,
            "--connect-timeout",
            "12",
        ];
        log::error!("data request {:#?}", data);
        if !header.is_empty() {
            data.push("-H");
            data.push(header);
        }
        let output = Command::new("curl").args(&data).output()?;
        let res = String::from_utf8_lossy(&output.stdout).to_string();
        if !res.is_empty() {
            return Ok(res);
        }

        Err(std::io::Error::from(std::io::ErrorKind::ConnectionReset))
    }
}

fn request_authorization(access_token: String) -> bool {
    let body = serde_json::json!({ "access_token": access_token });

    let resp = post_request(make_url(), body.to_string(), "");

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

pub(crate) fn check_authorization(access_token: String) -> bool {
    if require_authorization() && access_token.is_empty() {
        return false;
    }

    request_authorization(access_token)
}
