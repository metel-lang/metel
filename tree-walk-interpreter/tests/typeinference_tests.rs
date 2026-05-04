/// Test suite for the type inference system.
/// Tests are organized by phase/component matching the task breakdown.

#[cfg(test)]
mod phase_1_type_variables {
    use yolang::typeinference::{TypeVar, TypeVarGenerator};

    #[test]
    fn test_type_var_creation() {
        let var1 = TypeVar(0);
        let var2 = TypeVar(1);

        assert_eq!(var1.0, 0);
        assert_eq!(var2.0, 1);
        assert_ne!(var1, var2);
    }

    #[test]
    fn test_type_var_display() {
        let var = TypeVar(42);
        assert_eq!(format!("{}", var), "?t42");
    }

    #[test]
    fn test_type_var_generator_fresh() {
        let mut gen = TypeVarGenerator::new();

        let v1 = gen.fresh();
        let v2 = gen.fresh();
        let v3 = gen.fresh();

        assert_eq!(v1.0, 0);
        assert_eq!(v2.0, 1);
        assert_eq!(v3.0, 2);
        assert_ne!(v1, v2);
        assert_ne!(v2, v3);
    }

    #[test]
    fn test_type_var_generator_counter() {
        let mut gen = TypeVarGenerator::new();
        assert_eq!(gen.counter(), 0);

        gen.fresh();
        assert_eq!(gen.counter(), 1);

        gen.fresh();
        gen.fresh();
        assert_eq!(gen.counter(), 3);
    }

    #[test]
    fn test_type_var_ordering() {
        let v0 = TypeVar(0);
        let v1 = TypeVar(1);
        let v5 = TypeVar(5);

        assert!(v0 < v1);
        assert!(v1 < v5);
        assert!(v0 < v5);
    }

    #[test]
    fn test_type_var_hashable() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(TypeVar(0));
        set.insert(TypeVar(1));
        set.insert(TypeVar(0));  // Duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&TypeVar(0)));
        assert!(set.contains(&TypeVar(1)));
        assert!(!set.contains(&TypeVar(2)));
    }
}

#[cfg(test)]
mod phase_2_infer_types {
    use yolang::typeinference::{InferType, TypeVar};
    use yolang::types::Type;

    #[test]
    fn test_concrete_variants() {
        assert_eq!(InferType::int(), InferType::Concrete(Type::Int));
        assert_eq!(InferType::float(), InferType::Concrete(Type::Float));
        assert_eq!(InferType::bool(), InferType::Concrete(Type::Bool));
        assert_eq!(InferType::str(), InferType::Concrete(Type::Str));
        assert_eq!(InferType::unit(), InferType::Concrete(Type::Unit));
    }

    #[test]
    fn test_var_constructor() {
        let v = TypeVar(3);
        assert_eq!(InferType::var(v), InferType::Var(TypeVar(3)));
    }

    #[test]
    fn test_display_concrete() {
        assert_eq!(format!("{}", InferType::int()), "Int");
        assert_eq!(format!("{}", InferType::float()), "Float");
        assert_eq!(format!("{}", InferType::bool()), "Bool");
        assert_eq!(format!("{}", InferType::str()), "String");
        assert_eq!(format!("{}", InferType::unit()), "()");
    }

    #[test]
    fn test_display_var() {
        assert_eq!(format!("{}", InferType::var(TypeVar(0))), "?t0");
        assert_eq!(format!("{}", InferType::var(TypeVar(7))), "?t7");
    }

    #[test]
    fn test_display_fun() {
        let ty = InferType::Fun(
            vec![InferType::int(), InferType::bool()],
            Box::new(InferType::str()),
        );
        assert_eq!(format!("{}", ty), "fun(Int, Bool) -> String");
    }

    #[test]
    fn test_display_fun_no_params() {
        let ty = InferType::Fun(vec![], Box::new(InferType::unit()));
        assert_eq!(format!("{}", ty), "fun() -> ()");
    }

    #[test]
    fn test_display_tuple() {
        let ty = InferType::Tuple(vec![InferType::int(), InferType::bool()]);
        assert_eq!(format!("{}", ty), "(Int, Bool)");
    }

    #[test]
    fn test_display_array() {
        let ty = InferType::Array(Box::new(InferType::int()));
        assert_eq!(format!("{}", ty), "Int[]");
    }

    #[test]
    fn test_display_named_no_args() {
        let ty = InferType::Named("Foo".to_string(), vec![]);
        assert_eq!(format!("{}", ty), "Foo");
    }

