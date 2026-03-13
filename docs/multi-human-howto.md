# Multi-Human Workflows: Idea to Production

How two (or more) people go from "we need to build this" to shipped code — without discovering at merge time that nothing fits together.

Read [Multi-Human Intro](multi-human-intro.md) for the conceptual model. This guide is the practical walkthrough.

---

## The Setup

Your PM walks into Slack and says: "We need a subscription cancellation flow. The user clicks Cancel in their account settings, we show them a modal asking why, they confirm, we cancel in Stripe, create a case in Salesforce for the CSM to follow up, and send a confirmation email."

Two engineers are on it. Alice owns the frontend — the modal, the form, the loading states, the error handling, the confirmation screen. Bob owns the backend — the API endpoint, the Stripe integration, the Salesforce case creation, the email trigger.

They could both just start coding. Alice would build a modal that POSTs to `/api/cancel-subscription`. Bob would build an endpoint that accepts... something. A week later they'd open PRs and discover Alice sends `{ reason: "too expensive" }` while Bob's endpoint expects `{ cancellationReason: string, feedbackCategory: enum }`. And Bob built the Salesforce case creation as a synchronous step that Alice's UI doesn't show a spinner for. And the error response format doesn't match what Alice's error handler expects.

None of these are hard problems. They're just conversations that didn't happen. The multi-human workflow is those conversations — structured so agreements turn into artifacts your agents can check while you work.

**You'll need:**
- Two or more people, each responsible for their own piece
- A shared git repo everyone can push to
- A way to talk (Slack, call, whiteboard — whatever works)
- tackline installed (`claude plugin install tackline@tacklines`)

---

## Step 1: Prep — Figure Out Your Own Piece First

**Who**: You, alone with Claude. Everyone does this independently.
**Time**: 15-30 minutes.

Before you meet with anyone, let your agents explore what your piece actually involves. Not abstract events — concrete work. What screens do you need to build? What API calls do you need to make? What data do you need from the other person? What are you assuming about their side?

```
/storm-prep scope: cancellation-frontend, goal: subscription cancellation flow with reason capture and confirmation
```

Three agents fan out across your code. They're looking for the real touchpoints — where does your code call the backend? What does the form submit? What response does the UI expect? What happens on failure? They come back with two things:

**What happens in your piece** — the concrete things your code does:
- User opens the cancel modal from account settings
- Modal loads current subscription details (needs an API call)
- User selects a reason from a dropdown and optionally writes feedback
- Form submits to the backend (what endpoint? what payload?)
- UI shows a loading state while waiting
- On success: show confirmation, update the UI to reflect cancelled state
- On failure: show error message, let them retry

**What you're assuming about the other person's piece:**
- "I'm assuming there's a `GET /api/subscription` that gives me plan name and billing period for the modal header"
- "I'm assuming the cancel endpoint returns the Salesforce case ID so I can show a 'your account manager will reach out' message"
- "I'm assuming errors come back as `{ error: string, code: string }` so my error handler can show specific messages"

That second list — the assumptions — is the gold. Every assumption is a question you need to ask the other person before you start building.

Commit your prep file and push it so it's visible before you meet.

**The key discipline**: Don't look at anyone else's prep output before you finish yours. Independent findings surface real disagreements. If you peek first, you'll unconsciously align with their assumptions and miss the gaps.

---

## Step 2: Jam — Sit Down Together and Hash It Out

**Who**: Everyone, in the same room (or call). No agents — this is a human conversation.
**Time**: 30-60 minutes.

Pull up everyone's prep files. This is where the real work happens. You're not doing a ceremony — you're having the design conversation that would otherwise happen piecemeal over a week of Slack messages, except you're having it upfront and writing down what you agree on.

### What the Conversation Actually Sounds Like

**Alice**: "OK, so when the user clicks Cancel, I need to show a modal. I need their current plan name and billing date for the header — is there an endpoint for that?"

