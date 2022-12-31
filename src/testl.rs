use bevy::ecs::{prelude::*, system::SystemState};

use crate::{
    cyclicity::{Acyclic, Cyclic},
    restriction::{Many, One},
    EntityMutExt, EntityRefExt, NoitalerRef, RelKind, RelationRef,
};

fn assert_relation_graph_good<R: RelKind>(world: &mut World) {
    let mut state = SystemState::<(
        Query<(Entity, RelationRef<R>)>,
        Query<(Entity, NoitalerRef<R>)>,
    )>::new(world);
    let (relations, noitalers) = state.get_mut(world);

    for (source, relations) in &relations {
        for (target, _) in relations {
            let die = || {
                panic!(
                    "entity: {:?} had relation: {} to target: {:?} which did not have an entry in `Noitaler`",
                    source,
                    std::any::type_name::<R>(),
                    target
                )
            };

            let (_, noitalers) = noitalers.get(target).unwrap_or_else(|_| die());
            if noitalers
                .into_iter()
                .all(|noitalers_source| noitalers_source != source)
            {
                die();
            }
        }
    }

    for (source, relations) in &relations {
        if relations.into_iter().next().is_none() {
            panic!(
                "entity: {:?} had relation: {} with an empty list of targets",
                source,
                std::any::type_name::<R>(),
            );
        }
    }

    for (target, noitalers) in &noitalers {
        for source in noitalers {
            let die = || {
                panic!(
                    "entity: {:?} had noitaler: {} to source: {:?} which did not have an entry in `Relation`",
                    target,
                    std::any::type_name::<R>(),
                    source,
                )
            };

            let (_, relations) = relations.get(source).unwrap_or_else(|_| die());
            if relations
                .into_iter()
                .all(|(relations_target, _)| relations_target != target)
            {
                die();
            }
        }
    }

    for (target, noitalers) in &noitalers {
        if noitalers.into_iter().next().is_none() {
            panic!(
                "entity: {:?} had noitaler: {} with an empty list of sources",
                target,
                std::any::type_name::<R>(),
            )
        };
    }

    let sources = relations
        .iter()
        .map(|(source, _)| source)
        .collect::<Vec<_>>();
    for source in sources {
        use crate::AssertTreeIfAcyclic;
        if R::Cyclicity::assert_cyclicity(world.entity_mut(source)).is_err() {
            panic!(
                "entity: {:?} participates in a cycle with relation kind: {} which disallows cycles",
                source,
                std::any::type_name::<R>(),
            );
        };
    }
}

macro_rules! test_world {
    ($($foo:tt)*) => {{
        let mut world = World::new();
        inner_maker!{ world; $($foo)* }
        world
    }};
}

macro_rules! inner_maker {
    (
        $world:ident;
        spawn: [$($entity:ident),*]
        $($foo:tt)*
    ) => {
        $(
            let $entity = $world.spawn(()).id();
            assert_relation_graph_good::<R>(&mut $world);
        )*
        inner_maker!{$world; $($foo)*}
    };
    (
        $world:ident;
        despawn: [$($entity:ident),*]
        $($foo:tt)*
    ) => {
        $(
            $world.despawn($entity);
            assert_relation_graph_good::<R>(&mut $world);
        )*
        inner_maker!{$world; $($foo)*}
    };
    (
        $world:ident;
        insert: {
            $($source:ident -> $target:ident,)*
        }
        $($foo:tt)*
    ) => {
        $(
            $world.entity_mut($source).insert_relation(R, $target);
            assert_relation_graph_good::<R>(&mut $world);
        )*
        inner_maker!{$world; $($foo)*}
    };
    (
        $world:ident;
        remove: {
            $($source:ident -> $target:ident,)*
        }
        $($foo:tt)*
    ) => {
        $(
            $world.entity_mut($source).remove_relation(R, $target);
            assert_relation_graph_good::<R>(&mut $world);
        )*
        inner_maker!{$world; $($foo)*}
    };
    (
        $world:ident;
        exists: {
            $($source:ident -> $target:ident,)*
        }
        $($foo:tt)*
    ) => {
        $(
            if let None = $world.entity($source).get_relation::<R>($target) {
                panic!("expected `{:?}` to have relation `{}` targetting `{:?}`", $source, std::any::type_name::<R>(), $target);
            }
            assert_relation_graph_good::<R>(&mut $world);
        )*
        inner_maker!{$world; $($foo)*}
    };
    (
        $world:ident;
        not_exists: {
            $($source:ident -> $target:ident,)*
        }
        $($foo:tt)*
    ) => {
        $(
            if let Some(_) = $world.entity($source).get_relation::<R>($target) {
                panic!("expected `{:?}` to not have relation `{}` targetting `{:?}`", $source, std::any::type_name::<R>(), $target);
            }
            assert_relation_graph_good::<R>(&mut $world);
        )*
        inner_maker!{$world; $($foo)*}
    };
    (
        $world:ident;
        alive: [$($entity:ident),*]
        $($foo:tt)*
    ) => {
        $(
            if let None = $world.get_entity($entity) {
                panic!("expected {:?} to be alive", $entity);
            }
            assert_relation_graph_good::<R>(&mut $world);
        )*
        inner_maker!{$world; $($foo)*}
    };
    (
        $world:ident;
        dead: [$($entity:ident),*]
        $($foo:tt)*
    ) => {
        $(
            if let Some(_) = $world.get_entity($entity) {
                panic!("expected {:?} to be dead", $entity);
            }
            assert_relation_graph_good::<R>(&mut $world);
        )*
        inner_maker!{$world; $($foo)*}
    };
    ( $world:ident; ) => {};
}

