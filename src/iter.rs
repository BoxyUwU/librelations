use crate::{NoitalerRefItem, RelKind, RelationMutItem, RelationRefItem, Restriction};
use bevy::prelude::Entity;

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
impl<'a, T: RelKind> IntoIterator for &'a RelationRefItem<'_, T> {
    type Item = (Entity, &'a T);
    type IntoIter = RelationIter<'a, T>;

    fn into_iter(self) -> RelationIter<'a, T> {
        let (data, targets) = T::SourceRestriction::rel_iter(&self.inner.0);
        RelationIter { targets, data }
    }
}
impl<T: RelKind> RelationRefItem<'_, T> {
    pub fn iter(&self) -> RelationIter<'_, T> {
        <&Self>::into_iter(self)
    }
}
impl<'a, T: RelKind> IntoIterator for &'a RelationMutItem<'_, T> {
    type Item = (Entity, &'a T);
    type IntoIter = RelationIter<'a, T>;

    fn into_iter(self) -> RelationIter<'a, T> {
        let (data, targets) = T::SourceRestriction::rel_iter(&self.inner.0);
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
        let (data, targets) = T::SourceRestriction::rel_iter_mut(&mut self.inner.0);
        RelationIterMut { targets, data }
    }
}
impl<'a, T: RelKind> IntoIterator for RelationMutItem<'a, T> {
    type Item = (Entity, &'a mut T);
    type IntoIter = RelationIterMut<'a, T>;

    fn into_iter(self) -> RelationIterMut<'a, T> {
        // FIXME: whoops all mutated
        let inner = self.inner.into_inner();
        let (data, targets) = T::SourceRestriction::rel_iter_mut(&mut inner.0);
        RelationIterMut { targets, data }
    }
}
impl<T: RelKind> RelationMutItem<'_, T> {
    pub fn iter_mut(&mut self) -> RelationIterMut<'_, T> {
        <&mut Self>::into_iter(self)
    }
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
            targets: T::TargetRestriction::noi_iter(&self.inner.0),
        }
    }
}
impl<'a, T: RelKind> IntoIterator for NoitalerRefItem<'a, T> {
    type Item = Entity;
    type IntoIter = NoitalerIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        NoitalerIter {
            targets: T::TargetRestriction::noi_iter(&self.inner.0),
        }
    }
}
impl<'a, T: RelKind> NoitalerRefItem<'a, T> {
    pub fn iter(&self) -> NoitalerIter<'_, T> {
        <&Self>::into_iter(self)
    }
}
