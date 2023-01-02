use std::ops::Add;

use bevy::prelude::*;
use librelations::{
    cyclicity::{Acyclic, Cyclic},
    restriction::{Many, One},
    EntityCommandsExt, NoitalerRef, RelKind, RelationRef, WithRelation,
};

use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(create_groups)
        .add_system(move_camera)
        .add_system(move_bevys)
        .add_system(insert_group_target)
        .add_system(remove_group_target)
        .add_system(set_group_position)
        .run();
}

struct InGroup;
impl RelKind for InGroup {
    type SourceRestriction = One;
    type TargetRestriction = Many;
    type Cyclicity = Acyclic;
}

struct MoveToGroup;
impl RelKind for MoveToGroup {
    type SourceRestriction = One;
    type TargetRestriction = Many;
    type Cyclicity = Cyclic;
}

#[derive(Component)]
struct Group {
    position: Vec2,
}

#[derive(Component)]
struct TargetOffset(Vec2);

const NUM_GROUPS: u32 = 30;
const MAP_BOUNDS: (f32, f32) = (800., 500.);
const GROUP_RANGE: (f32, f32) = (-100., 100.);
const GROUP_UNITS_RANGE: (u32, u32) = (50, 150);

fn create_groups(mut commands: Commands<'_, '_>, assets: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let mut rng = rand::thread_rng();
    let texture = assets.load("icon.png");

    for _ in 0..NUM_GROUPS {
        let color = Color::rgb(rng.gen(), rng.gen(), rng.gen());
        let group_pos = Vec2::new(
            rng.gen_range(-MAP_BOUNDS.0..=MAP_BOUNDS.0),
            rng.gen_range((-MAP_BOUNDS.1)..=MAP_BOUNDS.1),
        );

        let group_id = commands
            .spawn(Group {
                position: group_pos,
            })
            .id();

        let group_member_count = rng.gen_range(GROUP_UNITS_RANGE.0..=GROUP_UNITS_RANGE.1);
        for _ in 0..group_member_count {
            commands
                .spawn(SpriteBundle {
                    texture: texture.clone(),
                    sprite: Sprite { color, ..default() },
                    ..default()
                })
                .insert(TargetOffset(Vec2::new(
                    rng.gen_range(GROUP_RANGE.0..=GROUP_RANGE.1),
                    rng.gen_range(GROUP_RANGE.0..=GROUP_RANGE.1),
                )))
                .insert(
                    Transform::from_translation(
                        Vec2::new(
                            rng.gen_range(GROUP_RANGE.0..=GROUP_RANGE.1),
                            rng.gen_range(GROUP_RANGE.0..=GROUP_RANGE.1),
                        )
                        .add(group_pos)
                        .extend(0.0),
                    )
                    .with_scale(Vec3::ONE * 0.1),
                )
                .insert_relation(InGroup, group_id);
        }
    }
}

fn move_camera(keys: Res<Input<KeyCode>>, mut query: Query<&mut Transform, With<Camera>>) {
    let mut transform = query.get_single_mut().unwrap();
    if keys.pressed(KeyCode::A) {
        transform.translation -= Vec3::new(10., 0., 0.);
    }
    if keys.pressed(KeyCode::D) {
        transform.translation += Vec3::new(10., 0., 0.);
    }
    if keys.pressed(KeyCode::W) {
        transform.translation += Vec3::new(0., 10., 0.);
    }
    if keys.pressed(KeyCode::S) {
        transform.translation -= Vec3::new(0., 10., 0.);
    }
    transform.translation.x = transform.translation.x.clamp(-MAP_BOUNDS.0, MAP_BOUNDS.0);
    transform.translation.y = transform.translation.y.clamp(-MAP_BOUNDS.1, MAP_BOUNDS.1);
}

fn set_group_position(
    bevys: Query<&Transform>,
    mut groups: Query<(&mut Group, NoitalerRef<InGroup>)>,
) {
    for (mut group, sources) in &mut groups {
        let average_pos = sources
            .iter()
            .map(|source| bevys.get(source).unwrap().translation.truncate())
            .sum::<Vec2>()
            / sources.iter().count() as f32; // really this ought to be `Itertools::tree_fold_1(|a, b| (a + b) / 2.0)`
        group.position = average_pos;
    }
}

fn remove_group_target(
    mut commands: Commands<'_, '_>,
    groups: Query<(Entity, &Group, Option<RelationRef<MoveToGroup>>)>,
) {
    for (group_id, group, move_to) in groups.iter().filter_map(|(group_id, group, opt_move_to)| {
        opt_move_to.map(|move_to| (group_id, group, move_to))
    }) {
        let (target_group, _) = move_to.into_iter().next().unwrap();
        let (target_group_id, target_group, _) = groups.get(target_group).unwrap();

        if (target_group.position - group.position).abs().length() <= 100.0 {
            commands
                .entity(group_id)
                .remove_relation::<MoveToGroup>(target_group_id);
        }
    }
}

fn insert_group_target(
    mut commands: Commands<'_, '_>,
    groups: Query<(Entity, Option<WithRelation<MoveToGroup>>), With<Group>>,
) {
    let mut rng = rand::thread_rng();

    for (group_id, _) in groups.iter().filter(|(_, has_target)| has_target.is_none()) {
        let (target_id, _) = groups
            .iter()
            .filter(|(id, _)| *id != group_id)
            .nth(rng.gen_range(0..(groups.iter().len() - 1)))
            .unwrap();

        commands
            .entity(group_id)
            .insert_relation(MoveToGroup, target_id);
    }
}

fn move_bevys(
    mut bevys: Query<(&mut Transform, &TargetOffset, RelationRef<InGroup>)>,
    groups: Query<(&Group, Option<RelationRef<MoveToGroup>>)>,
) {
    for (mut bevy_pos, offset, in_group) in &mut bevys {
        let in_group = in_group.into_iter().next().unwrap().0;
        let move_to_group = match groups.get(in_group).unwrap() {
            (_, None) => continue,
            (_, Some(move_to_group)) => move_to_group,
        };
        let move_to_group = move_to_group.into_iter().next().unwrap().0;
        let (move_to_group, _) = groups.get(move_to_group).unwrap();

        let velocity = ((move_to_group.position + offset.0) - bevy_pos.translation.truncate())
            .normalize_or_zero();
        bevy_pos.translation += velocity.extend(0.0);
    }
}
