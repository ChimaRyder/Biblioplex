pub mod mtgjson;

pub mod scryfall {
    use std::time::Duration;

    pub fn probe() -> Result<(), String> {
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(3))
            .user_agent("Biblioplex/0.1")
            .build()
            .map_err(|error| error.to_string())?
            .get("https://api.scryfall.com/cards/search?q=lightning")
            .send()
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}
