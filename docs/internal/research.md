---
id: research
title: "Academic Research Angles"
type: guide
created_date: '2026-05-24'
---

# Academic Research Angles

This document maps the project's potential academic contributions honestly: what is genuinely novel, what is prior art, what needs to exist before each angle is publishable, and what venues are realistic. It is written for the author as a researcher, not as a language designer.

Read this alongside `docs/internal/vision.md`, which covers the product identity. This document is about intellectual contribution.

---

## Prior Work and Honest Positioning

Before stating novelty claims, the prior work must be acknowledged precisely. Overstating novelty is the fastest way to get a paper rejected.

### Linear types

Linear types originate in Girard's linear logic (1987). Their application to programming languages has a long history:

| Work | What it does | What it doesn't do |
|---|---|---|
| Wadler (1990) — *Linear types can change the world* | Applies linear logic to functional programming | No opt-in; no RC coexistence; no dual-mode |
| Baker (1992) — *Lively Linear Lisp* | Linear types for garbage-collection-free Lisp | Dynamically typed; no static type system |
| Cyclone (2002) | Region types + linear types in a C-like language | Compiled only; mandatory; complex annotation burden |
| Rust (2010–) | Affine types via ownership + borrow checker | Mandatory; no interpreter; borrow checker required |
| Linear Haskell — Bernardy et al. (2018) | Opt-in linear types in a lazy functional language | No RC coexistence; no dual-mode; no Rust-like syntax; GHC-specific |
| Austral (2022) | Mandatory linear types, formally specified, compiled | Mandatory; compiled only; no RC fallback |

**What Gust adds to this space:** opt-in linear types in an RC-default language, with a specific mechanism for inspection without consumption (`&T`, expression-only) that avoids lifetime annotations, in a dual-mode (interpreted + compiled) setting. No prior work combines all three of these properties.

### Reference-counted memory management

RC is well understood and widely implemented (Swift ARC, Python CPython, Rust `Rc<T>`). There is no novelty claim here. Gust's RC runtime is prior art by design.

### Dual-mode languages (interpreter + compiler)

| Language | Dual-mode | Linear types | Rust-like |
|---|---|---|---|
| OCaml | Yes (bytecode + native) | No | No |
| Julia | JIT only (not interpreter + compiler) | No | No |
| Kotlin | JVM + Native + JS | No | No |
| Haskell (GHC + GHCi) | Yes | Linear Haskell (limited) | No |
| **Gust** | **Yes** | **Yes (opt-in)** | **Yes** |

The column combination is unoccupied. This is a real gap, but a gap in the design space does not by itself constitute a research contribution — it must be paired with a formal result or empirical study.

### Type inference

Hindley-Milner is textbook (Hindley 1969, Milner 1978, Damas-Milner 1982). No novelty claim. Gust's type inference is an application of existing theory.

---

## Novelty Claims

Three claims are defensible. They are ranked by confidence and by how much work is required before publication.

---

### Claim 1 — Formal soundness of expression-only `&T` in an opt-in linear type system

**Confidence: High. Prerequisites: interpreter only.**

#### What the claim is

RFC-0024 introduces `&T` — a non-owning read reference that is valid only in expression position. It cannot be bound to a `let`, stored in a struct field, or appear in a function return type. This restriction is specifically designed to make linear types inspectable without consumption, without requiring lifetime annotations.

The claim: this restricted reference form, combined with the opt-in linear type system (the linearity environment, branch consistency rules, loop restrictions), is sound — it does not admit use-after-free, double-free, or resource leak in the linear subset of the language.

#### Why this is novel

- Linear Haskell's opt-in mechanism uses a different approach (linearity-polymorphic function types). It does not use a restricted reference form and does not address the inspection-without-consumption problem in the same way.
- Rust addresses inspection via the full borrow checker with lifetime annotations. Gust's `&T` is strictly weaker but also strictly simpler — the question is whether it is *sufficient* and under what conditions it fails.
- The formal characterization of what programs `&T` accepts vs rejects compared to full lifetime tracking is an open question. A paper that answers this precisely — with a proof of soundness and a proof of incompleteness (programs that are safe but `&T` rejects) — is a self-contained contribution.

#### What needs to exist

