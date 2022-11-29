use std::marker::PhantomData;

use bevy::ecs::{
    component::TableStorage,
    prelude::*,
    query::WorldQuery,
    system::Command,
    world::{EntityMut, EntityRef},
};

pub struct One;
pub struct Many;

pub struct Cyclic;
pub struct Acyclic;

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Many {}
    impl Sealed for super::One {}
    impl Sealed for super::Cyclic {}
    impl Sealed for super::Acyclic {}
}

pub trait Restriction<T: RelKind>: sealed::Sealed {
    type RelStorage: Send + Sync + 'static;
    type NoiStorage: Send + Sync + 'static;
    fn make_rel_storage(data: T, target: Entity) -> Self::RelStorage;
    fn make_noi_storage(target: Entity) -> Self::NoiStorage;
    fn push_rel(rel: &mut Self::RelStorage, data: T, target: Entity) -> Option<Entity>;
    fn push_noi(noi: &mut Self::NoiStorage, target: Entity) -> Option<Entity>;
    fn remove_rel(rel: &mut Self::RelStorage, target: Entity) -> bool;
    fn remove_noi(noi: &mut Self::NoiStorage, target: Entity) -> bool;

    type RelDataIterMut<'a>: Iterator<Item = &'a mut T>;
    type RelDataIter<'a>: Iterator<Item = &'a T>;
    type RelTargetIter<'a>: Iterator<Item = Entity>;
    fn rel_iter_mut(
        rel: &mut Self::RelStorage,
    ) -> (Self::RelDataIterMut<'_>, Self::RelTargetIter<'_>);
    fn rel_iter(rel: &Self::RelStorage) -> (Self::RelDataIter<'_>, Self::RelTargetIter<'_>);

    type NoiTargetIter<'a>: Iterator<Item = Entity>;
    fn noi_iter(noi: &Self::NoiStorage) -> Self::NoiTargetIter<'_>;
}
impl<T: RelKind> Restriction<T> for Many {
    type RelStorage = (Vec<T>, Vec<Entity>);
    type NoiStorage = Vec<Entity>;

    fn push_rel(rel: &mut (Vec<T>, Vec<Entity>), data: T, target: Entity) -> Option<Entity> {
        match rel.1.iter().position(|target2| *target2 == target) {
            Some(pos) => rel.0[pos] = data,
            None => {
                rel.0.push(data);
                rel.1.push(target);
            }
        }
        None
    }

    fn push_noi(noi: &mut Vec<Entity>, target: Entity) -> Option<Entity> {
        if let None = noi.iter().find(|target2| **target2 == target) {
            noi.push(target);
        }
        None
    }

    fn make_rel_storage(data: T, target: Entity) -> Self::RelStorage {
        (vec![data], vec![target])
    }

    fn make_noi_storage(target: Entity) -> Self::NoiStorage {
        vec![target]
    }

    fn remove_rel(rel: &mut (Vec<T>, Vec<Entity>), target: Entity) -> bool {
        if rel.0.len() == 1 {
            return true;
        }

        let pos = rel.1.iter().position(|target2| *target2 == target).unwrap();
        rel.0.swap_remove(pos);
        rel.1.swap_remove(pos);

        false
    }

    fn remove_noi(noi: &mut Vec<Entity>, target: Entity) -> bool {
        if noi.len() == 1 {
            return true;
        }

        let pos = noi.iter().position(|target2| *target2 == target).unwrap();
        noi.swap_remove(pos);

        false
    }

    type RelDataIterMut<'a> = std::slice::IterMut<'a, T>;
    type RelDataIter<'a> = std::slice::Iter<'a, T>;
    type RelTargetIter<'a> = std::iter::Copied<std::slice::Iter<'a, Entity>>;
    fn rel_iter_mut(
        rel: &mut Self::RelStorage,
    ) -> (Self::RelDataIterMut<'_>, Self::RelTargetIter<'_>) {
        (rel.0.iter_mut(), rel.1.iter().copied())
    }
    fn rel_iter(rel: &Self::RelStorage) -> (Self::RelDataIter<'_>, Self::RelTargetIter<'_>) {
        (rel.0.iter(), rel.1.iter().copied())
    }

    type NoiTargetIter<'a> = std::iter::Copied<std::slice::Iter<'a, Entity>>;
    fn noi_iter(noi: &Self::NoiStorage) -> Self::NoiTargetIter<'_> {
        noi.iter().copied()
    }
}
impl<T: RelKind> Restriction<T> for One {
    type RelStorage = (T, Entity);
    type NoiStorage = Entity;
    fn push_rel(rel: &mut (T, Entity), data: T, target: Entity) -> Option<Entity> {
        match rel.1 == target {
            true => {
                rel.0 = data;
                None
            }
            false => {
                let old_target = rel.1;
                // drop/panic safety?
                *rel = (data, target);
                Some(old_target)
            }
        }
    }
    fn push_noi(noi: &mut Entity, target: Entity) -> Option<Entity> {
        match *noi == target {
            true => None,
            false => {
                let old_target = *noi;
                *noi = target;
                Some(old_target)
            }
        }
    }

