## #2423 - Windows troubleshooting guidance

**Decided:** Add a dedicated guide reference page, link it from the Windows
install section and a small FAQ, and test the rendered page and article
links. Keep the existing refusal-message troubleshooting page separate.

**Why:** A user whose app never opens needs startup and PC diagnostics
before machining refusal guidance. The existing guide has no FAQ, so one
focused question supplies the requested entry point without inventing a
larger FAQ. Rendered-page tests catch missing publication and broken
discovery paths.

**Decided:** Describe whole-PC freeze symptoms as possible driver or
hardware hangs, and label the Parallels ARM result as a report from one
setup. Give ordered checks and vendor links, with conditional UMA advice.

**Why:** Symptoms cannot prove a particular component is faulty. The issue
provides one VM result, not general compatibility testing. Hardware
settings depend on the motherboard and installed RAM.

**Dropped:** A categorical diagnosis, guaranteed backend fixes, or broad
ARM/VM support claims. None has sufficient evidence. Diagnostic tools are
named, not endorsed.

**Decided:** Document Command Prompt stderr capture into the user's folder
while automatic file logging in #2421 remains pending.

**Why:** The current logging implementation writes to stderr. A guessed
automatic log location would send users searching for a nonexistent file.
The command gives them a known file to inspect and share now.

**Dropped:** Waiting for #2421 or documenting its proposed path before it
ships. This page can help users immediately without coupling the changes.

**Cost to change later:** Update the logging paragraph after #2421 lands,
and revise the hardware or VM guidance as reproducible evidence arrives.
No runtime behavior, dependency, or posted output changes.
