use bevy::ecs::{
    component::TableStorage,
    prelude::*,
    system::Command,
    world::{EntityMut, EntityRef},
};

#[cfg(test)]
mod testl;

pub mod cyclicity;
pub mod iter;
pub mod restriction;

use cyclicity::AssertTreeIfAcyclic;

pub use restriction::Restriction;
pub use world_queries::{
    NoitalerRef, NoitalerRefItem, RelationMut, RelationMutItem, RelationMutReadOnly as RelationRef,
    RelationMutReadOnlyItem as RelationRefItem,
};

pub use cyclicity::Cyclicity;

pub trait RelKind: Sized + Send + Sync + 'static {
    /// Number of relations of kind `Self` allowed on a source entity
    type SourceRestriction: Restriction<Self>;
    /// Number of relations of kind `Self` allowed to point to a target entity
    type TargetRestriction: Restriction<Self>;

    /// Whether cycles are allowed in the graph created by edges of `Self`. This can only be
    /// set to [`cyclicity::Acyclic`] if either [`Self::SourceRestriction`] or [`Self::TargetRestriction`] is set to [`restriction::One`]
    type Cyclicity: Cyclicity
        + cyclicity::AssertTreeIfAcyclic<Self, Self::SourceRestriction, Self::TargetRestriction>;
}

struct Relation<T: RelKind>(<T::SourceRestriction as Restriction<T>>::RelStorage);
#[derive(Component)]
struct Noitaler<T: RelKind>(<T::TargetRestriction as Restriction<T>>::NoiStorage);

impl<T: RelKind> Component for Relation<T> {
    type Storage = TableStorage;

    fn despawn_hook() -> fn(Entity, &mut World, bevy::ecs::component::NestedDespawns<'_>)
    where
        Self: Sized,
    {
        |e, world, mut despawner| {
            let mut entity = world.entity_mut(e);
            let mut rel = entity.remove::<Relation<T>>().unwrap();
            let noi = entity.remove::<Noitaler<T>>();
            for target in T::SourceRestriction::rel_iter(&mut rel.0).1 {
                let mut target_thing = world.entity_mut(target);
                let mut noi = target_thing.get_mut::<Noitaler<T>>().unwrap();

                if T::TargetRestriction::remove_noi(&mut noi.0, e) {
                    target_thing.remove::<Noitaler<T>>();
                }

                // FIXME support non recursive despawns
                despawner.despawn(target);
            }

            if let Some(mut noi) = noi {
                for source in T::TargetRestriction::noi_iter(&mut noi.0) {
                    let mut source = world.entity_mut(source);
                    let mut rel = source.get_mut::<Relation<T>>().unwrap();

                    if T::SourceRestriction::remove_rel(&mut rel.0, e) {
                        source.remove::<Relation<T>>();
                    }
                }
            }
        }
    }
}

trait EntityMutExtInternal {
    fn get_or_insert_with<T: Component>(&mut self, with: impl FnOnce() -> T) -> Mut<'_, T>;
}
impl EntityMutExtInternal for EntityMut<'_> {
    fn get_or_insert_with<T: Component>(&mut self, with: impl FnOnce() -> T) -> Mut<'_, T> {
        if let None = self.get_mut::<T>() {
            self.insert(with());
        }

        self.get_mut::<T>().unwrap()
    }
}

pub trait EntityRefExt {
    fn get_relation<T: RelKind>(&self) -> Option<RelationRefItem<'_, T>>;
    fn get_noitaler<T: RelKind>(&self) -> Option<NoitalerRefItem<'_, T>>;
}
pub trait EntityMutExt {
    fn get_relation_mut<T: RelKind>(&mut self) -> Option<RelationMutItem<'_, T>>;

