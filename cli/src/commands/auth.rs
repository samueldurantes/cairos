use std::{thread, time::Duration};

use crate::clients::github::GetUserAuthorizedResponse;

pub async fn github_login(reqwest: &reqwest::Client, base_url: &str) -> anyhow::Result<()> {
    let start_now = time::OffsetDateTime::now_utc();
    let user_codes = crate::clients::github::create_user_codes(reqwest).await?;

    println!("First copy your one-time code: {}", &user_codes.user_code);
    println!("Open {} in your browser...", &user_codes.verification_uri);

    loop {
        let now = time::OffsetDateTime::now_utc();

        if now >= start_now + time::Duration::seconds(user_codes.expires_in) {
            break;
        }

        match crate::clients::github::get_user_authorized(&reqwest, &user_codes.device_code).await {
            Ok(res) => match res {
                GetUserAuthorizedResponse::Success { access_token } => {
                    let login_response = crate::clients::cairos::login(
                        reqwest,
                        base_url,
                        crate::clients::cairos::LoginParams { access_token },
                    )
                    .await?;

                    super::config::set_token(login_response.token)?;

                    break;
                }
                GetUserAuthorizedResponse::Error { .. } => {}
            },
            Err(_) => break,
        }

        thread::sleep(Duration::from_secs(user_codes.interval));
    }

    Ok(())
}
