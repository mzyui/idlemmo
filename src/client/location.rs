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
    utils::generate_obfuscated_data,
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

        let all_locs_url = Parser::LocationsAllApiEndpoint.get_value(&self.cache.html)?;
        debug!(url = %all_locs_url, "Calling API: Get All Locations");

        let response = self.client.post(all_locs_url).send().await?;
        let json: Value = response.json().await?;

        let location_ids: Vec<i64> = json
            .as_object()
            .map(|obj| {
                obj.values()
                    .filter_map(|v| v.get("id").and_then(serde_json::Value::as_i64))
                    .collect()
            })
            .unwrap_or_default();

        let total_locs = location_ids.len();
        debug!(count = total_locs, "Found initial locations.");
        let mut locations = Vec::with_capacity(total_locs);
        if total_locs == 0 {
            warn!("No locations found in the initial fetch.");
        } else {
            let quick_view_url =
                Parser::QuickViewLocationApiEndpoint.get_value(&self.cache.html)?;
            for location_id in location_ids {
                let response = self
                    .client
                    .post(&quick_view_url)
                    .json(&json!({ "location_id": location_id }))
                    .send()
                    .await?;

                let mut location = response.json::<Location>().await?;
                location
                    .enemies
                    .retain(|enemy| enemy.level >= self.cache.character_info.combat_level);
                location.skill_items.retain(|skill| {
                    let level = self
                        .cache
                        .character_info
                        .skill_level
                        .entry(skill.skill_type.clone())
                        .or_default();
                    *level >= skill.level_required
                });
                if !location.enemies.is_empty() || !location.skill_items.is_empty() {
                    locations.push(location);
                }
            }
            locations.sort_by_key(|v| Reverse(v.distance));
            self.cache.locations.clone_from(&locations);
        }
        info!(count = locations.len(), "Finished filtering locations.");
        Ok(locations)
    }

    #[tracing::instrument(skip(self, location, travel_mode))]
    async fn move_location(&mut self, travel_mode: TravelMode, location: Location) -> Result<()> {
        info!(location = %location.name, ?travel_mode, "Attempting to move to new location.");
        let message = match travel_mode {
            TravelMode::Teleport => {
                let current_gold = self.cache.character_info.gold;
                if current_gold < location.teleport_cost {
                    warn!(
                        current_gold,
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

                if current_gold > self.cache.character_info.gold {
                    "Teleport successful".to_string()
                } else {
                    "You already at location".to_string()
                }
            }
            TravelMode::Walk => {
                let api_url = Parser::LocationsTravelApiEndpoint.get_value(&self.cache.html)?;
                let response = self
                    .client
                    .post(api_url)
                    .json(&json!({
                        "location_id": location.id,
                        "ts2mic5ytx": generate_obfuscated_data(None),
                        "qty6bx4peh": generate_obfuscated_data(None),
                        "v": "1.0.0.1"
                    }))
                    .send()
                    .await?;
                let msg_data = response.json::<ResponseData>().await?;
                msg_data.message
            }
        };

        info!(
            location = %location.name,
            "{}", message
        );
        Ok(())
    }
}