**Bob**: "Yeah, `GET /api/subscription` already returns that. I'll add a `nextBillingDate` field if it's not there."

**Alice**: "Cool. Then they pick a reason from a dropdown. What reasons do you need?"

**Bob**: "Salesforce has specific case categories. Let me check... we need: 'too expensive', 'missing features', 'switching to competitor', 'other'. And if they pick 'other', we need a free-text field."

**Alice**: "Got it. So my form sends something like `{ reason: string, details?: string }` to a cancel endpoint?"

**Bob**: "Yeah. `POST /api/subscription/cancel`. But I also need to know — should the cancellation happen immediately or at the end of the billing period? Stripe supports both."

**Alice**: "Product said end of billing period. So the response should tell me when exactly it ends, right? So I can say 'Your subscription will remain active until March 15.'"

**Bob**: "Right. I'll return `{ cancelledAt: timestamp, activeUntil: timestamp, salesforceCaseId: string }`. The Salesforce case creation is async though — I'm putting it on a job queue. So `salesforceCaseId` might be null initially."

**Alice**: "Hmm, I was planning to show 'Your account manager will follow up (case #12345)' on the confirmation screen. If it's null..."

**Bob**: "I can make it synchronous. It's a single API call, adds maybe 2 seconds."

**Alice**: "Two seconds is fine if we show a spinner. But what if Salesforce is down?"

**Bob**: "Then the cancellation still succeeds — we don't fail the whole thing because Salesforce is flaky. I'll return `salesforceCaseId: null` and you show a generic 'our team will follow up' instead."

**That conversation just prevented three merge-time surprises**: the reason format, the async Salesforce issue, and the error handling for partial failures.

### What You Write Down

```markdown
# Jam Session: 2026-02-27

## What We're Building
Subscription cancellation flow — modal with reason capture,
Stripe cancellation, Salesforce case, confirmation email.

## Who Owns What

### Alice (cancellation-frontend)
- Cancel modal in account settings
- Reason selection form (dropdown + optional free text)
- Loading, error, and confirmation states
- Consumes: GET /api/subscription, POST /api/subscription/cancel

### Bob (cancellation-backend)
- GET /api/subscription (already exists, adding nextBillingDate)
- POST /api/subscription/cancel (new endpoint)
- Stripe cancellation (end of billing period)
- Salesforce case creation (synchronous, but soft-fail)
- Confirmation email via SendGrid

## Agreed Interfaces

### GET /api/subscription (Bob owns)
Response:
  - planName: string
  - billingPeriod: "monthly" | "annual"
  - nextBillingDate: string (ISO 8601) ← NEW FIELD

### POST /api/subscription/cancel (Bob owns)
Request:
  - reason: "too_expensive" | "missing_features" | "switching" | "other"
  - details: string (optional, only when reason is "other")

Success response (200):
  - cancelledAt: string (ISO 8601)
  - activeUntil: string (ISO 8601)
  - salesforceCaseId: string | null (null if SF was unreachable)

Error response (4xx/5xx):
  - error: string (human-readable message)
  - code: "already_cancelled" | "payment_pending" | "stripe_error" | "internal"

### Salesforce Case (Bob owns, Alice doesn't touch)
- Created synchronously during cancel request
- If SF is down: cancel still succeeds, case created by retry job later
- Alice shows "your account manager will follow up" either way
  - With case ID if available
  - Generic message if null

## Still Figuring Out
- Email template: Bob's sending it via SendGrid but we haven't
  decided what it says yet. Alice might want to match the UI copy.
- What happens if the user has already cancelled? Bob will return
  "already_cancelled" error — Alice needs to handle that state
  in the modal (maybe don't show the cancel button at all).
```

Commit this to the repo (e.g., `docs/jam/session-2026-02-27.md`).

**Time-box it.** If you can't resolve something in 60 minutes, write "still figuring out" and move on. The tooling will remind you later.

