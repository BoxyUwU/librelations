use bevy::ecs::prelude::*;

use crate::{
    cyclicity::{Acyclic, Cyclic},
    restriction::{Many, One},
    EntityMutExt, EntityRefExt, RelKind,
};

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
        $(let $entity = $world.spawn(()).id();)*
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
            match $world.entity($source).get_relation::<R>().and_then(|rels| rels.iter().find(|(t, _)| *t == $target).map(drop)) {
                None => panic!("expected `{:?}` to have relation `{}` targetting `{:?}`", $source, std::any::type_name::<R>(), $target),
                Some(_) => {},
            }
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
            match $world.entity($source).get_relation::<R>().and_then(|rels| rels.iter().find(|(t, _)| *t == $target).map(drop)) {
                None => {},
                Some(_) => panic!("expected `{:?}` to not have relation `{}` targetting `{:?}`", $source, std::any::type_name::<R>(), $target),
            }
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
        )*
        inner_maker!{$world; $($foo)*}
    };
    (
        $world:ident;
        alive: [$($entity:ident),*]
        $($foo:tt)*
    ) => {
        $(
            if $world.get_entity($entity).is_none() {
                panic!("expected {:?} to be alive", $entity);
            }
        )*
        inner_maker!{$world; $($foo)*}
    };
    (
        $world:ident;
        dead: [$($entity:ident),*]
        $($foo:tt)*
    ) => {
        $(
            if $world.get_entity($entity).is_some() {
                panic!("expected {:?} to be dead", $entity);
            }
        )*
        inner_maker!{$world; $($foo)*}
    };
    (
        $world:ident;
        custom: {
            $($code:stmt)*
        }
        $($foo:tt)*
    ) => {
        {$(
            $code
        )*}
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

// fn despawning_removes_noitaler() {
//     struct R;
//     impl RelKind for R {
//         type SourceRestriction = One;
//         type TargetRestriction = Many;
//         type Cyclicity = Cyclic;
//     }

//     test_world! {
//         spawn: [e0, e1, e2, e3]

//         insert: {
//             e0->e2,
//             e1->e2,
//             e2->e3,
//         }

//         exists: {
//             e0->e2,
//             e1->e2,
//             e2->e3,
//         }

//         despawn: [e0]
//         dead: [e0, e2, e3]
//         alive: [e1]

//         custom: {
//             world.entity(e0);
//         }
//     };
// }