    fn make_rel_storage(data: T, target: Entity) -> (T, Entity) {
        (data, target)
    }

    fn make_noi_storage(target: Entity) -> Entity {
        target
    }

    fn remove_rel(_rel: &mut (T, Entity), _target: Entity) -> bool {
        true
    }

    fn remove_noi(_noi: &mut Entity, _target: Entity) -> bool {
        true
    }

    type RelDataIterMut<'a> = std::iter::Once<&'a mut T>;
    type RelDataIter<'a> = std::iter::Once<&'a T>;
    type RelTargetIter<'a> = std::iter::Once<Entity>;
    fn rel_iter_mut(
        rel: &mut Self::RelStorage,
    ) -> (Self::RelDataIterMut<'_>, Self::RelTargetIter<'_>) {
        (std::iter::once(&mut rel.0), std::iter::once(rel.1))
    }
    fn rel_iter(rel: &Self::RelStorage) -> (Self::RelDataIter<'_>, Self::RelTargetIter<'_>) {
        (std::iter::once(&rel.0), std::iter::once(rel.1))
    }

    type NoiTargetIter<'a> = std::iter::Once<Entity>;
    fn noi_iter(noi: &Self::NoiStorage) -> Self::NoiTargetIter<'_> {
        std::iter::once(*noi)
    }
}

pub trait Cyclicity: sealed::Sealed {}
impl Cyclicity for Cyclic {}
impl Cyclicity for Acyclic {}

pub trait RelKind: Sized + Send + Sync + 'static {
    /// Number of relations of kind `Self` allowed on a source entity
    type SourceRestriction: Restriction<Self>;
    /// Number of relations of kind `Self` allowed to point to a target entity
    type TargetRestriction: Restriction<Self>;

    /// Whether cycles are allowed in the graph created by edges of `Self`. This can only be
    /// set to [`Acyclic`] if either [`Self::SourceRestriction`] or [`Self::TargetRestriction`] is set to [`One`]
    type Cyclicity: Cyclicity
        + AssertTreeIfAcyclic<Self, Self::SourceRestriction, Self::TargetRestriction>;
}

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

struct Relation<T: RelKind>(<T::SourceRestriction as Restriction<T>>::RelStorage);

impl<T: RelKind> Relation<T> {
    fn new(inner: <T::SourceRestriction as Restriction<T>>::RelStorage) -> Self {
        Self(inner)
    }

    fn get(&self) -> &<T::SourceRestriction as Restriction<T>>::RelStorage {
        &self.0
    }

    fn get_mut(&mut self) -> &mut <T::SourceRestriction as Restriction<T>>::RelStorage {
        &mut self.0
    }
}

#[derive(Component)]
struct Noitaler<T: RelKind>(<T::TargetRestriction as Restriction<T>>::NoiStorage);

impl<T: RelKind> Noitaler<T> {
    fn new(inner: <T::TargetRestriction as Restriction<T>>::NoiStorage) -> Self {
        Self(inner)
    }

    fn get(&self) -> &<T::TargetRestriction as Restriction<T>>::NoiStorage {
        &self.0
    }

    fn get_mut(&mut self) -> &mut <T::TargetRestriction as Restriction<T>>::NoiStorage {
        &mut self.0
    }
}

impl<T: RelKind> Component for Relation<T> {
    type Storage = TableStorage;

    fn despawn_hook() -> fn(Entity, &mut World, bevy::ecs::component::NestedDespawns<'_>)
    where
        Self: Sized,
    {
        |e, world, mut despawner| {
            let mut entity = world.entity_mut(e);
            let mut rel = entity.get_mut::<Relation<T>>().unwrap();
            for target in T::SourceRestriction::rel_iter(rel.get_mut()).1 {
                // FIXME support non recursive despawns
                despawner.despawn(target);
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
                    source.insert(Relation::<T>::new(T::SourceRestriction::make_rel_storage(
                        data, target_id,
                    )));
                    None
                }
                Some(mut rel) => T::SourceRestriction::push_rel(rel.get_mut(), data, target_id),
            };

            if let Some(remove_target) = opt_remove_target {
                let mut remove_target = world.entity_mut(remove_target);
                let mut noi = remove_target.get_mut::<Noitaler<T>>().unwrap();
                if T::TargetRestriction::remove_noi(noi.get_mut(), source_id) {
                    remove_target.remove::<Noitaler<T>>();
                }
            }

            let mut target = world.entity_mut(target_id);
            let opt_remove_source = match target.get_mut::<Noitaler<T>>() {
                None => {
                    target.insert(Noitaler::<T>::new(T::TargetRestriction::make_noi_storage(
                        source_id,
                    )));
                    None
                }
                Some(mut noi) => T::TargetRestriction::push_noi(noi.get_mut(), source_id),
            };

            if let Some(remove_source) = opt_remove_source {
                let mut remove_source = world.entity_mut(remove_source);
                let mut rel = remove_source.get_mut::<Relation<T>>().unwrap();
                if T::SourceRestriction::remove_rel(rel.get_mut(), target_id) {
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
                if T::SourceRestriction::remove_rel(source_rel.get_mut(), remove_target) {
                    source.remove::<Relation<T>>();
                }
            }

            let mut target = w.entity_mut(remove_target);
            if let Some(mut target_rel) = target.get_mut::<Noitaler<T>>() {
                if T::TargetRestriction::remove_noi(target_rel.get_mut(), source_id) {
                    target.remove::<Noitaler<T>>();
                }
            }
        });

        self
    }
}

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

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct RelationMut<T: RelKind> {
    inner: &'static mut Relation<T>,
}

