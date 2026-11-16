## #2043 - Open the installed example projects

**Chosen:** Add an "Open example" submenu to the File ribbon's Open list.
Read the `examples/` directory beside the executable.
List its direct regular `.camproj` files in name order.
Hide the submenu when the directory is absent or the app runs in a browser.
Show "No example projects" when the directory contains no project files.
Queue the existing `OpenRecent(Project, path)` command when the user selects a project.

**Why:** The File ribbon is the app's existing file menu.
The package location matches the installed examples from #2042.
The existing command keeps the open-project behavior and the unsaved-changes prompt.

**Dropped:** A new project-open command and MCP tool would duplicate the existing command.
A current-directory or repository fallback could show unrelated development files.
A recursive scan would include files outside the installed examples list.

**Cost to change later:** Change one UI module and its menu tests.

**Test isolation:** A test-only thread-local directory override keeps fixture projects separate from the installed package path.
Parallel UI tests keep their original file-menu behavior.
The first failing pin used the real executable directory before the fix.
