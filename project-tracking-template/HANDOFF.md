# Project Handoff Document - Universal Template

## 🚨 CRITICAL: Read This First
This document contains **IMMUTABLE INSTRUCTIONS** that must be followed by every team member, agent, or contributor working on this project. Failure to follow these instructions will result in wasted work, project regression, and workflow disruption.

---

## Session Reference
- **Latest session notes**: `project-docs/sessions/SESSION_*_NOTES.md` - **READ THE LAST SESSION NOTES BEFORE STARTING**
- **Up-to-date backlog**: `project-docs/TODO.json` - Review for current phase and priorities
- **User story catalogue**: `project-docs/USER_STORIES.json` - Understand the user requirements
- **Project configuration**: Check `TODO.json` configuration section for project-specific rules

## Standing Instructions

### 1. Session Start Protocol
**MANDATORY ACTIONS:**
1. Read the previous session notes from `project-docs/sessions/`
2. Review `TODO.json` for current phase and task status
3. Check project-specific quality gates and standards (defined in `TODO.json` configuration)
4. Verify you're on the correct branch (check project documentation)
5. Pull latest changes from the main development branch
6. Run project validation/build commands to ensure clean starting state

### 2. Development Guidelines
**DO NOT:**
- Modify the existing TODO.json or USER_STORIES.json schemas without explicit instruction
- Create features outside the current phase defined in TODO.json
- Work on tasks not assigned to you or not in the current sprint/phase
- Introduce breaking changes without proper documentation and team consensus
- Skip quality gates defined in the project configuration
- Work on tasks marked as BLOCKED without resolving dependencies first

**ALWAYS:**
- Follow the project's defined coding standards and conventions
- Update task statuses in TODO.json as you progress through work
- Document all public APIs according to project standards
- Write tests for new functionality according to project requirements
- Commit changes incrementally with clear, descriptive messages
- Follow the established git workflow and branching strategy

### 3. Quality Standards
**ENFORCED REQUIREMENTS** (customize per project):
- All code must pass defined quality gates before commit
- Test coverage must meet minimum thresholds (defined in configuration)
- All public functions/methods must be documented
- Code must pass linting and static analysis tools
- Security scans must pass (if applicable to project)
- Performance benchmarks must not regress (if applicable)

### 4. Task Management
**CURRENT PHASE**: Check `TODO.json` → `currentPhase` field
- Only work on todos from the current phase unless explicitly instructed
- Complete tasks according to their priority: CRITICAL → HIGH → MEDIUM → LOW
- Update todo status in TODO.json as you progress: NOT_STARTED → IN_PROGRESS → COMPLETED
- Document blockers immediately when encountered
- Don't start new phases without team/stakeholder approval

### 5. Communication Standards
**STATUS UPDATES:**
- Update TODO.json immediately when starting work on a task (set to IN_PROGRESS)
- Update TODO.json immediately when completing a task (set to COMPLETED)
- Add notes to todos when significant decisions are made or blockers encountered
- Document any scope changes or requirement clarifications

**DECISION DOCUMENTATION:**
- All architectural decisions must be documented in session notes
- Technical trade-offs must include rationale and alternatives considered
- Breaking changes must be documented with migration strategies
- Dependencies added/removed must be documented with justification

### 6. Session Documentation Requirements
**MANDATORY SESSION NOTES:**
- Create new file: `SESSION_{N}_NOTES.md` where N is the next session number
- Include: Date, Duration, Team Members, Completed Tasks, Decisions, Risks, Next Steps
- Be specific about what was attempted, what worked, and what failed
- Document any changes to project requirements or technical approach
- Include performance metrics or quality measurements if applicable

**SESSION NOTE SECTIONS** (required):
```markdown
# Session {N} Notes - {Brief Description}
**Date**: YYYY-MM-DD
**Duration**: X hours
**Participants**: List of team members involved
**Focus**: Brief description of session goals

## Tasks Completed
- [x] Task ID: Brief description of what was completed
- [x] Task ID: Include any important technical details

## Decisions Made  
- Decision 1: Rationale and alternatives considered
- Decision 2: Impact on project timeline or architecture

## Blockers & Risks
- Blocker 1: Description and proposed resolution
- Risk 1: Identified risk and mitigation strategy

## Next Steps
- Specific actionable items for next session
- Any dependencies that need to be resolved
- Stakeholder decisions that are needed
```

### 7. Git Workflow
**REQUIRED PRACTICES:**
1. Work on feature branches derived from main development branch
2. Commit frequently with descriptive messages following project convention
3. Use consistent commit message format: `type(scope): description`
4. Never commit directly to main/production branches
5. Always create pull/merge requests for code review
6. Ensure all tests pass before creating pull requests

**COMMIT MESSAGE FORMATS** (adapt to project):
- `feat(component): add new functionality`
- `fix(component): resolve specific issue`
- `docs(section): update documentation`
- `test(component): add or update tests`
- `refactor(component): improve code without changing functionality`
- `chore(area): maintenance and tooling updates`

