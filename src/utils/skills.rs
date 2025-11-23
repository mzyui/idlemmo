use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::models::character::Profile;
use crate::models::config::SkillConfig;
use crate::models::game_action::{SkillRecommendation, SkillType};
use crate::models::item::FilterBy;
use crate::models::world::{Location, SkillItem};
#[derive(Debug)]
pub struct FarmingStep<'a> {
    pub location: &'a Location,
    pub skills: Vec<SkillType>,
    pub items: Vec<&'a SkillItem>,
}

fn is_current_location(profile: &Profile, loc: &Location) -> bool {
    loc.id == profile.location_id || loc.distance == 0 || loc.teleport_cost == 50
}

fn distance_score(profile: &Profile, loc: &Location) -> u64 {
    if is_current_location(profile, loc) {
        0
    } else {
        loc.distance
    }
}

pub fn highest_skill(profile: &Profile) -> Option<(SkillType, u64)> {
    profile
        .skill_level
        .iter()
        .max_by_key(|(_, lvl)| *lvl)
        .map(|(skill, lvl)| (skill.clone(), *lvl))
}

pub fn find_item_for_skill<'a>(
    profile: &Profile,
    locations: &'a [Location],
    config: &SkillConfig,
) -> Option<(&'a Location, &'a SkillItem)> {
    let player_level = *profile.skill_level.get(&config.skill_type)?;

    let mut candidates: Vec<(&Location, &SkillItem)> = locations
        .iter()
        .filter(|loc| !loc.disabled)
        .flat_map(|loc| {
            loc.skill_items
                .iter()
                .filter({
                    let skill = config.skill_type.clone();
                    move |item| item.skill == skill && item.level_required <= player_level
                })
                .map(move |item| (loc, item))
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    match &config.filter_by {
        FilterBy::HighestLevelRequired => {
            candidates.sort_by(|(loc_a, item_a), (loc_b, item_b)| {
                item_b
                    .level_required
                    .cmp(&item_a.level_required)
                    .then_with(|| {
                        distance_score(profile, loc_a).cmp(&distance_score(profile, loc_b))
                    })
            });
        }
        FilterBy::LowestLevelRequired => {
            candidates.sort_by(|(loc_a, item_a), (loc_b, item_b)| {
                item_a
                    .level_required
                    .cmp(&item_b.level_required)
                    .then_with(|| {
                        distance_score(profile, loc_a).cmp(&distance_score(profile, loc_b))
                    })
            });
        }
        FilterBy::ItemName(target) => {
            candidates.retain(|(_, item)| item.name == *target);
            if candidates.is_empty() {
                return None;
            }
            candidates.sort_by(|(loc_a, item_a), (loc_b, item_b)| {
                distance_score(profile, loc_a)
                    .cmp(&distance_score(profile, loc_b))
                    .then_with(|| item_b.level_required.cmp(&item_a.level_required))
            });
        }
        FilterBy::ItemNameContains(substr) => {
            let substr_lower = substr.to_lowercase();
            candidates.retain(|(_, item)| item.name.to_lowercase().contains(&substr_lower));
            if candidates.is_empty() {
                return None;
            }
            candidates.sort_by(|(loc_a, item_a), (loc_b, item_b)| {
                distance_score(profile, loc_a)
                    .cmp(&distance_score(profile, loc_b))
                    .then_with(|| item_b.level_required.cmp(&item_a.level_required))
            });
        }
        FilterBy::Closest => {
            candidates.sort_by(|(loc_a, item_a), (loc_b, item_b)| {
                distance_score(profile, loc_a)
                    .cmp(&distance_score(profile, loc_b))
                    .then_with(|| item_b.level_required.cmp(&item_a.level_required))
            });
        }
    }

    candidates.into_iter().next()
}

pub fn recommend_all_skills<'a>(
    profile: &Profile,
    skill_config: &SkillConfig,
    locations: &'a [Location],
) -> Vec<SkillRecommendation<'a>> {
    let mut result = Vec::new();
    for (skill_type, &level) in &profile.skill_level {
        if let Some((loc, item)) = find_item_for_skill(profile, locations, skill_config) {
            result.push(SkillRecommendation {
                skill: skill_type.clone(),
                level,
                location: loc,
                item,
            });
        }
    }
    result.sort_by(|a, b| b.level.cmp(&a.level));
    result
}
