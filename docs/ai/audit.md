# Bug audit procedure

## Scope and method

1. Identify the requested component or diff, current revision, working-tree changes,
   and any baseline supplied by the user. State exclusions. If no target is given,
   choose and state a bounded starting component; do not claim a whole-project audit.
2. Read the relevant [architecture](architecture.md), implementation, callers,
   tests and existing resilience notes. Trace inputs to observable outcomes.
3. Prioritize concrete failures: data loss, privilege misuse, IPC/ABI mismatch,
   invalid paths, races, hangs, crashes and incorrect results. Style preferences
   and hypothetical problems without a reachable trigger are not bug findings.
4. Check whether existing validation, callers or tests invalidate the hypothesis.
   Identify the trigger and affected configuration. For diff reviews, distinguish
   introduced regressions from pre-existing problems.
5. Where useful, reproduce with a harmless fixture or focused test following
   [QA](qa.md). Do not change production code during an audit unless asked to fix it.
   Keep any temporary reproduction separate and disclose persistent test changes.
6. Report verified findings first. Keep plausible but unresolved hypotheses in a
   separate section stating the missing evidence. Deduplicate known issues.

## Report format

Start with scope, revision and checks actually performed. For each finding include:

- **Title and priority:** P0 immediate critical impact; P1 high impact needing
  prompt attention; P2 ordinary actionable defect; P3 low-impact defect. Base
  priority on demonstrated impact and prerequisites, independently of certainty.
- **Location:** exact file, symbol and current line references.
- **Trigger:** configuration, input and reachable steps.
- **Expected / actual:** observable behavior and who or what is affected.
- **Evidence:** minimal reproduction and observed result, or a complete code path.
- **Confidence:** reproduced, established by code inspection, or unconfirmed.
- **Suggested fix and regression check:** enough detail to guide a subsequent fix.

End with checks not run, limitations and open questions. If there are no supported
findings, say so; this does not mean the application is bug-free. Do not manufacture
findings to fill a quota or claim an automated review certifies security.

Follow [SECURITY.md](../../SECURITY.md) for suspected vulnerabilities. Prepare a
private report; do not publish or send it automatically.

## Example request

“Follow docs/ai/audit.md to audit the scanner quarantine paths. Produce a report
with code references and harmless reproduction steps. Do not change production code.”
