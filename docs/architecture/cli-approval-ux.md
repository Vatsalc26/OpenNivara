# CLI Approval UX

CLI is the first proof surface for approval pause/resume.

Cross-surface action consistency is defined in [Surface Consistency Matrix](surface-consistency-matrix.md).

It should support both:

1. interactive approval inside `ask`/`chat` when stdin is a TTY
2. non-interactive approval subcommands for pending approvals

Never auto-approve in non-interactive mode.

## Interactive Approval

When `EngineResponseKind::ApprovalRequired` is returned and stdin is interactive, show:

```text
Approval required

Operation: write_file
Target: /path/to/notes.txt
Classification: local_modify
Reason: Local file modification requires approval.

Preview:
OpenNivara wants to create notes.txt.
+ hello world

Approve once? [y/N/details/quit]:
```

Inputs:

- `y` / `yes`: approve once, execute operation, continue model response
- `n` / `no`: deny operation, continue denial explanation
- `d` / `details`: show full preview and full arguments, then ask again
- `q` / `quit`: leave approval pending and exit/return to prompt
- Enter: default no/deny

Default: Enter means no/deny.

## Non-Interactive Behavior

When approval is required and stdin is not a TTY, print:

```text
Approval required: appr_123
Operation: write_file
Target: /path/to/notes.txt

Run:
  opennivara approvals show appr_123
  opennivara approvals approve appr_123 --session sess_123
  opennivara approvals deny appr_123 --session sess_123
```

The command exits/returns without approval. It must not execute the operation.

## Clap Structure

Target structure:

```rust
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Chat(ChatArgs),
    Ask(AskArgs),

    #[command(subcommand)]
    Approvals(ApprovalsCommand),

    // existing commands remain
}

#[derive(Args)]
pub struct ChatArgs {
    #[arg(long)]
    pub session: Option<String>,

    #[arg(long)]
    pub non_interactive: bool,

    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct AskArgs {
    pub message: Vec<String>,

    #[arg(long)]
    pub session: Option<String>,

    #[arg(long)]
    pub non_interactive: bool,

    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum ApprovalsCommand {
    List(ApprovalsListArgs),
    Show(ApprovalShowArgs),
    Approve(ApprovalActionArgs),
    Deny(ApprovalActionArgs),
    Continue(ApprovalActionArgs),
}

#[derive(Args)]
pub struct ApprovalsListArgs {
    #[arg(long)]
    pub session: Option<String>,

    #[arg(long)]
    pub status: Option<ApprovalStatusFilter>,

    #[arg(long)]
    pub all: bool,

    #[arg(long)]
    pub json: bool,
}

#[derive(Clone, ValueEnum)]
pub enum ApprovalStatusFilter {
    Pending,
    Denied,
    Executing,
    Executed,
    Failed,
    Completed,
}

#[derive(Args)]
pub struct ApprovalShowArgs {
    pub approval_id: String,

    #[arg(long)]
    pub session: Option<String>,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub full_args: bool,
}

#[derive(Args)]
pub struct ApprovalActionArgs {
    pub approval_id: String,

    #[arg(long)]
    pub session: Option<String>,

    #[arg(long)]
    pub json: bool,
}
```

Command names:

- `opennivara approvals list`
- `opennivara approvals show <approval_id>`
- `opennivara approvals approve <approval_id>`
- `opennivara approvals deny <approval_id>`
- `opennivara approvals continue <approval_id>`

Use `continue`, not `resume`. `resume` is easy to confuse with chat/session resume.

Optional aliases later:

- `approval` as alias for `approvals`
- `cont` as alias for `continue`

## Examples

```bash
opennivara ask "Create notes.txt with hello world"
opennivara approvals list
opennivara approvals show appr_123
opennivara approvals show appr_123 --full-args
opennivara approvals approve appr_123 --session sess_123
opennivara approvals deny appr_123 --session sess_123
opennivara approvals continue appr_123 --session sess_123
```

## List Behavior

Default list shows only action-needed approvals:

- pending
- executed with final response pending
- denied with explanation pending, if that state exists
- failed requiring attention

Hide completed by default. Use `--all` to include completed.

Columns:

```text
ID          STATUS     OPERATION   TARGET                  ACTION
appr_123   pending    write_file  /project/notes.txt       approve/deny
appr_456   executed   write_file  /project/readme.md       continue
```

## Status-Specific Behavior

`pending`:

- approve: execute operation once and continue model
- deny: append `approval_denied` tool result and continue model
- continue: error; operation has not been approved or denied yet

`executing`:

- approve: reject as `already_executing`
- deny: reject as `already_executing`
- continue: explain operation is still executing or stale recovery is needed

`executed`:

- approve: do not rerun; say use `continue`
- deny: not allowed; operation already executed
- continue: retry provider/model continuation only

`denied`:

- approve: not allowed
- deny: already denied
- continue: retry denial explanation only if pending turn remains

`completed`:

- all actions return already completed
- hidden from list by default

`failed`:

- show error
- no automatic retry tool execution
- user should start a new request if they want to try again

Executed approval output:

```text
This operation already executed. Use:
  opennivara approvals continue appr_123 --session sess_123
```

Completed output:

```text
Approval already completed.
```

Failed output:

```text
This operation failed and will not be retried automatically.
Start a new request if you want to try again.
```

## Show Output

Pending:

```text
Approval appr_123

Status: pending
Operation: write_file
Target: /project/notes.txt
Classification: local_modify
Reason: Local file modification requires approval.

Preview:
OpenNivara wants to create notes.txt.

Full arguments:
{
  "path": "notes.txt",
  "mode": "create_new",
  "content": "hello world"
}

Actions:
  opennivara approvals approve appr_123 --session sess_123
  opennivara approvals deny appr_123 --session sess_123
```

Executed with continuation pending:

```text
Approval appr_123

Status: executed
Operation: write_file
Target: /project/notes.txt

The approved operation already ran:
Created notes.txt, writing 11 bytes.

Final response is still pending.

Last error:
provider timeout

Action:
  opennivara approvals continue appr_123 --session sess_123
```

## JSON Behavior

`--json` outputs stable serialized DTOs:

- `ApprovalView` for show
- `Vec<PendingApprovalSummary>` for list
- `ApprovalActionResponse` for approve/deny/continue

Do not output ad hoc JSON.

## Locked Decisions

1. Interactive CLI approval exists for TTY ask/chat.
2. Default Enter means no/deny.
3. `details` shows full preview and full arguments.
4. `q` leaves approval pending.
5. Non-interactive CLI never auto-approves.
6. Add `approvals` subcommands.
7. Use `continue`, not `resume`.
8. `approve` never reruns executed approvals.
9. `continue` never runs tools.
10. Completed approvals are hidden from list by default.
11. `--json` uses shared DTOs, not ad hoc JSON.
12. `show --full-args` exposes full arguments.

## Tests

Required tests:

1. Pending approval appears in approvals list.
2. Completed approval is hidden by default.
3. `--all` shows completed approvals.
4. `approvals show` prints preview and available actions.
5. `approvals show --json` serializes `ApprovalView`.
6. `approvals approve` pending executes once.
7. `approvals deny` pending does not execute tool.
8. `approvals continue` executed retries provider only.
9. `approvals approve` executed does not rerun tool.
10. Non-interactive approval-required ask prints commands and exits without approval.
11. Interactive `y` approves.
12. Interactive `n` denies.
13. Interactive `d` shows details.
14. Interactive `q` leaves approval pending.
