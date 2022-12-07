use bevy::{ecs::world::EntityMut, prelude::Entity};

use crate::{
    restriction::{Many, One},
    EntityRefExt, RelKind, Restriction,
};

pub struct Cyclic;
pub struct Acyclic;

pub trait Cyclicity: super::sealed::Sealed {}
impl Cyclicity for Cyclic {}
impl Cyclicity for Acyclic {}

pub trait AssertTreeIfAcyclic<R, SourceRestriction, TargetRestriction>
where
    R: RelKind,
    SourceRestriction: Restriction<R>,
    TargetRestriction: Restriction<R>,
{
    fn assert_cyclicity(entity: EntityMut<'_>) -> Result<(), ()>;
}

fn assert_cyclicity<R: RelKind>(
    mut entity: EntityMut<'_>,
    next_step: impl Fn(&mut EntityMut<'_>) -> Option<Entity>,
) -> Result<(), ()> {
    let detector_id = entity.id();

    loop {
        match next_step(&mut entity) {
            Some(target) if target == detector_id => return Err(()),
            Some(target) => entity = entity.into_world_mut().entity_mut(target),
            None => return Ok(()),
        };
    }
}

impl<R: RelKind> AssertTreeIfAcyclic<R, One, Many> for Acyclic {
    fn assert_cyclicity(entity: EntityMut<'_>) -> Result<(), ()> {
        assert_cyclicity::<R>(entity, |entity| {
            entity
                .get_relation::<R>()
                .map(|r| r.iter().next().unwrap().0)
        })
    }
}
impl<R: RelKind> AssertTreeIfAcyclic<R, Many, One> for Acyclic {
    fn assert_cyclicity(entity: EntityMut<'_>) -> Result<(), ()> {
        assert_cyclicity::<R>(entity, |entity| {
            entity.get_noitaler::<R>().map(|r| r.iter().next().unwrap())
        })
    }
}
impl<R: RelKind> AssertTreeIfAcyclic<R, One, One> for Acyclic {
    fn assert_cyclicity(entity: EntityMut<'_>) -> Result<(), ()> {
        assert_cyclicity::<R>(entity, |entity| {
            entity.get_noitaler::<R>().map(|r| r.iter().next().unwrap())
        })
    }
}
impl<R: RelKind, T: Restriction<R>, U: Restriction<R>> AssertTreeIfAcyclic<R, T, U> for Cyclic {
    fn assert_cyclicity(_: EntityMut<'_>) -> Result<(), ()> {
        Ok(())
    }
}
