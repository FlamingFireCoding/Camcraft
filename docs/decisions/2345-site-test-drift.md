## #2345 - Keep guide pins and site icons aligned with the app

**Decided:** Read the machine and post registrations from `posting_files.rs`.
Bind the guide's machine name to its shipped filename in the pin.
Copy the app's missing `heart.png` and `tool-wire.png` into the site, together
with the current icon license notice.

**Why:** The registrations moved out of `posting.rs`. The site must ship the
same icon bytes and attribution as the app. Keep these strict checks rather
than weaken them to accept missing assets or stale sources.

**Dropped:** Broad Rust changes and CI wiring are outside this repair.
The full site suite still reports an existing stale tool-number label and
six errors because Node is unavailable. The machine and icon pins pass,
but the issue's full-suite acceptance is not fully verified.

**Cost to change later:** A future registry move needs another test-path
update. New app icons and their license notice must also reach the site.