- A formal definition of the linear type system: linearity environment, judgment rules for `&T`, branch consistency, loop restriction.
- A soundness theorem: well-typed programs with linear types do not exhibit use-after-free or double-free.
- A proof, ideally mechanized (Lean 4 or Coq).
- An incompleteness characterization: examples of safe programs `&T` cannot express, requiring either lifetime annotations or a consuming-and-returning style.

#### What does not need to exist

- The compiler. This is purely about the static type system, which runs identically in both modes.
- The full language. The paper can be scoped to the linear fragment: `linear struct`, `&T`, `drop`, branching, and loops.

#### Realistic venues

- **ICFP** (International Conference on Functional Programming) — if the paper emphasizes the type-theoretic contribution
- **POPL** (Principles of Programming Languages) — if the proof is mechanized and the theorem is strong
- **ECOOP** (European Conference on Object-Oriented Programming) — if the framing emphasizes the language design tradeoffs
- **Onward!** — if the framing emphasizes the design experiment and the `&T` tradeoff analysis without a full proof

---

### Claim 2 — Formal model of RC/linear coexistence

**Confidence: Medium. Prerequisites: interpreter only, but requires the memory model RFC cluster to be resolved.**

#### What the claim is

Gust mixes two memory management disciplines in a single program: RC-managed values (the default) and linearly-managed values (opt-in). The boundary between them has specific rules:

- `Arc<LinearT>` and `Rc<LinearT>` are forbidden.
- `*T` and `*mut T` (RFC-0001) cannot point to linear values.
- A linear value that is `Send` can move through a channel (consumption = send).
- `unsafe` blocks relax linearity at programmer assertion.

The claim: these boundary rules form a coherent, sound system. A linear value cannot become RC-managed (no aliasing via reference counting); an RC value cannot be treated as linear (no false single-ownership claims); the unsafe escape hatch is the only way to cross the boundary, making unsafe surface auditable.

#### Why this is novel

Linear Haskell's linear types coexist with non-linear Haskell values, but Haskell has no RC — the coexistence boundary is different. Rust's `Rc<T>` and owned values coexist, but ownership is mandatory — there is no "RC by default, linear by opt-in" framing. The specific combination has not been formally analyzed.

The `Send` interaction is particularly underexplored: a linear `T` that is `Send` can move through a channel as a consumption event. This maps linear type theory onto Go-style channel semantics in a way that has not appeared in the literature.

#### What needs to exist

- Resolution of the memory model RFC cluster (RFC-0001, RFC-0003, RFC-0006, RFC-0024 interactions) — the boundary rules must be decided before they can be formalized.
- A formal type system that includes both RC-managed and linear-managed values, with explicit boundary rules.
- A soundness theorem: no program can have both RC and linear access to the same value simultaneously (no aliasing violation).
- The `unsafe` escape hatch must be formally characterized as the boundary crossing mechanism.

#### What does not need to exist

- The compiler.
- A mechanized proof (though it would strengthen the paper significantly).

#### Realistic venues

- **PLDI** (Programming Language Design and Implementation) — if the paper includes an implementation evaluation
- **ICFP** — if the type-theoretic contribution is the focus
- **OOPSLA** — if the practical programming model is the focus

---

### Claim 3 — Dual-mode semantic equivalence for a linear type system

**Confidence: Lower. Prerequisites: both backends (interpreter + compiler) must exist.**

#### What the claim is

The interpreter and native compiler implement identical semantics for the linear type system. A program that is accepted by the linearity checker in one mode is accepted in the other; a program that is rejected in one mode is rejected in the other; and for all accepted programs, observable behaviour is identical.

This is a semantic equivalence theorem between two implementations of the same specification. It is also a validation of the specification's precision: if the spec is ambiguous enough to be implemented differently by two independent backends, the divergence reveals a spec bug.

#### Why this is novel

Most dual-mode languages (OCaml, Haskell/GHCi) do not have a linear type system. Most linear type system implementations are single-backend. The combination of linear type system + dual-mode semantic equivalence proof is not in the literature.

The research question has a practical engineering dimension that is also publishable: what spec precision is required to implement a linear type system twice, independently, and have the implementations agree? This is a question about specification methodology, not just type theory.

#### What needs to exist

- A production-quality interpreter (not just a tree-walk PoC — the formal equivalence claim is only meaningful if both backends are complete).
- A native compiler implementing the same spec version.
- A formal semantics for both backends (small-step operational semantics or denotational).
- A bisimulation or logical relations proof connecting the two.
- A cross-backend test suite demonstrating empirical agreement (not a substitute for the proof, but necessary supporting evidence).