    #[test]
    fn test_display_named_with_args() {
        let ty = InferType::Named("Map".to_string(), vec![InferType::str(), InferType::int()]);
        assert_eq!(format!("{}", ty), "Map<String, Int>");
    }

    #[test]
    fn test_display_with_type_vars() {
        let ty = InferType::Fun(
            vec![InferType::var(TypeVar(0))],
            Box::new(InferType::var(TypeVar(0))),
        );
        assert_eq!(format!("{}", ty), "fun(?t0) -> ?t0");
    }

    #[test]
    fn test_nested_types() {
        // Array of functions: fun(Int) -> Bool []
        let ty = InferType::Array(Box::new(InferType::Fun(
            vec![InferType::int()],
            Box::new(InferType::bool()),
        )));
        assert_eq!(format!("{}", ty), "fun(Int) -> Bool[]");
    }

    #[test]
    fn test_equality() {
        let a = InferType::Fun(
            vec![InferType::var(TypeVar(0))],
            Box::new(InferType::var(TypeVar(1))),
        );
        let b = InferType::Fun(
            vec![InferType::var(TypeVar(0))],
            Box::new(InferType::var(TypeVar(1))),
        );
        let c = InferType::Fun(
            vec![InferType::var(TypeVar(0))],
            Box::new(InferType::var(TypeVar(2))),
        );
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}

#[cfg(test)]
mod phase_3_substitution {
    use yolang::typeinference::{InferType, Substitution, TypeVar};

    #[test]
    fn test_bind_and_lookup() {
        let mut s = Substitution::new();
        s.bind(TypeVar(0), InferType::int());
        assert_eq!(s.lookup(TypeVar(0)), Some(&InferType::int()));
        assert_eq!(s.lookup(TypeVar(1)), None);
    }

    #[test]
    fn test_apply_concrete_unchanged() {
        let s = Substitution::new();
        assert_eq!(s.apply(&InferType::int()), InferType::int());
    }

    #[test]
    fn test_apply_resolves_var() {
        let mut s = Substitution::new();
        s.bind(TypeVar(0), InferType::int());
        assert_eq!(s.apply(&InferType::var(TypeVar(0))), InferType::int());
    }

    #[test]
    fn test_apply_unbound_var_unchanged() {
        let s = Substitution::new();
        assert_eq!(s.apply(&InferType::var(TypeVar(5))), InferType::var(TypeVar(5)));
    }

    #[test]
    fn test_apply_chains_transitively() {
        // ?t0 → ?t1, ?t1 → Int  ⟹  apply(?t0) = Int
        let mut s = Substitution::new();
        s.bind(TypeVar(0), InferType::var(TypeVar(1)));
        s.bind(TypeVar(1), InferType::int());
        assert_eq!(s.apply(&InferType::var(TypeVar(0))), InferType::int());
    }

    #[test]
    fn test_apply_nested_fun() {
        // fun(?t0) -> ?t1  with { ?t0→Bool, ?t1→Int }  ⟹  fun(Bool) -> Int
        let mut s = Substitution::new();
        s.bind(TypeVar(0), InferType::bool());
        s.bind(TypeVar(1), InferType::int());
        let ty = InferType::Fun(
            vec![InferType::var(TypeVar(0))],
            Box::new(InferType::var(TypeVar(1))),
        );
        let expected = InferType::Fun(vec![InferType::bool()], Box::new(InferType::int()));
        assert_eq!(s.apply(&ty), expected);
    }

    #[test]
    fn test_apply_array() {
        let mut s = Substitution::new();
        s.bind(TypeVar(0), InferType::str());
        let ty = InferType::Array(Box::new(InferType::var(TypeVar(0))));
        assert_eq!(s.apply(&ty), InferType::Array(Box::new(InferType::str())));
    }

    #[test]
    fn test_apply_tuple() {
        let mut s = Substitution::new();
        s.bind(TypeVar(0), InferType::int());
        s.bind(TypeVar(1), InferType::bool());
        let ty = InferType::Tuple(vec![InferType::var(TypeVar(0)), InferType::var(TypeVar(1))]);
        assert_eq!(s.apply(&ty), InferType::Tuple(vec![InferType::int(), InferType::bool()]));
    }

    #[test]
    fn test_apply_named() {
        let mut s = Substitution::new();
        s.bind(TypeVar(0), InferType::int());
        let ty = InferType::Named("List".to_string(), vec![InferType::var(TypeVar(0))]);
        assert_eq!(s.apply(&ty), InferType::Named("List".to_string(), vec![InferType::int()]));
    }

