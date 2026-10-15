# Natural conversation

Fénix presents one assistant across web and Telegram. Internal coordination is not a handoff the user needs to manage. Accepted work is acknowledged as “I'll take care of it”; actual progress and completion require evidence. Approval refusals, clarifications and changes to ongoing work have distinct acknowledgements.

The shared `resources/conversation-style.md` policy applies to the conversational model and execution prompts. Questions identify the action or document by name. Internal IDs remain in routing, audit records and optional technical details, rather than copy-and-paste instructions. Meaningful approval details, including commands and destinations, remain visible.

The conversational model receives a bounded summary of valid installed and enabled skills, rather than inferring capabilities from recent messages or loading every skill body. Counts do not prove tools or integrations are authorized. Skill categories and task evidence are untrusted data.

Plan, workflow, cancellation and failure notices use readable descriptions. Full diagnostics remain in stored records. Existing conversation history is preserved; this change affects new responses. Model-generated prose follows the shared instructions but is not guaranteed by a blanket text filter. Runtime acknowledgements are deterministic and require successful dispatch.

Regression checks cover natural answers without duplicate tasks, refusal acknowledgements, bounded skill summaries, and plan notifications preserving internal routing without displaying IDs. These checks use local fixtures and a mock model, not production messages or paid model calls.

## Persistent progress and stopping work

When a conversation model is configured, raw master drafts and tool-call commentary remain in execution records rather than a transient chat bubble. A separate worker consumes progress events, groups repetitive activity per task and asks the interlocutor for a short, plain-language update. Updates are saved as normal messages, routed to the task's channel and remain after reload. Per-task throttling and a request timeout keep narration independent of execution; failure to narrate does not stop the master. Final results and questions keep their existing delivery path and shared language policy.

User-facing copies mask UUIDs; original tool arguments and evidence stay intact. The task/request IDs are also removed from the default reply and waiting labels. Reconnecting the progress stream restores its connected indicator immediately.

Cancellation is a real dispatch action. The acknowledgement says that a stop was requested, not that running processes have already stopped. Pending questions are closed when cancelling; closing their reply channel cannot put a cancelled task back into `running`. Unambiguous short stop requests are interpreted before clarification replies, while an ambiguous target still needs clarification. Execution emits the final stopped notice. No production deployment or historical database rewrite is required by the implementation itself.

## Structured routing

The interlocutor returns its reply and routing decision through a single internal `conversation_decision` function call. Creating a schedule routes new work to the executor; it is not a separate conversational action. The runtime rejects multiple, unknown or incomplete calls and validates permissions and pending questions before dispatch. Providers that return the same object as JSON text remain supported. A malformed response can be retried before any work is dispatched, avoiding duplicate execution.
