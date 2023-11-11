use bevy::prelude::*;

use crate::*;

#[derive(Resource)]
pub struct Score(pub u32);

#[derive(Component)]
pub struct Scoreboard;

pub fn edit_objective_label(
    mut objective_label_query: Query<&mut Text, With<ObjectiveLabel>>,
    objective: Res<State<Objective>>,
) {
    for mut text in &mut objective_label_query {
        text.sections[0].value = format!("Current Objective: {}", objective.get().get_name());
    }
}
