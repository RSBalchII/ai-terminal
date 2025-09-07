# TODO: Fix module import issues in layout/pane.rs

## Issue Description
There's an issue with importing `PaneManager` and `SplitOrientation` from the `layout::pane` module. 
The compiler reports that these items are not found in `layout::pane`, but they are clearly defined 
and marked as `pub` in the pane.rs file.

## Current Status
- Both `PaneManager` and `SplitOrientation` are defined in `/home/rsbiiw/projects/ai-terminal/terminal-ui/src/layout/pane.rs`
- Both items are marked as `pub`
- The mod.rs file correctly declares `pub mod pane;`
- The mod.rs file correctly tries to re-export with `pub use pane::{PaneManager, SplitOrientation};`
- Imports in lib.rs have been updated to use the correct paths

## Error Message
```
error[E0432]: unresolved imports `pane::PaneManager`, `pane::SplitOrientation`
  --> terminal-ui/src/layout/mod.rs:10:16
   |
10 | pub use pane::{PaneManager, SplitOrientation};
   |                ^^^^^^^^^^^  ^^^^^^^^^^^^^^^^ no `SplitOrientation` in `layout::pane`
   |                |
   |                no `PaneManager` in `layout::pane`
```

## Possible Causes
1. There might be a circular dependency or module resolution issue
2. The pane.rs file might have a syntax error that's preventing proper compilation
3. There might be an issue with how the modules are organized

## Next Steps
1. Review the pane.rs file for any syntax errors
2. Check for circular dependencies
3. Consider restructuring the module hierarchy if needed