    // FIXME it'd be nice if relation insert/removes could just be bundles and use "normal" apis.
    // unfortuantly bevy's `Bundle` is good for little more than "set of component types" so it is
    // not useful for us...
    fn insert_relation<T: RelKind>(&mut self, data: T, target: Entity) -> &mut Self;
    fn remove_relation<T: RelKind>(&mut self, target: Entity) -> &mut Self;
}
impl EntityRefExt for EntityRef<'_> {
    fn get_relation<T: RelKind>(&self) -> Option<RelationRefItem<'_, T>> {
        Some(RelationRefItem {
            inner: self.get::<Relation<T>>()?,
        })
    }

    fn get_noitaler<T: RelKind>(&self) -> Option<NoitalerRefItem<'_, T>> {
        Some(NoitalerRefItem {
            inner: self.get::<Noitaler<T>>()?,
        })
    }
}
impl EntityRefExt for EntityMut<'_> {
    fn get_relation<T: RelKind>(&self) -> Option<RelationRefItem<'_, T>> {
        Some(RelationRefItem {
            inner: self.get::<Relation<T>>()?,
        })
    }

    fn get_noitaler<T: RelKind>(&self) -> Option<NoitalerRefItem<'_, T>> {
        Some(NoitalerRefItem {
            inner: self.get::<Noitaler<T>>()?,
        })
    }
}
impl EntityMutExt for EntityMut<'_> {
    fn get_relation_mut<T: RelKind>(&mut self) -> Option<RelationMutItem<'_, T>> {
        Some(RelationMutItem {
            inner: self.get_mut::<Relation<T>>()?,
        })
    }

    fn insert_relation<T: RelKind>(&mut self, data: T, target_id: Entity) -> &mut Self {
        let source_id = self.id();
        self.world_scope(|world| {
            let mut source = world.entity_mut(source_id);
            let opt_remove_target = match source.get_mut::<Relation<T>>() {
                None => {
                    source.insert(Relation::<T>(T::SourceRestriction::make_rel_storage(
                        data, target_id,
                    )));
                    None
                }
                Some(mut rel) => T::SourceRestriction::push_rel(&mut rel.0, data, target_id),
            };

            if let Some(remove_target) = opt_remove_target {
                let mut remove_target = world.entity_mut(remove_target);
                let mut noi = remove_target.get_mut::<Noitaler<T>>().unwrap();
                if T::TargetRestriction::remove_noi(&mut noi.0, source_id) {
                    remove_target.remove::<Noitaler<T>>();
                }
            }

            let mut target = world.entity_mut(target_id);
            let opt_remove_source = match target.get_mut::<Noitaler<T>>() {
                None => {
                    target.insert(Noitaler::<T>(T::TargetRestriction::make_noi_storage(
                        source_id,
                    )));
                    None
                }
                Some(mut noi) => T::TargetRestriction::push_noi(&mut noi.0, source_id),
            };

            if let Some(remove_source) = opt_remove_source {
                let mut remove_source = world.entity_mut(remove_source);
                let mut rel = remove_source.get_mut::<Relation<T>>().unwrap();
                if T::SourceRestriction::remove_rel(&mut rel.0, target_id) {
                    remove_source.remove::<Relation<T>>();
                }
            }

            if let Err(()) = T::Cyclicity::assert_cyclicity(world.entity_mut(source_id)) {
                panic!(
                    "Attempting to insert relation `{:?}` -> {} -> `{:?}` introduces a cycle.",
                    source_id,
                    std::any::type_name::<T>(),
                    target_id
                );
            }
        });
        self
    }

    fn remove_relation<T: RelKind>(&mut self, remove_target: Entity) -> &mut Self {
        let source_id = self.id();

        self.world_scope(|w| {
            let mut source = w.entity_mut(source_id);
            if let Some(mut source_rel) = source.get_mut::<Relation<T>>() {
                if T::SourceRestriction::remove_rel(&mut source_rel.0, remove_target) {
                    source.remove::<Relation<T>>();
                }
            }

            let mut target = w.entity_mut(remove_target);
            if let Some(mut target_rel) = target.get_mut::<Noitaler<T>>() {
                if T::TargetRestriction::remove_noi(&mut target_rel.0, source_id) {
                    target.remove::<Noitaler<T>>();
                }
            }
        });

        self
    }
}

pub mod commands {
    use super::{Command, Entity, EntityMutExt, RelKind, World};
    use std::marker::PhantomData;

    pub struct InsertRelation<T: RelKind> {
        source: Entity,
        data: T,
        target: Entity,
    }
    impl<T: RelKind> Command for InsertRelation<T> {
        fn write(self, world: &mut World) {
            world
                .entity_mut(self.source)
                .insert_relation(self.data, self.target);
        }
    }

    pub struct RemoveRelation<T: RelKind> {
        source: Entity,
        target: Entity,
        _p: PhantomData<T>,
    }
    impl<T: RelKind> Command for RemoveRelation<T> {
        fn write(self, world: &mut World) {
            world
                .entity_mut(self.source)
                .remove_relation::<T>(self.target);
        }
    }
}

mod world_queries {
    use crate::{Noitaler, RelKind, Relation};
    use bevy::ecs::query::WorldQuery;

    #[derive(WorldQuery)]
    #[world_query(mutable)]
    pub struct RelationMut<T: RelKind> {
        pub(crate) inner: &'static mut Relation<T>,
    }

    #[derive(WorldQuery)]
    pub struct NoitalerRef<T: RelKind> {
        pub(crate) inner: &'static Noitaler<T>,
    }
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::restriction::Many {}
    impl Sealed for super::restriction::One {}
    impl Sealed for super::cyclicity::Cyclic {}
    impl Sealed for super::cyclicity::Acyclic {}
}
