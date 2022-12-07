use bevy::prelude::Entity;

use crate::RelKind;

pub struct One;
pub struct Many;

pub trait Restriction<T: RelKind>: crate::sealed::Sealed {
    #[doc(hidden)]
    type RelStorage: Send + Sync + 'static;
    #[doc(hidden)]
    type NoiStorage: Send + Sync + 'static;
    #[doc(hidden)]
    fn make_rel_storage(data: T, target: Entity) -> Self::RelStorage;
    #[doc(hidden)]
    fn make_noi_storage(target: Entity) -> Self::NoiStorage;
    #[doc(hidden)]
    fn push_rel(rel: &mut Self::RelStorage, data: T, target: Entity) -> Option<Entity>;
    #[doc(hidden)]
    fn push_noi(noi: &mut Self::NoiStorage, target: Entity) -> Option<Entity>;
    #[doc(hidden)]
    fn remove_rel(rel: &mut Self::RelStorage, target: Entity) -> bool;
    #[doc(hidden)]
    fn remove_noi(noi: &mut Self::NoiStorage, target: Entity) -> bool;

    #[doc(hidden)]
    type RelDataIterMut<'a>: Iterator<Item = &'a mut T>;
    #[doc(hidden)]
    type RelDataIter<'a>: Iterator<Item = &'a T>;
    #[doc(hidden)]
    type RelTargetIter<'a>: Iterator<Item = Entity>;
    #[doc(hidden)]
    fn rel_iter_mut(
        rel: &mut Self::RelStorage,
    ) -> (Self::RelDataIterMut<'_>, Self::RelTargetIter<'_>);
    #[doc(hidden)]
    fn rel_iter(rel: &Self::RelStorage) -> (Self::RelDataIter<'_>, Self::RelTargetIter<'_>);

    #[doc(hidden)]
    type NoiTargetIter<'a>: Iterator<Item = Entity>;
    #[doc(hidden)]
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