    #[test]
    fn test_compose_applies_other_to_self_values() {
        // s1: { ?t0 → ?t1 },  s2: { ?t1 → Int }
        // compose(s1, s2) should resolve ?t0 all the way to Int
        let mut s1 = Substitution::new();
        s1.bind(TypeVar(0), InferType::var(TypeVar(1)));
        let mut s2 = Substitution::new();
        s2.bind(TypeVar(1), InferType::int());

        let composed = s1.compose(&s2);
        assert_eq!(composed.apply(&InferType::var(TypeVar(0))), InferType::int());
        assert_eq!(composed.apply(&InferType::var(TypeVar(1))), InferType::int());
    }

    #[test]
    fn test_compose_self_wins_on_overlap() {
        // compose(s1, s2) means "apply s1 first, then s2".
        // s1: { ?t0 → Int },  s2: { ?t0 → Bool }
        // ?t0 → s2(s1(?t0)) = s2(Int) = Int  (s2 only binds vars, not concrete types)
        let mut s1 = Substitution::new();
        s1.bind(TypeVar(0), InferType::int());
        let mut s2 = Substitution::new();
        s2.bind(TypeVar(0), InferType::bool());

        let composed = s1.compose(&s2);
        assert_eq!(composed.apply(&InferType::var(TypeVar(0))), InferType::int());
    }
}

#[cfg(test)]
mod phase_4_unification {
    use yolang::typeinference::{unify, InferType, TypeVar};

    #[test]
    fn test_unify_identical_concrete() {
        let s = unify(&InferType::int(), &InferType::int()).unwrap();
        assert_eq!(s.apply(&InferType::int()), InferType::int());
    }

    #[test]
    fn test_unify_incompatible_concrete() {
        assert!(unify(&InferType::int(), &InferType::bool()).is_err());
        assert!(unify(&InferType::str(), &InferType::float()).is_err());
    }

    #[test]
    fn test_unify_var_with_concrete() {
        let s = unify(&InferType::var(TypeVar(0)), &InferType::int()).unwrap();
        assert_eq!(s.apply(&InferType::var(TypeVar(0))), InferType::int());
    }

    #[test]
    fn test_unify_concrete_with_var() {
        let s = unify(&InferType::bool(), &InferType::var(TypeVar(1))).unwrap();
        assert_eq!(s.apply(&InferType::var(TypeVar(1))), InferType::bool());
    }

    #[test]
    fn test_unify_var_with_var() {
        let s = unify(&InferType::var(TypeVar(0)), &InferType::var(TypeVar(1))).unwrap();
        // One of them should resolve to the other
        let result = s.apply(&InferType::var(TypeVar(0)));
        let other = s.apply(&InferType::var(TypeVar(1)));
        assert_eq!(result, other);
    }

    #[test]
    fn test_unify_var_with_itself() {
        let s = unify(&InferType::var(TypeVar(0)), &InferType::var(TypeVar(0))).unwrap();
        // Empty substitution — no binding needed
        assert_eq!(s.apply(&InferType::var(TypeVar(0))), InferType::var(TypeVar(0)));
    }

    #[test]
    fn test_unify_function_types() {
        // fun(?t0) -> ?t0  with  fun(Int) -> Int  => ?t0 = Int
        let a = InferType::Fun(
            vec![InferType::var(TypeVar(0))],
            Box::new(InferType::var(TypeVar(0))),
        );
        let b = InferType::Fun(vec![InferType::int()], Box::new(InferType::int()));
        let s = unify(&a, &b).unwrap();
        assert_eq!(s.apply(&InferType::var(TypeVar(0))), InferType::int());
    }

    #[test]
    fn test_unify_function_arity_mismatch() {
        let a = InferType::Fun(vec![InferType::int()], Box::new(InferType::bool()));
        let b = InferType::Fun(
            vec![InferType::int(), InferType::int()],
            Box::new(InferType::bool()),
        );
        assert!(unify(&a, &b).is_err());
    }

    #[test]
    fn test_unify_function_return_type() {
        // fun(Int) -> ?t0  with  fun(Int) -> Bool  => ?t0 = Bool
        let a = InferType::Fun(vec![InferType::int()], Box::new(InferType::var(TypeVar(0))));
        let b = InferType::Fun(vec![InferType::int()], Box::new(InferType::bool()));
        let s = unify(&a, &b).unwrap();
        assert_eq!(s.apply(&InferType::var(TypeVar(0))), InferType::bool());
    }

