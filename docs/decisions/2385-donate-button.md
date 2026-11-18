## #2385 - Open the project donation link from the top bar

**What was decided.** Add an enabled Donate button at the right of the
ribbon tab row. Keep the Stripe URL in one `DONATION_URL` constant. It
matches `.github/FUNDING.yml` and the "chip in here" link in `README.md`.
The native shell starts the system browser launcher on a background
thread. A launch failure shows a named message in the status strip. The
browser shell uses `OpenUrl::new_tab`.

The button uses the Tabler heart outline icon. Its source is
`https://raw.githubusercontent.com/tabler/tabler-icons/main/icons/outline/heart.svg`.
The repository includes the SVG master and its PNG. The existing Tabler
MIT notice covers this icon. The licence headings explicitly separate
this supplied master from the original CamCraft artwork.

**Why.** The maintainer asked for a Donate button in the top bar. The
tab row has space at the right without reducing the operation cells.
The shared URL prevents platform-specific donation destinations. A
background launcher keeps the UI responsive. A visible launch error
lets the user identify a browser failure. Opening a website changes no
machining state, so this action needs no undoable machining command.

**Considered and dropped.** A new browser crate dependency adds scope
for one fixed link. The system launcher uses the existing standard
library instead. Intercepting platform output in the shell frame would
change the same function as PR #1798. The separate donation module
keeps this change local. A new payment dialog duplicates the existing
Stripe page and adds no required behavior.

**Cost to change later.** A destination change updates one constant.
A launcher change stays in the donation module. The licence edit touches
the artwork headings; PR #2122 changes another licence section. Whichever
PR merges second refreshes if Git reports a conflict.
