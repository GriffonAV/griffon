# Release Pipeline Guide

## Purpose

This document defines the release process for the project. Its purpose is to ensure that every release is functionally correct, technically performant, and ready for production. Each release follows the same sequence of validation steps, allowing the team to detect regressions early and maintain a consistent level of quality.

Performance evaluation is integrated throughout the release process rather than being treated as a final verification step.

## Release Workflow

Every release follows the workflow below:

- Complete feature development.
- Verify that all functional requirements have been implemented.
- Execute the complete testing suite.
- Measure the application's technical performance.
- Compare the collected results against the project's performance objectives.
- Investigate and resolve any performance regressions.
- Approve the release only if all quality criteria are satisfied.
- Deploy the validated version.
- Review performance after deployment.

No release should bypass any of these stages.

### Pre-Release Verification

Before a release can begin, developers should ensure that:

- All planned features have been completed.
- Known critical defects have been resolved.
- The codebase is synchronized with the main development branch.
- Any configuration changes have been documented.
- The application builds successfully.

Only after these conditions are met should the release process continue.

### Functional Validation

The first validation step is to confirm that the application behaves as expected.

The development team should execute the project's automated tests and verify that all critical functionality operates correctly. Any failed test must be investigated before proceeding.

The release process must stop until functional issues have been resolved.

### Performance Validation

Once functional correctness has been confirmed, the application's technical performance should be evaluated.

The objective of this stage is to identify any degradation introduced by recent changes.

The team should measure the project's selected performance indicators and compare the results with those obtained from previous releases. Any unexpected increase in resource consumption or reduction in performance should be considered a regression and investigated before deployment.

If performance no longer meets the project's expected standards, the release should be postponed until corrective actions have been implemented.

### Release Approval

A release may only be approved when all validation stages have been successfully completed.

Before deployment, the team should confirm that:

- All functional tests have passed.
- Performance remains within acceptable limits.
- No unresolved critical issues remain.
- The release is considered stable by the development team.

Only after these conditions are satisfied should the release proceed.

### Deployment

After approval, the validated version can be deployed.

The deployment should follow the project's standard deployment procedure to ensure consistency between releases.

Any deployment-specific configuration should be verified before making the application available to users.

### Post-Release Review

Deployment marks the beginning of the monitoring phase rather than the end of the release process.

After each release, the team should observe the application's behaviour to confirm that it performs as expected under normal usage.

If unexpected issues or performance regressions are identified, they should be documented, investigated, and scheduled for correction as part of the next development cycle.

### Continuous Improvement

Each release provides information that can be used to improve future releases.

Following deployment, the team should review the release by considering questions such as:

- Did the application maintain its expected level of performance?
- Were any regressions introduced?
- Could any stage of the release process be improved?
- Are the current performance objectives still appropriate?

Lessons learned should be incorporated into future releases so that the release process evolves alongside the project.

## Release Checklist

Before approving a release, confirm that the following conditions have been met:

Feature implementation is complete.
Functional validation has succeeded.
Technical performance has been evaluated.
No significant performance regressions have been identified.
Outstanding critical issues have been resolved.
The application has been approved for deployment.
Post-release monitoring has been planned.

A release should only proceed when every item in this checklist has been completed.