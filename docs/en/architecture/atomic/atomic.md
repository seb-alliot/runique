# Atomic file writes

When Runique writes several files at once, it writes them as a single batch: **all of them, or none**. An error halfway through, or a power cut, never leaves a project half written.

You never call this mechanism yourself. This page explains what it does, the files it can leave behind, and what to do when a command refuses to go on.

---

## Where Runique uses it

| Command or function | What is written as one batch | Journal |
| --- | --- | --- |
| `runique new` | every file of the project, the `migration/` crate and the `.env` | in the project's folder |
| `runique makemigrations`, first run | the `migration/` crate when the project has none | in `migration/` |
| `runique makemigrations` | the migrations, the removal of those it replaces, and `lib.rs` | in `migration/src/` |
| `runique migration up`, once applied | the snapshots of the changed tables and the removal of the migrations' tag | in `migration/src/` |
| file uploads (`FileField`, `MediaField`) | the files of one request | none |

---

## How a batch is written

1. Runique notes in a **journal** the list of files it is about to write.
2. Each file is first written under a temporary name, next to its destination.
3. If a file already exists there, it is set aside in a backup.
4. The temporary files take their place.
5. The journal is marked "everything is in place".
6. The temporary files and the backups are removed, then the journal, last.

The rule that follows:

| When the incident happens | What Runique does |
| --- | --- |
| before the mark of step 5 | it undoes: the project goes back to exactly what it was |
| after the mark | it finishes the cleanup: every file is already in place |

---

## The files you may come across

They only exist during a run. If you see one while no command is running, a run was interrupted.

| File | What it is |
| --- | --- |
| `.runique-journal` | the batch's journal, in the folder given above |
| `<file>.runique-tmp` | the new content, not in place yet |
| `<file>.runique-bak` | the old content of a file the batch replaces |

These three names belong to Runique: don't create a file ending in `.runique-tmp` or `.runique-bak`, nor a file named `.runique-journal`. They don't belong in version control.

---

## After an error

Disk full, write permission denied, file locked: the command stops, undoes everything it had written and shows the error, with the file involved. The project is as it was before. There is nothing to clean up: fix the cause, then run the command again.

---

## After a power cut

A power cut, or a killed process, leaves Runique no time to undo. The journal left on disk is what makes the repair possible, **the next time the same command runs**:

| You run again | What Runique does first |
| --- | --- |
| `runique new my_project` | it removes the unfinished project, then creates it as usual |
| `runique makemigrations` | it puts `migration/src/` back as it was, then computes the migrations |

The repair is announced by one line:

```text
An interrupted run was repaired in 'migration/src'.
```

If the cut happened during the final cleanup, every file was already in place: Runique simply finishes the cleanup. For `runique new`, the project is then complete, and the command answers that the folder already exists.

### `runique migration` waits for the repair

`runique migration up`, `down` and `status` only read the migrations: they repair nothing. As long as a journal is present in `migration/` or `migration/src/`, they refuse to start, so that half-written migrations are never applied:

```text
A makemigrations run was interrupted in 'migration/src'. Run makemigrations again before applying the migrations.
```

---

## When Runique refuses to repair

The repair only knows the states a power cut can leave. If a file of the batch was **edited, removed or added by hand** since the cut, Runique can no longer tell which version is the right one. It doesn't choose for you: **it touches nothing** and lists the files involved.

```text
A run was interrupted in 'migration/src' and files changed since. Nothing was touched. Put right the ones listed below, then run the command again:
migration/src/lib.rs: changed since the batch was interrupted, and its backup holds something else
```

Each line names a file and gives the reason, in the application's language:

| Reason shown | What it means | What to do |
| --- | --- | --- |
| `changed since the batch was interrupted, and its backup holds something else` | the file was edited after the cut; its `.runique-bak` backup holds the old version | to keep your version: delete the `.runique-bak`. To go back to the old one: copy the `.runique-bak` over the file |
| `is not the file the batch was creating` | Runique was creating this file, and something else is there | move or delete that file |
| `is gone, and no backup holds it` | the file existed before the batch, it is gone and no backup holds it | put it back, from git for instance |
| `has a backup, though nothing was there before the batch` | a `.runique-bak` sits where the batch had nothing to back up | delete that `.runique-bak` |
| `not a file of the batch, in a folder the batch created` | a file was added in a folder the batch had just created; Runique won't remove that folder with a file of yours in it | move that file elsewhere |
| `is no longer in the journal's folder` | a folder of the project was replaced by a link to another place | put the real folder back |

Once the listed files are put right, run the command again: the repair starts over.

For versioned migrations, `git status` shows at a glance what moved in `migration/src/`.

### A temporary file or a backup already there

Outside any power cut, a command also refuses to start when a `.runique-tmp` or a `.runique-bak` already takes the place it needs:

```text
migration/src/lib.rs.runique-bak is in the way
```

That file doesn't belong to the current run. If it holds something you want to keep, save it; otherwise delete it, then run the command again.

---

## What not to do

- **Delete `.runique-journal` by hand.** Without it, Runique no longer knows a batch was interrupted: the files already replaced stay as they are, and the `.runique-tmp` and `.runique-bak` files stay in place. Let the command repair.
- **Run `runique migration up` to "get past it".** It refuses as long as the journal is there, on purpose.

If the journal was deleted anyway, the batch has to be undone by hand: for each `<file>.runique-bak`, copy it over `<file>`; then delete the `.runique-bak` and `.runique-tmp` files.

---

## File uploads

The files of one request are saved to the media folder as a batch: if one of them can't be, those already saved are removed, and the form shows the error.

An upload has no journal. A power cut while saving can therefore leave a `.runique-tmp` in the media folder; it can be deleted safely.

The `.replace()` option of a file field, which removes the old file when a new one takes its place, is described with the [form fields](/docs/en/formulaire/fields).

---

## Limits

- **Two commands at the same time** in the same project are not protected from each other: don't run two `makemigrations` at once.
- **Windows**: the mechanism is written for it, but it has only been proven on Linux.

---

← [**Architecture**](/docs/en/architecture) | [**Makemigrations**](/docs/en/architecture/makemigrations) →
