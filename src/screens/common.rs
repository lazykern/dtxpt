use bevy::prelude::*;

pub fn cleanup_screen<T: Component>(
    mut commands: Commands,
    entities: Query<Entity, (With<T>, Without<ChildOf>)>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
