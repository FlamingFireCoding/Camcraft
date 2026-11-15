## #1794 - Remove committed PR review artifacts

**What was decided.** Remove the review artifacts listed in the issue for
PRs #1711, #1771, #1765 and #1756. Include #1771 because the issue inventories
its two images, even though its removal summary names only three folders.
Replace historical references to the removed folders with their original
PR links. Pin the absence of all four folders in an xtask integration test.
For these four folders, this supersedes the earlier rationale for retaining
review evidence in the product tree; the older entries retain their history.

**Why.** These images and their capture helpers are review evidence rather
than app or site inputs. Review evidence belongs in PR bodies, with images
on the author fork's images-only branch. The pin makes reintroducing these
known artifacts fail without restricting legitimate site assets.

**Considered and dropped.** A general ban on documentation images would
reject valid site content. Migrating images or removing other review
folders would expand this cleanup beyond the issue's listed artifacts.

**Cost to change later.** Small: the pin lists four folders. Future policy
can broaden it separately if the repository adopts a general review check.
