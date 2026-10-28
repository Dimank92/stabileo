### Fixed
#### Exported files named after a translation key (2026-10-28)

A new, unnamed model was exported as `tabBarnewStructure.xlsx` (and the like): the model was named
before the application's dictionaries loaded. It is named «Nueva estructura» / «New structure»
from the start; a saved project that carries the old name takes the right one when the language changes.
