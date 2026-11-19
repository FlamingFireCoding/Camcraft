## #2744 - The Report door sends file names, not full paths

**Decided.** The Report door puts a shared form of the Debug info report
into the GitHub URL (`debug_info::shared_report`). That form names the
project file and the machine file by their file names only. The Debug info
panel in the application keeps the full paths (`debug_info::report`). An
unsaved project still reads `Project: unsaved`. A path with no file name
part reads `(no file name)`, never the path.

**Why.** The browser sends the URL to github.com, and the user can publish
the body without notice. A full path usually holds the user name. The file
name is enough for a triage. The machine line had the same leak (`from
<path>` of a user's own machine file), so it takes the same rule.

**Dropped.**
- A second, separate report text for the door: two sources drift. One
  builder with a "names only" switch keeps the fields the same.
- A general scrub of the `Status:` line: the status text is free text, and
  a scrub that guesses paths can remove good text. The shared form only
  replaces the exact project path in the status (for example after a save)
  with the file name.
- A home-directory replace (`~/...`): the directory names under the home
  can still identify the user or the customer. The file name is enough.

**Cost to change later.** Low: one switch in `debug_info::report_with`.