pub type RelationRef<T> = <RelationMut<T> as WorldQuery>::ReadOnly;
pub type RelationRefItem<'w, T> = <RelationRef<T> as WorldQuery>::Item<'w>;

pub struct RelationIter<'a, T: RelKind> {
    targets: <T::SourceRestriction as Restriction<T>>::RelTargetIter<'a>,
    data: <T::SourceRestriction as Restriction<T>>::RelDataIter<'a>,
}
impl<'a, T: RelKind> Iterator for RelationIter<'a, T> {
    type Item = (Entity, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        Some((self.targets.next()?, self.data.next()?))
    }
}
impl<'a, T: RelKind> IntoIterator for &'a RelationMutReadOnlyItem<'_, T> {
    type Item = (Entity, &'a T);
    type IntoIter = RelationIter<'a, T>;

    fn into_iter(self) -> RelationIter<'a, T> {
        let (data, targets) = T::SourceRestriction::rel_iter(self.inner.get());
        RelationIter { targets, data }
    }
}
impl<T: RelKind> RelationMutReadOnlyItem<'_, T> {
    pub fn iter(&self) -> RelationIter<'_, T> {
        <&Self>::into_iter(self)
    }
}
impl<'a, T: RelKind> IntoIterator for &'a RelationMutItem<'_, T> {
    type Item = (Entity, &'a T);
    type IntoIter = RelationIter<'a, T>;

    fn into_iter(self) -> RelationIter<'a, T> {
        let (data, targets) = T::SourceRestriction::rel_iter(self.inner.get());
        RelationIter { targets, data }
    }
}
impl<T: RelKind> RelationMutItem<'_, T> {
    pub fn iter(&self) -> RelationIter<'_, T> {
        <&Self>::into_iter(self)
    }
}

pub struct RelationIterMut<'a, T: RelKind> {
    targets: <T::SourceRestriction as Restriction<T>>::RelTargetIter<'a>,
    data: <T::SourceRestriction as Restriction<T>>::RelDataIterMut<'a>,
}
impl<'a, T: RelKind> Iterator for RelationIterMut<'a, T> {
    type Item = (Entity, &'a mut T);
    fn next(&mut self) -> Option<(Entity, &'a mut T)> {
        Some((self.targets.next()?, self.data.next()?))
    }
}
impl<'a, T: RelKind> IntoIterator for &'a mut RelationMutItem<'_, T> {
    type Item = (Entity, &'a mut T);
    type IntoIter = RelationIterMut<'a, T>;

    fn into_iter(self) -> RelationIterMut<'a, T> {
        // FIXME: whoops all mutated
        let (data, targets) = T::SourceRestriction::rel_iter_mut(self.inner.get_mut());
        RelationIterMut { targets, data }
    }
}
impl<'a, T: RelKind> IntoIterator for RelationMutItem<'a, T> {
    type Item = (Entity, &'a mut T);
    type IntoIter = RelationIterMut<'a, T>;

    fn into_iter(self) -> RelationIterMut<'a, T> {
        // FIXME: whoops all mutated
        let inner = self.inner.into_inner();
        let (data, targets) = T::SourceRestriction::rel_iter_mut(inner.get_mut());
        RelationIterMut { targets, data }
    }
}
impl<T: RelKind> RelationMutItem<'_, T> {
    pub fn iter_mut(&mut self) -> RelationIterMut<'_, T> {
        <&mut Self>::into_iter(self)
    }
}

#[derive(WorldQuery)]
pub struct NoitalerRef<T: RelKind> {
    inner: &'static Noitaler<T>,
}

pub struct NoitalerIter<'a, T: RelKind> {
    targets: <T::TargetRestriction as Restriction<T>>::NoiTargetIter<'a>,
}
impl<'a, T: RelKind> Iterator for NoitalerIter<'a, T> {
    type Item = Entity;
    fn next(&mut self) -> Option<Self::Item> {
        self.targets.next()
    }
}
impl<'a, T: RelKind> IntoIterator for &'a NoitalerRefItem<'_, T> {
    type Item = Entity;
    type IntoIter = NoitalerIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        NoitalerIter {
            targets: T::TargetRestriction::noi_iter(self.inner.get()),
        }
    }
}
impl<'a, T: RelKind> IntoIterator for NoitalerRefItem<'a, T> {
    type Item = Entity;
    type IntoIter = NoitalerIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        NoitalerIter {
            targets: T::TargetRestriction::noi_iter(self.inner.get()),
        }
    }
}
impl<'a, T: RelKind> NoitalerRefItem<'a, T> {
    pub fn iter(&self) -> NoitalerIter<'_, T> {
        <&Self>::into_iter(self)
    }
}
