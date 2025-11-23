use crate::{
    config::{FilterBy, SkillConfig},
    models::world::{Location, SkillItem},
};

fn find_best_skill_for_location<'a>(
    location: &'a Location,
    config: &SkillConfig,
) -> Option<&'a SkillItem> {
    let skills = &location.skill_items;

    let skills_of_type = skills
        .iter()
        .filter(|skill_item| skill_item.skill == config.skill_type);

    match config.filter_by {
        FilterBy::HighestLevelRequired => {
            skills_of_type.max_by_key(|skill_item| skill_item.level_required)
        }
        FilterBy::LowestLevelRequired => {
            skills_of_type.min_by_key(|skill_item| skill_item.level_required)
        }
        _ => unimplemented!(),
    }
}

pub fn find_best_skill<'a>(
    locations: &'a [Location],
    config: &SkillConfig,
) -> Option<(&'a Location, &'a SkillItem)> {
    let best_skills_per_location = locations.iter().filter_map(|location| {
        find_best_skill_for_location(location, config).map(|skill_item| (location, skill_item))
    });

    match config.filter_by {
        FilterBy::HighestLevelRequired => {
            best_skills_per_location.max_by_key(|(_, skill_item)| skill_item.level_required)
        }
        FilterBy::LowestLevelRequired => {
            best_skills_per_location.min_by_key(|(_, skill_item)| skill_item.level_required)
        }
        _ => unimplemented!(),
    }
}