#[test]
fn cycle_despawn() {
    struct R;
    impl RelKind for R {
        type SourceRestriction = One;
        type TargetRestriction = One;
        type Cyclicity = Cyclic;
    }

    test_world! {
        spawn: [e0, e1, e2]

        insert: {
            e0->e1,
            e1->e2,
            e2->e0,
        }

        exists: {
            e0->e1,
            e1->e2,
            e2->e0,
        }

        despawn: [e2]
    };
}

#[test]
fn target_restriction() {
    struct R;
    impl RelKind for R {
        type SourceRestriction = Many;
        type TargetRestriction = One;
        type Cyclicity = Cyclic;
    }

    test_world! {
        spawn: [e0, e1, e2]

        insert: {
            e0->e1,
            e1->e2,
            e0->e2,
        }

        exists: {
            e0->e1,
            e0->e2,
        }

        not_exists: {
            e1->e2,
        }
    };
}

#[test]
fn source_restriction() {
    struct R;
    impl RelKind for R {
        type SourceRestriction = One;
        type TargetRestriction = Many;
        type Cyclicity = Cyclic;
    }

    test_world! {
        spawn: [e0, e1, e2]

        insert: {
            e1->e0,
            e2->e1,
            e2->e0,
        }

        exists: {
            e1->e0,
            e2->e0,
        }

        not_exists: {
            e2->e1,
        }
    };
}

#[test]
fn despawning_removes_noitaler() {
    struct R;
    impl RelKind for R {
        type SourceRestriction = One;
        type TargetRestriction = Many;
        type Cyclicity = Cyclic;
    }

    test_world! {
        spawn: [e0, e1, e2, e3]

        insert: {
            e0->e2,
            e1->e2,
            e2->e3,
        }

        exists: {
            e0->e2,
            e1->e2,
            e2->e3,
        }

        despawn: [e0]
        dead: [e0, e2, e3]
        alive: [e1]

        not_exists: {
            e1->e2,
        }
    };
}

#[test]
#[should_panic = "Attempting to insert relation"]
fn self_cycle() {
    struct R;
    impl RelKind for R {
        type SourceRestriction = One;
        type TargetRestriction = One;
        type Cyclicity = Acyclic;
    }

    test_world! {
        spawn: [e0]

        insert: {
            e0->e0,
        }
    };
}

#[test]
#[should_panic = "Attempting to insert relation"]
fn simple_cycle() {
    struct R;
    impl RelKind for R {
        type SourceRestriction = One;
        type TargetRestriction = One;
        type Cyclicity = Acyclic;
    }

    test_world! {
        spawn: [e0, e1]

        insert: {
            e0->e1,
            e1->e0,
        }
    };
}

#[test]
#[should_panic = "Attempting to insert relation"]
fn complex_cycle() {
    struct R;
    impl RelKind for R {
        type SourceRestriction = Many;
        type TargetRestriction = One;
        type Cyclicity = Acyclic;
    }

    test_world! {
        spawn: [e0, e1, e2, e3, e4]

        insert: {
            e0->e1,
            e0->e2,
            e2->e3,
            e2->e4,
            e3->e0,
        }
    };
}

#[test]
fn has_multiple_relations() {
    struct R;
    impl RelKind for R {
        type SourceRestriction = Many;
        type TargetRestriction = One;
        type Cyclicity = Acyclic;
    }

    test_world! {
        spawn: [e0, e1, e2]

        insert: {
            e0->e1,
            e0->e2,
        }

        exists: {
            e0->e1,
            e0->e2,
        }
    };
}
