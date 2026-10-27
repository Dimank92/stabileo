### BREAKING (results)
#### Design never on the unfactored loads, completed; a NaN check and a staleness that hold on the combination path (2026-10-26)

- **Slabs, walls and the steel optimiser no longer design over the unfactored solve**
  (W2-01, SA-2026-024, which #588 fixed for footings and joints). The floor run designed slab
  bending from the result on screen's mx and my, and the optimiser took it as its one
  combination. With combinations to design for and none to read, every design refuses and says
  why (`designCombinationsWithheld`): not solved, only SRSS or ABS active, or an active list that
  names no solved one. An empty active list now counts as unsolved: the single solve stood in for
  it and footings, joints and floors designed over it. RC design says the combinations are not
  solved instead of "Define at least one load combination". The message no longer tells to
  "Solve again (Calcular)", which did not help when the combinations were refused.
- **The combination path checks for NaN displacements** before it publishes the base, the cases
  and the combinations, as the single solve does; #588's manual solve checked only the single.
- **Every manual solve cancels the pending live calc** (an export, the re-run), not only Enter and
  the ribbon: a live calc firing mid-solve made it report "the model changed" for an untouched
  model, or published its single solve in place of the combinations.
- **A solve that left the combinations out keeps the project stale** from any entry point; a plain
  Calcular after the refused re-run stamped it current over the same single solve. In Basic 3D the
  combinations' failure is information, not an error, when the single solve stands in. (#649)