### 8. Testing Protocol
**BEFORE COMMITTING** (customize per project):
```bash
# Run project-specific validation commands
# Examples - adapt to your project:
npm test                    # Run test suite
npm run lint               # Check code quality
npm run build              # Ensure project builds
npm run security-check     # Run security analysis (if applicable)
```

### 9. Handoff Protocol
**WHEN COMPLETING WORK:**
1. Update all relevant TODO.json task statuses to COMPLETED
2. Create comprehensive session notes documenting work performed
3. Commit all changes with clear messages
4. Push changes to remote branch
5. Update any project metrics or dashboards
6. Document any decisions needed from stakeholders or other team members
7. Ensure the project is in a clean state for the next team member

**WHEN BLOCKED:**
1. Document the specific blocker in TODO.json task notes
2. Set task status to BLOCKED with clear explanation
3. Identify who can resolve the blocker and when
4. Work on other available tasks while waiting for resolution
5. Escalate blockers that significantly impact project timeline

---

## Project-Specific Configuration

### Current Project State
- **Project Name**: [Fill in project name]
- **Current Version**: [Fill in current version]
- **Target Version**: [Fill in target version]
- **Development Branch**: [Fill in main development branch name]
- **Production Branch**: [Fill in production branch name]

### Technology Stack
- **Languages**: [List programming languages]
- **Frameworks**: [List frameworks and major libraries]
- **Build Tools**: [List build system and tools]
- **Testing Framework**: [List testing tools and requirements]
- **Deployment Platform**: [List deployment targets]

### Key Stakeholders
- **Project Owner**: [Name and contact]
- **Technical Lead**: [Name and contact]  
- **QA Lead**: [Name and contact]
- **DevOps Contact**: [Name and contact]

### External Dependencies
- **Required Services**: [List external services/APIs]
- **Environment Requirements**: [Development environment setup]
- **Access Requirements**: [Credentials, VPN, etc.]

### Performance & Quality Metrics
- **Build Time Target**: [Expected build duration]
- **Test Coverage Minimum**: [Required test coverage percentage]
- **Performance Benchmarks**: [Response time, throughput requirements]
- **Code Quality Gates**: [Linting rules, complexity limits]

---

## Emergency Procedures

### If You Encounter Critical Issues
1. **STOP** work immediately to avoid further impact
2. Document the issue with steps to reproduce
3. Revert to last known good state if possible
4. Contact technical lead or project owner immediately
5. Create incident documentation in session notes

### If Project State is Unclear
1. **DO NOT** make assumptions or proceed with uncertain work
2. Review the most recent 2-3 session notes for context
3. Check git history for recent changes and their rationale
4. Contact the person who worked on the last session
5. Wait for clarification rather than potentially creating more issues

### If Requirements Change During Work
1. **PAUSE** current work to assess impact
2. Document the requirement change request
3. Estimate impact on current sprint/phase
4. Get stakeholder approval for scope/timeline changes
5. Update TODO.json and USER_STORIES.json accordingly

---

## Quick Reference Commands

```bash
# Project Setup (customize for your project)
git clone [repository-url]
cd [project-directory]
# [Add project-specific setup commands]

# Daily Workflow
git status                  # Check current state
git pull origin [branch]    # Get latest changes
# [Add project-specific development commands]
git add -A                  # Stage changes
git commit -m "message"     # Commit with description
git push origin [branch]    # Push changes

# Quality Checks
# [Add project-specific quality check commands]

# Common Project Commands
# [Add frequently used project-specific commands]
```

---

## Project Directory Structure

```
project-root/
├── project-docs/           # This tracking system (add to .gitignore if desired)
│   ├── TODO.json          # Task tracking
│   ├── USER_STORIES.json  # Requirements tracking  
│   ├── HANDOFF.md         # This document
│   └── sessions/          # Session documentation
├── [project-specific directories]
│   ├── src/               # Source code (example)
│   ├── tests/             # Test files (example)
│   ├── docs/              # Public documentation (example)
│   └── config/            # Configuration files (example)
├── README.md              # Project overview
├── .gitignore            # Git ignore rules
└── [project configuration files]
```

---

## ⚠️ FINAL WARNINGS

1. **NEVER** work without reading the last session notes
2. **NEVER** introduce breaking changes without proper documentation
3. **NEVER** skip quality gates defined in project configuration
4. **NEVER** work on tasks outside the current phase without approval
5. **ALWAYS** maintain the project in a clean, buildable state
6. **ALWAYS** document decisions and changes thoroughly
7. **ALWAYS** update TODO.json status as work progresses
8. **ALWAYS** create session notes when completing work

---

**Remember**: This template is designed for consistency across projects. Customize the project-specific sections while maintaining the core workflow and quality standards. The goal is systematic, high-quality progress with complete traceability and seamless team handoffs.