---

## Step 3: Formalize — Turn Agreements Into Checkable Contracts

**Who**: You, alone with Claude. Everyone does this independently.
**Time**: 10-20 minutes.

Feed the jam artifact to Claude and let it generate machine-readable contracts from your agreements. Each person runs this for their own piece:

```
/formalize docs/jam/session-2026-02-27.md
```

Claude reads the jam file, asks which part is yours, then generates:

- **Schemas** for the interfaces you own — the authoritative definition of your API endpoints, request/response shapes, and error formats
- **Mocks** for the interfaces you consume — fake responses your agents can test against without needing the other person's code running
- **A validation config** — rules your sprint agents will check against as they work

For Alice's frontend, that means mock API responses she can develop against immediately. For Bob's backend, that means request schemas his endpoint must conform to.

Every field gets a confidence tag. Things you explicitly agreed on in the jam are `CONFIRMED`. Things Claude inferred from context are `LIKELY` or `POSSIBLE` — and `POSSIBLE` fields will nag you for a decision before shipping. If something from the "still figuring out" section shows up, Claude will stop and ask rather than guessing.

Commit the generated contracts and push.

---

## Step 4: Build — Sprint Like Normal, But With Guardrails

**Who**: You, alone with Claude. Everyone sprints independently.
**Time**: Hours to days.

This is a normal `/sprint`. The difference is that your agents now have the contracts and validation config from the previous step. Point them at it:

> "Validate against `contracts/validation.yaml` as you work. Interfaces we own must match the schemas in `contracts/agreed/`. For stuff we consume, use the mocks."

What this means in practice:

- Alice's agents build the modal and forms against the mock API responses — she doesn't need Bob's endpoint to be live yet
- If Bob's agent changes the error response format from `{ error, code }` to `{ message, errorType }`, the schema catches it immediately — not when Alice opens a bug three days later
- If Alice's form starts sending `cancellationReason` instead of `reason`, the request schema flags it
- Fields tagged `POSSIBLE` get flagged before anyone implements them — your agents won't silently build on assumptions that were never actually agreed on

**Work on your own branch.** Don't merge to main until you've run the integration check.

If you realize mid-sprint that something needs to change — maybe Bob discovers Stripe's webhook sends the cancellation confirmation differently than expected — update the contract **and tell the other person**. A quick Slack message: "Hey, `activeUntil` is coming back as a Unix timestamp from Stripe, not ISO 8601. I'm updating the schema." Then Alice adjusts her date formatting and nobody's surprised at merge time.

---

## Step 5: Integrate — Check That Everything Fits

**Who**: One of you (or both).
**Time**: 15-30 minutes.

Both sides done? Run the integration check:

```
/integrate contracts/
```

This dispatches agents to answer three questions:

1. **Does each side match its own contracts?** Does Bob's endpoint actually return `{ cancelledAt, activeUntil, salesforceCaseId }` like the schema says? Does Alice's form actually send `{ reason, details }` like they agreed?

2. **Do the pieces fit together?** Does what Bob sends match what Alice expects to receive? Does Alice's request match what Bob's endpoint accepts?

3. **Did anything drift?** Did someone rename a field, change a type, add a parameter that wasn't in the contract?

You get back a report sorted by severity:

- **FATAL**: This will break at runtime. Bob's endpoint returns `caseId` but Alice's code reads `salesforceCaseId`. Fix before merging.
- **SERIOUS**: This will cause bugs. The error `code` enum has values Alice's error handler doesn't cover. Should fix.
- **ADVISORY**: Cosmetic. Bob added a `cancellationId` field that nobody consumes yet. Track it and move on.

If there are conflicts, `/integrate` proposes specific fixes and names who needs to agree. Resolve, re-sprint the affected piece if needed, re-run `/integrate` until it's clean. Then merge and ship.

---

## The Whole Thing, at a Glance

