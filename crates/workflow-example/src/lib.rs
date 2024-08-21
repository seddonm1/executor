use workflow::{http, log, rand, workflow, Result};

#[workflow]
fn workflow() -> Result<()> {
    // Get the location of the ISS
    let result = http::get("http://localhost:3000/iss/now", None)?
        .error_for_status()
        .inspect_err(|err| {
            log::error!("could not get location: {:?}", err);
        })?;
    log::info!("current location: {}", result.text()?);

    // Send the notification if a condition is met
    if rand::rand::<bool>() {
        let result = http::post("http://localhost:3000/email/send", None, None)?
            .error_for_status()
            .inspect_err(|err| {
                log::error!("could not send email: {:?}", err);
            })?;
        log::info!("sent email: {}", result.error_for_status()?.text()?);
    }

    // Always update the database
    let result = http::post("http://localhost:3000/database/update", None, None)?
        .error_for_status()
        .inspect_err(|err| {
            log::error!("could not update database: {:?}", err);
        })?;
    log::debug!("database updated: {}", result.error_for_status()?.text()?);

    Ok(())
}
