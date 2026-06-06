# Telegram Approval UX

Telegram is an equal surface and uses the same backend approval APIs as Desktop and CLI.

Cross-surface action consistency is defined in [Surface Consistency Matrix](surface-consistency-matrix.md).

Telegram has no special tool permission layer. Its responsibilities are:

- chat authorization
- command parsing
- concise message formatting
- same-chat approval binding

## Backend Actions

Telegram must use the shared backend actions:

- `approve_pending_operation`
- `deny_pending_operation`
- `continue_pending_approval`
- `list_pending_approvals`
- `get_approval_details`

## Commands

MVP commands:

- `/approvals`
- `/approval <approval_id>`
- `/approve <approval_id>`
- `/deny <approval_id>`
- `/continue <approval_id>`

Use `/continue`, not `/resume`, to match CLI and avoid chat/session resume confusion.

Optional aliases later:

- `/pending`
- `/details <approval_id>`

MVP uses commands only. Do not implement inline buttons yet.

Reason:

- simpler implementation
- easier tests
- no callback-query state model
- works in all Telegram clients

Future inline buttons may be:

- Approve once
- Deny
- Details
- Continue response

## Pending Approval Message

```text
Approval required

Operation: write_file
Target: /project/notes.txt
Classification: local_modify

Preview:
OpenNivara wants to create notes.txt.
+ hello world

Approve:
/approve appr_123

Deny:
/deny appr_123

Details:
/approval appr_123
```

## Details Command

Command:

```text
/approval appr_123
```

Example:

```text
Approval appr_123

Status: pending
Operation: write_file
Target: /project/notes.txt
Reason: Local file modification requires approval.

Preview:
OpenNivara wants to create notes.txt.

Actions:
/approve appr_123
/deny appr_123
```

Full details:

- do not dump massive JSON by default
- if full arguments are large, say full details are available in Desktop/CLI
- MVP can avoid sending `full_arguments_json` in Telegram

## Status-Specific Behavior

`pending`:

- `/approve`: execute operation once and continue model response
- `/deny`: deny and continue model denial explanation
- `/continue`: reply `This operation has not been approved or denied yet.`

`executing`:

- `/approve`: `Already executing.`
- `/deny`: `Already executing.`
- `/continue`: `Operation is still executing. Try again shortly.`
- stale recovery later may mark failed/interrupted

`executed`:

- meaning: tool already ran and final model response is pending
- `/approve`: `This operation already executed. Use /continue appr_123.`
- `/deny`: `This operation already executed and cannot be denied.`
- `/continue`: retry model continuation only

`denied`:

- if denial explanation is pending, `/continue` continues denial explanation
- if completed, reply `Already denied and completed.`

`failed`:

```text
Operation failed.

Reason:
<sanitized error>

This operation will not be retried automatically.
Start a new request if you want to try again.
```

No retry-operation command in MVP.

`completed`:

```text
Approval already completed.
```

Completed approvals are hidden from `/approvals` by default.

## /approvals Output

Default shows action-needed approvals only.

Example:

```text
Pending approvals

appr_123
write_file
/project/notes.txt
Action: /approve appr_123 or /deny appr_123

appr_456
write_file
/project/readme.md
Action: /continue appr_456
```

Optional later:

- `/approvals all`
- `/approvals completed`

## Same-Chat Rule

Approval belongs to the particular Telegram chat that created it.

Telegram approval actions must pass:

- `RequestSource::Telegram(chat_id)`
- `session_id`
- `actor_id = telegram_<chat_id>`

Approval action is accepted only if:

- same chat/session
- actor has approve permission

MVP practical rule:

- same authorized Telegram chat can approve/deny/continue its own approval
- wrong chat returns `This approval belongs to another chat.`

Telegram must not approve globally across surfaces/chats.

## Preview Length

Keep Telegram concise.

Show:

1. summary
2. target
3. short preview/diff if small
4. command actions

Do not dump:

- `full_arguments_json`
- giant diffs
- full preview details JSON

## Backend Response Handling

After `/approve`:

If backend returns final answer:

```text
Approved and executed.

<final assistant answer>
```

If backend returns executed but continuation failed:

```text
The operation executed, but OpenNivara could not finish the final response.

Use:
/continue appr_123
```

If backend returns another approval required:

```text
Another approval is required:
<approval summary/actions>
```

After `/deny`:

If backend returns final denial explanation:

```text
Denied.

<final assistant denial explanation>
```

If continuation fails:

```text
Denied. Final explanation could not be completed.

Use:
/continue appr_123
```

## Locked Decisions

1. Telegram uses the same backend `ApprovalView` and `ApprovalActionResponse`.
2. Telegram has no special tool permission layer.
3. Telegram approval is same-chat only.
4. Commands are `/approvals`, `/approval`, `/approve`, `/deny`, `/continue`.
5. Use `/continue`, not `/resume`.
6. MVP uses commands only, no inline buttons.
7. Pending supports approve/deny.
8. Executed supports continue only.
9. Failed has no retry operation.
10. Completed is hidden from `/approvals` by default.
11. Telegram previews are concise and truncated.
12. Full argument JSON is not dumped by default.
13. Wrong chat/session approval is rejected.
14. Telegram should not approve globally across surfaces/chats.

## Tests

Required tests:

1. `/approvals` lists pending approvals for same chat.
2. `/approval` shows pending approval summary/actions.
3. `/approve` pending executes once.
4. `/deny` pending does not execute tool.
5. `/continue` pending returns not-approved-yet message.
6. `/approve` executed says use `/continue`.
7. `/deny` executed is rejected.
8. `/continue` executed retries provider only.
9. Completed approvals are hidden from `/approvals`.
10. Wrong chat cannot approve.
11. Long preview is truncated.
12. Telegram does not dump `full_arguments_json` by default.
13. Failed approval has no retry-operation command.
14. Command parser accepts `/approve`, `/deny`, and `/continue` with approval IDs.
