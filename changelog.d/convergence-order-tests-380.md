### Added
#### Convergence-order tests (MMS) for plates, quads, shells and solids (#380, 2026-10-28)

Added systematic convergence-order and Method of Manufactured Solutions (MMS) verification suites in `engine/tests/convergence_orders.rs` covering all structural plate, shell, and solid element families under h-refinement:
- DKT thin plates: observed asymptotic convergence rate $p = 2.038$ matching theoretical $O(h^2)$ rate.
- MITC4 quad shells: observed asymptotic convergence rate $p = 2.007$ matching theoretical $O(h^2)$ rate.
- Curved 4-node shells: observed asymptotic convergence rate $p = 2.007$ matching theoretical $O(h^2)$ rate.
- SHB8-ANS solid shells: observed asymptotic convergence rate $p = 2.006$ matching theoretical $O(h^2)$ rate.
- MITC9 biquadratic quad shells: observed asymptotic convergence rate $p = 4.022$ matching theoretical quartic $O(h^4)$ rate.