#### What does not need to exist

- v1.0. The claim can be scoped to a well-defined language subset (e.g. the linear fragment without concurrency).

#### Realistic venues

- **POPL** — if the proof is mechanized and the theorem is strong
- **PLDI** — if the implementation and testing methodology is emphasized
- **Formal Methods conferences** — if the focus is on specification methodology

---

## What is Explicitly Not a Novelty Claim

The following are prior art and should not be framed as contributions:

- Hindley-Milner type inference
- Reference counting as a memory management strategy
- Algebraic data types and exhaustive pattern matching
- Expression-oriented syntax
- The Rust-like surface syntax
- Fibers and channel-based concurrency (Go model)
- The `Perhaps<T>` / `Result<T, E>` error handling pattern

Claiming novelty for any of the above would be a rejection trigger at any serious venue.

---

## What Would Make the Project More Academically Compelling

Ranked by effort-to-impact ratio:

**1. Formalize the linear type system now (high impact, moderate effort)**

Write down the inference rules for the linearity environment formally — judgment forms, typing rules for `&T`, branch merge rules, loop restriction — and publish them in the spec or a companion document. This costs nothing in implementation terms and immediately makes Claim 1 pursuable. The rules are already described in prose in RFC-0024; translating them to formal notation is the work.

**2. Write the incompleteness analysis for `&T` (high impact, low effort)**

Identify and document programs that are safe but that `&T` cannot express without a consuming-and-returning style. This requires no implementation — it is a design analysis. It makes the tradeoff between `&T` and full lifetime tracking precise and gives the paper a concrete negative result to go alongside the soundness claim.

**3. Resolve the memory model RFC cluster (enables Claim 2)**

RFC-0001, RFC-0003, RFC-0006, RFC-0024 must be decided before the RC/linear coexistence boundary can be formalized. The decisions are implementation prerequisites regardless; the academic contribution comes for free once they are made and written up formally.

**4. Mechanize the proof (dramatically increases publishability)**

A soundness proof in prose is reviewable but contestable. A mechanized proof in Lean 4 or Coq is much harder to reject. Lean 4 is recommended over Coq for this project's design space — its dependent type theory is a good fit for the linear type system's structural rules, and its syntax is closer to functional programming than Coq's tactic-heavy style.

**5. Start a compiler (enables Claim 3, years away)**

Claim 3 requires both backends. The compiler is not imminent. This is the lowest-priority academic action.

---

## Publication Strategy

The realistic publication path, given that both backends will not exist for several years:

**Short term (Claim 1 + incompleteness analysis):**
Write a paper on the `&T` design: formal system, soundness proof, incompleteness characterization, comparison with lifetime-based approaches. This can be done with the interpreter alone. Target: Onward! or ECOOP for a first submission (more design-focused, more accepting of work-in-progress); ICFP or POPL for a version with a mechanized proof.

**Medium term (Claim 2, after RFC cluster resolution):**
Extend the paper or write a follow-on covering RC/linear coexistence and the `Send` interaction. This builds on Claim 1's formal system. Target: PLDI or OOPSLA.

**Long term (Claim 3, after compiler exists):**
Dual-mode equivalence. This is a dissertation-level contribution if mechanized. Target: POPL or a formal methods venue.

---

## References

- Project vision: `docs/internal/vision.md`
- RFC-0024: `docs/internal/rfcs/rfc-0024-linear-types.md` — linear type system design
- RFC cluster report: `docs/internal/rfc-cluster-memory-model.md` — RC/linear coexistence rules
- Bernardy et al. (2018) — *Linear Haskell: practical linearity in a higher-order polymorphic language*. POPL 2018.
- Girard (1987) — *Linear logic*. Theoretical Computer Science 50(1).
- Walker (2005) — *Substructural type systems*. In Pierce (ed.), *Advanced Topics in Types and Programming Languages*. MIT Press. (Best accessible survey of linear, affine, and relevant type systems.)
- Wadler (1990) — *Linear types can change the world*. IFIP TC 2 Working Conference on Programming Concepts and Methods.
- Cyclone: Grossman et al. (2002) — *Region-based memory management in Cyclone*. PLDI 2002.
- Austral specification: https://austral-lang.org/spec/spec.html
