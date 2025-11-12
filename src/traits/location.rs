use async_trait::async_trait;

use crate::error::Result;
use crate::models::location::{Location, TravelMode};

#[allow(dead_code)]
#[async_trait]
pub trait LocationApi {
    async fn get_locations(&mut self, load_from_cache: bool) -> Result<Vec<Location>>;
    async fn move_location(&mut self, travel_mode: TravelMode, location: Location) -> Result<()>;
}
