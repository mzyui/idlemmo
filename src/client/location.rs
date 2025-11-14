use std::cmp::Reverse;

use async_trait::async_trait;
use serde_json::{Value, json};
use tracing::{debug, info, warn};

use crate::{
    client::IdleMMOClient,
    error::Result,
    models::{
        ResponseData,
        location::{Location, TravelMode},
    },
    parser::Parser,
    utils::{API_VERSION, generate_obfuscated_data},
};

#[allow(dead_code)]
#[async_trait]
pub trait LocationApi {
    async fn get_locations(&mut self, load_from_cache: bool) -> Result<Vec<Location>>;
    async fn move_location(&mut self, travel_mode: TravelMode, location: Location) -> Result<()>;
}

#[async_trait]
impl LocationApi for IdleMMOClient {
    #[tracing::instrument(skip(self, load_from_cache))]
    async fn get_locations(&mut self, load_from_cache: bool) -> Result<Vec<Location>> {
        if load_from_cache && !self.cache.locations.is_empty() {
            return Ok(self.cache.locations.clone());
        }

        let all_locations_api_url = Parser::LocationsAllApiEndpoint.get_value(&self.cache.html)?;
        debug!(url = %all_locations_api_url, "Calling API: Get All Locations");

        let http_response = self.client.post(all_locations_api_url).send().await?;
        let json_response_data: Value = http_response.json().await?;

        let raw_location_ids: Vec<u64> = json_response_data
            .as_object()
            .map(|obj| {
                obj.values()
                    .filter_map(|v| v.get("id").and_then(serde_json::Value::as_u64))
                    .collect()
            })
            .unwrap_or_default();

        let total_raw_locations = raw_location_ids.len();
        debug!(count = total_raw_locations, "Found initial locations.");
        let mut filtered_locations = Vec::with_capacity(total_raw_locations);
        if total_raw_locations == 0 {
            warn!("No locations found in the initial fetch.");
        } else {
            let quick_view_api_url =
                Parser::QuickViewLocationApiEndpoint.get_value(&self.cache.html)?;
            for current_location_id in raw_location_ids {
                let quick_view_response = self
                    .client
                    .post(&quick_view_api_url)
                    .json(&json!({ "location_id": current_location_id }))
                    .send()
                    .await?;

                let mut current_location_details = quick_view_response.json::<Location>().await?;
                current_location_details
                    .enemies
                    .retain(|current_enemy| current_enemy.level >= self.cache.character_info.combat_level);
                current_location_details.skill_items.retain(|current_skill| {
                    let character_skill_level = self
                        .cache
                        .character_info
                        .skill_level
                        .entry(current_skill.skill_type.clone())
                        .or_default();
                    *character_skill_level >= current_skill.level_required
                });
                if !current_location_details.enemies.is_empty() || !current_location_details.skill_items.is_empty() {
                    filtered_locations.push(current_location_details);
                }
            }
            filtered_locations.sort_by_key(|location_by_distance| Reverse(location_by_distance.distance));
            self.cache.locations.clone_from(&filtered_locations);
        }
        info!(count = filtered_locations.len(), "Finished filtering locations.");
        Ok(filtered_locations)
    }

    #[tracing::instrument(skip(self, location, travel_mode))]
    async fn move_location(&mut self, travel_mode: TravelMode, location: Location) -> Result<()> {
        info!(location = %location.name, ?travel_mode, "Attempting to move to new location.");
        let status_message = match travel_mode {
            TravelMode::Teleport => {
                let character_gold_amount = self.cache.character_info.gold;
                if character_gold_amount < location.teleport_cost {
                    warn!(
                        current_gold = character_gold_amount,
                        cost = location.teleport_cost,
                        "Teleport failed: Not enough gold."
                    );
                    return Ok(());
                }

                self.client
                    .post(format!(
                        "{}locations/teleport/{}",
                        self.base_url, location.key
                    ))
                    .form(&json!({
                        "_token": self.cache.csrf_token,
                    }))
                    .send()
                    .await?;

                self.update_current_data().await?;

                if character_gold_amount != self.cache.character_info.gold {
                    "Teleport successful".to_string()
                } else {
                    "You already at location".to_string()
                }
            }
            TravelMode::Walk => {
                let travel_api_url = Parser::LocationsTravelApiEndpoint.get_value(&self.cache.html)?;
                let travel_http_response = self
                    .client
                    .post(travel_api_url)
                    .json(&json!({
                        "location_id": location.id,
                        "ts2mic5ytx": generate_obfuscated_data(None),
                        "qty6bx4peh": generate_obfuscated_data(None),
                        "v": API_VERSION
                    }))
                    .send()
                    .await?;
                let response_message_data = travel_http_response.json::<ResponseData>().await?;
                response_message_data.message
            }
        };

        info!(
            location = %location.name,
            "{}", status_message
        );
        Ok(())
    }
}