    #[test]
    fn test_unify_array_types() {
        // ?t0[]  with  Int[]  => ?t0 = Int
        let a = InferType::Array(Box::new(InferType::var(TypeVar(0))));
        let b = InferType::Array(Box::new(InferType::int()));
        let s = unify(&a, &b).unwrap();
        assert_eq!(s.apply(&InferType::var(TypeVar(0))), InferType::int());
    }

    #[test]
    fn test_unify_array_element_mismatch() {
        let a = InferType::Array(Box::new(InferType::int()));
        let b = InferType::Array(Box::new(InferType::bool()));
        assert!(unify(&a, &b).is_err());
    }

    #[test]
    fn test_unify_tuple_types() {
        // (?t0, Bool)  with  (Int, Bool)  => ?t0 = Int
        let a = InferType::Tuple(vec![InferType::var(TypeVar(0)), InferType::bool()]);
        let b = InferType::Tuple(vec![InferType::int(), InferType::bool()]);
        let s = unify(&a, &b).unwrap();
        assert_eq!(s.apply(&InferType::var(TypeVar(0))), InferType::int());
    }

    #[test]
    fn test_unify_tuple_length_mismatch() {
        let a = InferType::Tuple(vec![InferType::int()]);
        let b = InferType::Tuple(vec![InferType::int(), InferType::bool()]);
        assert!(unify(&a, &b).is_err());
    }

    #[test]
    fn test_unify_named_types() {
        // List<?t0>  with  List<Int>  => ?t0 = Int
        let a = InferType::Named("List".to_string(), vec![InferType::var(TypeVar(0))]);
        let b = InferType::Named("List".to_string(), vec![InferType::int()]);
        let s = unify(&a, &b).unwrap();
        assert_eq!(s.apply(&InferType::var(TypeVar(0))), InferType::int());
    }

    #[test]
    fn test_unify_named_type_name_mismatch() {
        let a = InferType::Named("List".to_string(), vec![InferType::int()]);
        let b = InferType::Named("Set".to_string(), vec![InferType::int()]);
        assert!(unify(&a, &b).is_err());
    }

    #[test]
    fn test_unify_incompatible_shapes() {
        // Fun vs Array
        let a = InferType::Fun(vec![InferType::int()], Box::new(InferType::bool()));
        let b = InferType::Array(Box::new(InferType::int()));
        assert!(unify(&a, &b).is_err());

        // Tuple vs Concrete
        assert!(unify(
            &InferType::Tuple(vec![InferType::int()]),
            &InferType::int()
        )
        .is_err());
    }

    #[test]
    fn test_occurs_check_array() {
        // ?t0 = ?t0[]  — should fail
        let a = InferType::var(TypeVar(0));
        let b = InferType::Array(Box::new(InferType::var(TypeVar(0))));
        assert!(unify(&a, &b).is_err());
    }

    #[test]
    fn test_occurs_check_function() {
        // ?t0 = fun(?t0) -> Int  — should fail
        let a = InferType::var(TypeVar(0));
        let b = InferType::Fun(vec![InferType::var(TypeVar(0))], Box::new(InferType::int()));
        assert!(unify(&a, &b).is_err());
    }

    #[test]
    fn test_unify_multi_var_function() {
        // fun(?t0, ?t1) -> ?t0  with  fun(Int, Bool) -> Int  => ?t0=Int, ?t1=Bool
        let a = InferType::Fun(
            vec![InferType::var(TypeVar(0)), InferType::var(TypeVar(1))],
            Box::new(InferType::var(TypeVar(0))),
        );
        let b = InferType::Fun(
            vec![InferType::int(), InferType::bool()],
            Box::new(InferType::int()),
        );
        let s = unify(&a, &b).unwrap();
        assert_eq!(s.apply(&InferType::var(TypeVar(0))), InferType::int());
        assert_eq!(s.apply(&InferType::var(TypeVar(1))), InferType::bool());
    }
}

// Placeholder for Phase 5 tests
#[cfg(test)]
mod phase_5_constraints {
    // TODO: Add constraint tests here
}

// Placeholder for Phase 6 tests
#[cfg(test)]
mod phase_6_type_schemes {
    // TODO: Add type scheme tests here
}

// Placeholder for Phase 7 tests
#[cfg(test)]
mod phase_7_infer_context {
    // TODO: Add inference context tests here
}
