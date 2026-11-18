## #2374 - A machine file names the post that writes its programs

**What was decided:** A `.mch` file may state `controller.post`, the name of
a built-in post. `Machine.post` carries it. `posting::post_offer` returns a
`PostOffer { post, why }`: the machine's named post (`OfferWhy::Machine`)
when it is a registry post that serves the mount and is not the setup's
post, else the kind offer as before (`OfferWhy::Kind`). The Setup page's
banner says which. Taking the offer queues the existing `SelectPostNamed`,
so it rides undo. The reader refuses anything but a non-empty string; the
validator also refuses an unknown name and a post that does not serve the
machine's kind, at `controller.post`.

**Why:** `default_post_for` takes only the machine's kind, so for every
mill it offered Fanuc, and a setup on the grblHAL machine was never pointed
at the grblHAL post. The pairings existed only as comments in `MACHINES`.
In the file, a user's own machine can state its post too. `controller` is
the section about the control, and the post speaks the control's dialect.

**Shipped files:** 32 state their post, where the pairing is written in
`MACHINES`, a machine's issue or its post: the PathPilot mills, routers,
lathes and plasma table; grblHAL and the Sienci AltMills; TinyG; the
Makera machines; the Doosan DNM 5700s; the Haas machines. The generic
machines (`vmc`, `tilthead`, `millturn`, `lathe`), the Sodick ALC400G and
the Mach3-era Tormach PCNC 770 and 1100 mills state none: no pairing is
written for them, and a Mach3 control is not PathPilot. The sinker
template states none either: its kind's default is already the sinker
post, and the EDM suites share a byte copy of the file.

**Considered and dropped:**
- A table of machine-to-post pairs in code next to `MACHINES`: a user's
  own machine could not state its post.
- Changing the post on mount: a mount never re-points the post (#343,
  #388), and the maintainer repeated it on #2374.
- The reader refusing an unknown name: the registry is the app's, so the
  post crate would need it passed in. The validator refuses it in the
  catalogue and at save, and `post_offer` offers only a registry post that
  serves the mount, so a stranger name is never offered.
- A `PROJECT_VERSION` bump: `Machine` is `#[serde(default)]`, so an older
  reader drops the field. That loses only the offer, never a posted line.

**Cost to change later:** Local to `post::machine::post_name`,
`app::machine::validate`, `app::posting::post_offer` and the Setup page's
banner. Moving the registry into `camcraft-post` (#924) moves the
validator's registry check with it.