```
/storm-prep (each person, alone — figure out your piece)
       |
       v
  Jam Session (everyone together — the only sync point)
       |
       v
/formalize (each person, alone — turn agreements into contracts)
       |
       v
  /sprint (each person, alone — build with guardrails)
       |
       v
/integrate (together or one person — verify everything fits)
       |
       v
    Merge & Ship
```

What passes between steps:

| From | To | What Gets Handed Off |
|------|----|----------------------|
| `/storm-prep` | Jam | Each person's list of what they need + what they're assuming |
| Jam | `/formalize` | The shared agreement doc (`docs/jam/session-{date}.md`) |
| `/formalize` | `/sprint` | Schemas, mocks, validation rules |
| `/sprint` | `/integrate` | Your branch + the contracts |
| `/integrate` | Merge | Integration report — go/no-go + fix proposals |

---

## Variations

### Three or More People

Same pipeline, just more people at the jam session. If Alice is on frontend, Bob is on the API, and Carol is on the Salesforce integration — all three prep independently, jam together, and formalize their own piece. The jam session just has more handoffs to agree on: Alice-to-Bob (API shape), Bob-to-Carol (what data to send to Salesforce), Carol-to-Bob (what case ID comes back).

For 4+ people, consider splitting the jam into focused pairs — Alice and Bob hash out the API, Bob and Carol hash out the Salesforce integration — then a quick all-hands to review the full picture.

### Starting From Scratch (No Existing Code)

`/storm-prep` works without a codebase. The agents reason from the project goal instead of code exploration. Your prep output will be more speculative — "we'll probably need an endpoint for X" rather than "the existing endpoint at `src/api/billing.ts:42` already handles this." That's fine. The jam session is where you nail things down regardless.

### Changing a Contract Mid-Sprint

It happens. When it does:

1. Update the contract in `contracts/agreed/`
2. Tell the other person right away
3. They update their side (re-run `/formalize` or manually fix)
4. Both keep sprinting against the updated contract

The one thing you can't do is change it silently. That's how you end up back in the "discover at merge time" failure mode.

### The Jam Didn't Resolve Everything

That's fine. Mark what's unresolved, run `/formalize` anyway (it'll tag those fields as `POSSIBLE`), sprint with what you have, and run `/integrate` to see if the unresolved stuff actually matters. Sometimes it doesn't — the code never touches that field. When it does, run a quick focused jam on just the open questions.

### It's Not Just APIs

The examples here use REST endpoints, but the same pattern works for any handoff between people's work:

- **Shared components**: Alice is building a `<ReasonSelector>` component that Bob's admin dashboard also needs. The jam agrees on the props interface.
- **Database schema**: Alice needs a `cancellation_reasons` table that Bob's analytics queries will join against. The jam agrees on the column names and types.
- **External service contracts**: Bob's Salesforce integration returns data that Carol's reporting pipeline consumes. The jam agrees on what fields the case object includes.
- **Event queues**: Bob publishes a cancellation event that Carol's email service subscribes to. The jam agrees on the message shape.

Anywhere two people's code has to agree on a shape, the workflow applies.

---

## Checklist

```
[ ] 1. Ran /storm-prep for your part
[ ] 2. Committed and pushed your prep file
[ ] 3. Jam session done — sat down and talked through the handoffs
[ ] 4. Shared jam artifact committed
[ ] 5. Ran /formalize — contracts and mocks generated
[ ] 6. Committed and pushed contracts
[ ] 7. Sprint done — your part is built
[ ] 8. Ran /integrate — everything lines up
[ ] 9. Merged
[ ] 10. Shipped
```

---

## See Also

- [Multi-Human Intro](multi-human-intro.md) — the conceptual model and why this works
- [Workflow Pipelines](pipelines.md) — how single-person workflows work
- [Team System Guide](team-system-guide.md) — persistent teams and agent coordination
