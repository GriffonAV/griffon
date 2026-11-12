# AI assistance policy

Griffon permits careful, human-directed AI assistance in development. Our priorities
are reliable software, contributor understanding, transparent collaboration, and
respect for maintainers' time. AI is optional, and choosing not to use it must not
exclude someone from contributing.

## Permitted assistance

Contributors may use AI to understand code, explore designs, investigate bugs,
prepare code and tests, and draft or revise documentation. Public-facing text,
including issues and pull request descriptions, may be assisted by AI when a human
checks its accuracy, edits it as needed, and takes responsibility for it.

This policy concerns project development. It does not authorize adding an AI
service to the application or sending application users' data to a model provider.
Generative images, audio and other media require a separate maintainer decision;
the permission for code and text does not automatically extend to media.

## Responsibility and review

- A human contributor owns every submission and must understand the submitted
  changes, explain their purpose and tradeoffs, and maintain them through review.
- AI-assisted work meets the same correctness, security, licensing, testing and
  documentation standards as other work. Do not submit output you cannot review.
- Keep changes focused and reviewable. Do not submit bulk speculative bug reports,
  fabricated references, invented test results or unverified claims as facts.
- An AI review is supporting evidence, not a security certification or a substitute
  for human review. Automated suggestions do not authorize merging or publishing.
- Maintainers may request evidence, revisions or a smaller scope, or decline work
  that is unverifiable or that its contributor cannot explain.

## Disclosure

Disclose AI assistance in the pull request, issue or other contribution where it
was used. Briefly identify the tool and its role, the affected scope, and what you
personally checked. This includes assistance with analysis, code, tests or prose.
Do not include private prompts, credentials or sensitive conversation transcripts.

Example disclosure (replace with facts about your contribution):

> AI assistance: [tool] helped with [analysis/code/tests/text] in [scope].
> I reviewed [parts] and ran [checks and results]. Not verified: [remaining gaps].

## Data and security

Do not transmit secrets, personal files, private logs, malware samples or
confidential vulnerability details to an external AI service without authorization
from the party responsible for that data. Use minimal, sanitized examples.
Check generated code and dependencies for provenance and license compatibility;
AI output does not waive the project's contribution license requirements.

Test destructive behavior only with disposable data in an isolated environment.
Report suspected vulnerabilities privately according to [SECURITY.md](SECURITY.md);
do not automatically turn an audit result into a public issue.

## Community

Communicate in your own judgment and take responsibility for messages you post.
Do not let an agent impersonate contributors or automatically send communications
on their behalf. Respect people's decisions to use or avoid AI; harassment and
pressure are covered by our [Code of Conduct](CODE_OF_CONDUCT.md).

## Related guidance

[AGENTS.md](AGENTS.md) describes agent workflows, while
[CONTRIBUTING.md](CONTRIBUTING.md) applies to every contribution.

Inspired by [Bevy's AI policy](https://bevy.org/learn/contribute/policies/ai/).
Griffon's policy permits human-reviewed AI-assisted code and prose and does not
adopt Bevy's prohibitions wholesale. Policy changes should be reviewed by the
project maintainers like other contribution rules.
