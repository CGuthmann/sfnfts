# Git Guideline

## Branching

- We use the "feature branching" Strategy (see points below and [Merging](#Merging))
- **each task** is carried out in its **own branch** (e.g. ``feature/new-feature``, ``bugfix/bug``)
- there is only **one feature** or bugfix **per branch**
- **make incremental, small changes** (the longer a branch exist, conflicts will become more likely)
- as always: use descriptive names and make use of prefixes (e.g. ``feature`` and ``bugfix``)

## Commits

- **keep commits atomic** (as small as possible)
  - usually one file with one specific modification (e.g. new function, refactor, etc.)
- use descriptive commit messages
  - What was done?
  - What file/class/function is affected?
  - "should work now", "quickfix", etc. are not valid commit messages

## Merging

- **don't intersect ``feature`` or ``bugfix`` branches with each other** if the synchronization of two 
sub branches is necessary:
  - merged both branches with dev and create a new one
  - merge one branch into the other and only work on this one branch afterward
- **only the ``dev`` branch may be merged to ``main``**
  - ``dev`` may only be merged with ``main`` if ``dev`` is in a **stable and complete** (publishable) state 
  (e.g. version or milestones)
- every merge to ``dev`` **requires a merge request** and a code review
  - only merge **complete and tested** features or bugfixes to ``dev``
- squash and delete sub-branches afterward, then they are merged to ``dev`` 

## Excluding files

- immediately add all files or directories that should be excluded to the ``.gitignore`` file
- exclude all files that depend on your local system from the git repository
  - this includes all kinds of **generated files** and **executables**, local IDE and project 
  **settings**, etc.
- <u>**At no point of time should a file containing passwords be added to the git repository!**</u>
  - no, not even to briefly test or share something