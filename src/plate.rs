use futures::future::try_join;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlateError {
    #[error("Not Found")]
    NotFound,

    #[error("{0}")]
    RequestError(#[from] reqwest::Error),

    #[error("{0}")]
    BadResponse(#[from] serde_json::Error),
}

type Result<T, E = PlateError> = std::result::Result<T, E>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FuelsaverVehicle {
    make: String,
    model: String,
}

#[derive(Debug, Deserialize)]
struct RightcarVehicle {
    colour: String,
}

#[derive(Debug, Deserialize)]
struct RightcarResponse {
    detail: Option<Vec<RightcarVehicle>>,
}

#[derive(Debug, Serialize)]
pub struct Vehicle {
    make: String,
    model: String,
    colour: String,
}

pub struct PlateClient {
    client: Client,
}

impl PlateClient {
    pub fn new(user_agent: Option<String>) -> Self {
        let mut client = reqwest::ClientBuilder::new().cookie_store(true);
        if let Some(user_agent) = user_agent {
            if !user_agent.is_empty() {
                client = client.user_agent(user_agent)
            }
        }

        Self {
            client: client.build().unwrap(),
        }
    }

    async fn search_fuelsaver(&self, plate: &str) -> Result<FuelsaverVehicle> {
        // To set cookie
        self.client
            .get("https://resources.fuelsaver.govt.nz/label-generator")
            .send()
            .await?;

        let params = json!({
            "api": "labels",
            "version": "test",
            "service": "fuel_label_generator",
            "action": "get_text",
            "plate": plate,
            "button": "2",
            "CCDpriceEligible": "",
        })
        .to_string();
        let response = self
            .client
            .get("https://resources.fuelsaver.govt.nz/label-generator/_ws/vfel_lookup.aspx")
            .query(&[("params", &params)])
            .send()
            .await?
            .text()
            .await?;
        log::debug!("{}", response);

        Ok(serde_json::from_str(&response)?)
    }

    async fn search_rightcar(&self, plate: &str) -> Result<RightcarVehicle> {
        let params = json!({ "q": format!("P{}", plate) }).to_string();
        let response_str = self
            .client
            .get("https://rightcar.govt.nz/_ws/get_detail.aspx")
            .query(&[("params", &params)])
            .send()
            .await?
            .text()
            .await?;
        log::debug!("{}", response_str);

        let response: RightcarResponse = serde_json::from_str(&response_str)?;

        let vehicle = response.detail.and_then(|mut d| d.pop());
        if let Some(vehicle) = vehicle {
            Ok(vehicle)
        } else {
            Err(PlateError::NotFound)
        }
    }

    pub async fn search_plate(&self, plate: &str) -> Result<Vehicle> {
        let (fuelsaver, rightcar) =
            try_join(self.search_fuelsaver(plate), self.search_rightcar(plate)).await?;

        Ok(Vehicle {
            make: fuelsaver.make,
            model: fuelsaver.model,
            colour: rightcar.colour,
        })
    }
}
