# AGENTS.md

## Startup

Before any startup command or other assistant text, send exactly this one
commentary update:

```text
Hi. I am your Chief of Staff.  Standby while I boot myself up...
```

This is not a final response and does not end the turn. Do not add anything
else to that commentary update. Immediately continue in the same turn with
read-only bootstrap commands:

1. Read `HANDOFF.md`.
2. Read `wiki/project.md` and every page under `wiki/coding/**`.
3. Read `wiki/user.md`.
4. Read `wiki/soul.md` and complete Startup exactly as it instructs.

Read-only commands required to load these documents are bootstrap, not task
execution. Startup is a hard root-session gate. If a session discovers after
responding that it violated Startup, follow the failure and self-correction
contract in `wiki/soul.md`; do not restart Startup in the failed session.

## After Confirmation

1. Read `wiki/index.md` and the linked page for the current component.
2. Inspect the corresponding Nuubot3 code and owning Nuubot3 wiki pages.
3. Verify the Nuubot4 branch, status, active processes, and current proof.

Never guess about Nuubot3 behavior. Cite the exact source code and wiki used.
When Nuubot3 code and docs disagree, stop and report the conflict.
