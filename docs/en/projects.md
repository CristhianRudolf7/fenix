# Projects in one conversation

[Español](../es/projects.md)

Fénix keeps one visible conversation for the owner across web and Telegram.
Projects organize context and work internally; they do not create separate chats,
require a project selector before sending a message, or run a permanent agent for
each project. General conversation remains a first-class scope.

These assistant projects are separate from the provider credential project's
setting. Switching work projects does not select another model or API key; the
configured provider connection remains shared by the master and its children.

## Register and maintain a project

Open **Projects** in the web navigation. Give the project a name, optional aliases,
description and working brief. An optional existing folder supplies the default
workspace for its tasks. Office, planning and personal projects need no folder.
Names and aliases must be unique across projects. The brief can use this structure:

```text
Objective
Constraints and preferences
Progress
  Completed
  In progress
  Blocked
Key decisions
Relevant files
Next steps
Critical context
```

The panel supports creation, editing and a recent-task view. Editing a project does
not change the workspace already captured by an existing execution. Keep general
preferences in general memory and project-specific decisions in that project's
memory. The **Memory** panel filters these scopes separately and prevents changing
an existing memory's scope as an incidental edit.

## Route first, then assemble context

Every inbound request enters the existing queue. Its optional reply reference and
foreground project hint are captured on arrival. Clear names, aliases, greetings
and explicit general-conversation requests can be resolved by deterministic rules.
Other requests use a bounded routing request to the configured model, with the
project catalog and relevant conversational references, before project memories
or execution tools are exposed. This routing request consumes tokens; it does not
use a separately configured or automatically cheaper model.

The router distinguishes general discussion, a single project, coordination across
projects and an ambiguous destination. It may divide a message into ordered intents.
Uncertain destinations require a textual clarification before project effects.
Invalid routing output stops the task instead of silently selecting a project.
A clear destination does not resolve an unclear objective: the task agent must
still clarify what an unspecified “improvement” should achieve.

A task binding stores the destination, readable project scopes, reply reference,
workspace and project context version. The engine validates project IDs and scope
access. Each execution retains its binding while other messages change focus.
Explicit cross-project requests grant those read scopes to coordination; they do
not merge project memories or enable arbitrary cross-project writes.

Context combines stable instructions, applicable general memory, a project brief,
task references, scoped memory/history and a compact skill catalog. Full skill
instructions remain on demand. History and summaries are scoped; original messages
remain in the shared visible conversation and storage. General history is handled
separately, without appending every project's detailed transcript to each request.

## Example: inspect A, then improve B

For “Tell me how A is going and then improve B's login,” the master coordinates two
intents in the requested order. It can consult A's recorded state directly and then
delegate substantial work in B with a precise objective and expected output. A
state lookup does not automatically need a subagent, and a mentioned project does
not automatically create one. The model chooses suitable delegation through the
existing tools and instructions; this is not a fixed recipe for every wording.

Project work delegated from coordination must specify its destination. A child gets
the relevant handoff and project binding, not the full main conversation. It cannot
delegate again. The master remains available while children work or wait for a
textual answer. Existing retry and concurrency limits still apply.

When a child finishes, its report remains associated with its original project.
The report does not move foreground focus to that project or replace another
task's workspace. Pending questions and approvals are likewise linked to their
original task. Web **Reply** links and Telegram reply references can anchor a new
message to an earlier task; these references do not approve an action by themselves.

## Storage and API

SQLite migrations add projects, task bindings and scoped memory/history support.
Existing messages and memories are preserved as general records. Migration does
not infer ownership from historical text or automatically reassign old work to
new projects. Runtime data remains in the configured private data directory.

| Endpoint | Behavior |
| --- | --- |
| `GET /api/projects` | List registered projects. |
| `POST /api/projects` | Create with `name`, `aliases`, `description`, optional `workspace`, and `brief`. |
| `PUT /api/projects/{id}` | Update the same project; increment its context version. |
| `GET /api/projects/{id}` | Return the project and recent scoped tasks. |
| `POST /api/messages` or `/api/chat` | Submit `text` and optional `reply_to_task_id` to the same conversation. |
| `GET /api/messages`, `/api/tasks`, `/api/tasks/{id}` | Include task binding information alongside the existing records. |
| `GET /api/memory?project_id=...` | Search only the chosen scope; omit the parameter for general memory. |
| `POST /api/memory` | Accept optional `project_id`; omission selects general memory. |
| `DELETE /api/memory/{id}?project_id=...` | Require the matching scope for a project memory. |

## Limits and validation

This is logical context isolation, not an OS sandbox. Commands retain the Linux
user's filesystem access. Incorrect natural-language routing is still possible;
deterministic scope checks enforce the resolved binding, not the truth of its
interpretation. Explicit references and clarifications reduce that risk.

Graphs, embeddings, autonomous memory extraction, retrospective project assignment,
advanced task recovery and project-specific permission hierarchies remain future
work. Interrupted execution is recorded and is not automatically replayed.

The repository includes deterministic, mocked-provider, HTTP and browser coverage
for project boundaries and the single-chat interface. These tests are not a live
model benchmark and do not establish universal routing accuracy or a token-saving
percentage. Live-model evaluation should measure routing errors, wrong-scope reads
and effects, unnecessary questions, completed work, tokens and latency together.
