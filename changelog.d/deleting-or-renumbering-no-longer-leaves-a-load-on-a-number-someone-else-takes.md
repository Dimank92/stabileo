### Fixed
#### Deleting or renumbering no longer leaves a load on a number someone else takes (2026-10-21)

Audit 2026-10, W1-01 to W1-05. A load names a node, a member or a definition by number, and
numbers are reused.
- **A deleted floor-load definition takes its loads with it.** They were kept as plain loads and
  went on into every solve. `removeGroup` now drops the loads the definition wrote, so deleting it
  from the panel or as a group does the same; undo brings both back. A load from a definition the
  model never held (a link, a paste) is still kept as a plain load.
- **Deleting a member takes its tendon, and deleting a node takes its imposed displacement.**
  Each delete kept its own list of load types, and neither list had the newer types. They now
  read one table, `LOAD_TARGET`, of what each load type stands on. Before, the solve skipped the
  orphan tendon and said "solved", and the orphan displacement made every solve refuse, naming
  the deleted node.
- **The solve refuses a load on a member or a node the model does not have**, naming the load
  and the number (`orphanLoadRefusal`: the single solve, both combination solves, direct
  analysis). Such a load now comes only from an older file or a paste. Before, a member load
  like that was skipped without a word.
- **Renumbering refuses while a load names something the model does not have**
  (`refused: 'orphanLoads'`, by load). The renumber left an unknown number unchanged, so a
  deleted support's settlement ended up on another support and solved.
- **A project spectrum's number is never given again.** Numbers were the highest in use plus
  one, so deleting spectrum 2 and adding a new one gave 2 again, and a spectral case on the old
  spectrum solved on the new one's placeholder table. The counter is saved with the project
  (`nextId.spectrum`). Older files start it past every spectrum and every number a case uses. A
  spectrum is not deleted while a spectral case uses it; the panel names the cases.
