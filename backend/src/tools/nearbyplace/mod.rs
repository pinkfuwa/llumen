use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use dotenv::var;
use crate::tools::Tool;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NearByPlace;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NearByPlaceInput {
    keyword: String,
    radius: Option<u32>, // in meters
}
impl Tool for NearByPlace {
    type Input = NearByPlaceInput;
    type Output = String;

    const NAME: &str = "nearbyplace";
    const DESCRIPTION: &str = "get nearby place info in json format.
    you can use this api to find some types of places.
    Then you can use the result to answer user questions such as 'What are some good restaurants near me?' or 'Find me a nearby hotel'.
    keywords can be: restaurant, hotel, museum, park, bank, pub, hospital, bus_station, arena, supermarket.
    ";
    const PROMPT: &str = "use `nearbyplace` to get nearby place info when user request";

    async fn call(&mut self, input: Self::Input) -> anyhow::Result<Self::Output> {
        let url = "https://places.googleapis.com/v1/places:searchNearby";
        let api_key = var("GOOGLE_MAP_API_KEY").unwrap_or("".to_owned());
        let body = serde_json::json!({
            "includedTypes": [input.keyword],
            "maxResultCount": 10,
            "locationRestriction": {
                "circle": {
                    "center": {
                        "latitude": 24.7944222,
                        "longitude": 120.988158
                    },
                    "radius": std::cmp::min(input.radius.unwrap_or(5000), 50000) // default to 10000 meters, max 50000
                }
            }
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-Goog-Api-Key", api_key)
            .header("X-Goog-FieldMask", "places.displayName,places.formattedAddress,places.addressDescriptor,places.priceLevel,places.rating,places.currentOpeningHours")
            .json(&body)
            .send()
            .await?
            .text()
            .await?;

        Ok(resp)
    }
}
