# Setup Guide - Project Tracking Template

This guide will help you set up the project tracking system for your project in 15 minutes.

## Quick Setup (5 minutes)

### Step 1: Copy Template
```bash
# Copy the entire template directory to your project
cp -r project-tracking-template/ /path/to/your-project/project-docs/

# Or create as a separate repository
git clone [this-template] your-project-tracking
```

### Step 2: Basic Customization
Edit these files with your project information:

1. **TODO.json** - Replace placeholder values:
   - `projectName`: "Your Project Name" → "Actual Project Name"
   - `currentPhase`: Update to your starting phase
   - `phases`: Customize phase names and descriptions
   - `teamMembers`: Add your actual team members

2. **USER_STORIES.json** - Update project info:
   - `projectName`: Match the name from TODO.json
   - `stories`: Replace example stories with your actual requirements
   - `personas`: Define your actual user types

3. **HANDOFF.md** - Fill in project-specific sections:
   - Project name and version information
   - Technology stack details
   - Key stakeholder contacts
   - Quality gates and validation commands

### Step 3: Start Using
1. **Test the unified interface**: `./project-tracker help`
2. **Validate your setup**: `./project-tracker validate`
3. **Create your first session**: `./project-tracker new-session`
4. **Generate a dashboard**: `./project-tracker dashboard`
5. Begin following the handoff protocol from HANDOFF.md

## Detailed Setup (15 minutes)

### Project Information Setup

1. **Update Project Metadata**
   ```json
   // In both TODO.json and USER_STORIES.json
   {
     "projectName": "My Awesome Project",
     "projectVersion": "0.1.0",
     "lastUpdated": "2024-01-15"
   }
   ```

2. **Configure Team Members**
   ```json
   // In TODO.json
   "teamMembers": [
     {
       "name": "Alice Developer",
       "role": "Lead Developer", 
       "email": "alice@company.com",
       "expertise": ["frontend", "react", "typescript"],
       "availability": "full-time"
     }
   ]
   ```

### Phase and Task Setup

1. **Define Your Development Phases**
   ```json
   // Example phases - customize for your project
   "PHASE_1": {
     "name": "MVP Development",
     "description": "Core features needed for minimum viable product",
     "priority": "CRITICAL"
   }
   ```

2. **Create Initial Tasks**
   ```json
   {
     "id": "P1-001",
     "title": "Setup Development Environment",
     "feature": "infrastructure",
     "priority": "CRITICAL",
     "estimatedHours": 4,
     "assignee": "Alice Developer"
   }
   ```

### User Stories Setup  

1. **Define Your Personas**
   ```json
   {
     "name": "End User",
     "description": "Primary users of the application",
     "characteristics": ["Non-technical", "Mobile-first", "Time-conscious"]
   }
   ```

2. **Create Your First Stories**
   ```json
   {
     "id": "US-001", 
     "title": "User Login",
     "persona": "End User",
     "story": "As an end user, I want to log in securely so that I can access my personal data",
     "effort": 5
   }
   ```

### Quality Gates Configuration

1. **Set Project Standards**
   ```json
   // In TODO.json configuration section
   "qualityGateRequirements": {
     "testCoverage": 85,
     "codeReviewRequired": true,
     "documentationRequired": true
   }
   ```

2. **Define Validation Commands**
   ```bash
   # Add to HANDOFF.md
   npm test                # Run your test suite
   npm run lint           # Run your linter
   npm run build          # Verify build works
   ```

## Integration Options

### With Existing Tools

#### GitHub Integration
- Use GitHub Issues API to sync with TODO.json
- Create GitHub Actions to validate JSON schemas
- Use labels that match your tags system

#### Jira/Linear Integration  
- Map story IDs to Jira tickets
- Use webhooks to sync status updates
- Export reports to external systems

#### Slack Integration
- Bot notifications for task status changes
- Daily standup reports from JSON data
- Risk alerts for overdue or blocked items

### Complete Automation Suite

The template includes a **complete automation suite** with production-ready scripts:

#### 🎯 Unified Interface - project-tracker
```bash
# Single entry point for all operations
./project-tracker help                    # Show all commands
./project-tracker status                  # Quick project status
./project-tracker validate                # Validate all files
./project-tracker dashboard               # Generate HTML dashboard
./project-tracker new-session             # Create session notes
./project-tracker update-metrics          # Sync all metrics
./project-tracker report json             # JSON output for APIs
./project-tracker quick-check             # Fast health check
```

#### 🔍 Advanced Validation - validate-tracking.sh
- Complete JSON validation with schema compliance
- Consistency checks between all tracking files
- Placeholder detection for unfinished setup
- Smart schema location detection
- Colored output with clear error reporting

#### 📊 Multi-Format Reporting - generate-report.py
- 4 output formats: Text, Markdown, HTML dashboards, JSON APIs
- Comprehensive metrics with progress bars and analytics
- Risk & blocker identification with prioritization
- Milestone tracking with overdue detection
- Team velocity analysis and capacity planning

#### 📝 Smart Session Management - new-session.sh
- Auto-incrementing session numbers with conflict detection
- Smart template filling with current project context
- Previous session context display for continuity
- Editor auto-launch with multiple editor support
- Workflow reminders and best practices built-in

#### 📈 Metrics Synchronization - update-metrics.py
- Automatic calculation from current task/story state
- Comprehensive coverage of all counters and percentages
- Dry-run mode for safe preview of changes
- Tag organization and cleanup automation
- Overdue detection and phase progress tracking

### 🔧 Production-Ready Features

#### **Error Handling & Reliability**:
- ✅ **Graceful failures** with helpful error messages and recovery suggestions
- ✅ **Input validation** prevents corruption of tracking files with pre-flight checks
- ✅ **Dry-run modes** provide safe preview of changes before applying
- ✅ **Verbose logging** with detailed output for debugging and troubleshooting
- ✅ **Cross-platform** compatibility - tested on Linux, macOS, Windows (WSL)

#### **Integration & Automation**:
- ✅ **CI/CD ready** with GitHub Actions, Git hooks, and pipeline examples
- ✅ **API-first design** with JSON output format for dashboards and external tools
- ✅ **Webhook support** for Slack, Discord, Microsoft Teams integration
- ✅ **Cron automation** examples for daily/weekly automated reporting
- ✅ **Multi-format output** supports Text, Markdown, HTML dashboards, JSON APIs

#### **Perfect for AI/LLM Agents**:
- ✅ **Pure CLI operation** - no user input required for any command
- ✅ **Deterministic output** - same command always produces same result
- ✅ **Exit codes** - 0 for success, 1 for failure for reliable scripting
- ✅ **Machine-readable** - JSON format for structured data processing
- ✅ **Scriptable** - commands can be chained together seamlessly

#### **Developer Experience**:
- ✅ **Smart defaults** with auto-detection of project info and settings
- ✅ **Editor integration** with automatic file opening in preferred editors
- ✅ **Progress tracking** featuring visual progress bars and status indicators
- ✅ **Context awareness** showing previous session info and current project status
- ✅ **Workflow guidance** with built-in reminders and best practices

## Advanced Configuration

### Custom Fields
Add project-specific fields to the schemas:

```json
// In todo-schema.json, add to todo definition
"customFields": {
  "type": "object",
  "properties": {
    "complexity": {
      "type": "string",
      "enum": ["simple", "moderate", "complex"]
    },
    "testingRequired": {
      "type": "boolean"
    }
  }
}
```

### Reporting Dashboards
Create automated reports:

1. **Sprint Burndown Charts** from effort tracking
2. **Risk Heat Maps** from risk register data  
3. **Velocity Tracking** from completed story points
4. **Quality Metrics** from test coverage and review data

### Workflow Automation
Set up automation for:

1. **Status Updates**: Automatically move tasks based on git commits
2. **Risk Monitoring**: Alert when tasks are overdue or blocked
3. **Metrics Collection**: Gather data from build systems and test results
4. **Stakeholder Reports**: Generate executive summaries automatically

## Best Practices

### File Management
- Keep TODO.json and USER_STORIES.json in sync
- Update lastUpdated timestamps when making changes
- Validate JSON regularly using the provided schemas
- Back up tracking files before major changes

### Team Adoption  
- Start with a small pilot team
- Provide training on the handoff protocol
- Make templates easy to use and modify
- Celebrate early wins and improvements

### Maintenance
- Review and update personas quarterly
- Archive completed phases but preserve history
- Regular retrospectives on tracking effectiveness
- Continuous improvement of templates and processes

### Quality Control
- Enforce schema validation in CI/CD
- Regular reviews of story quality and completeness
- Consistent effort estimation calibration
- Documentation of all architectural decisions

## Troubleshooting

### Common Issues

**JSON Validation Errors**
- Check for trailing commas (not allowed in JSON)
- Verify all required fields are present
- Ensure ID formats match regex patterns (P1-001, US-001)

**Schema Mismatches**
- Update schema versions when adding custom fields
- Test changes in a copy before applying to main files
- Use online JSON schema validators for debugging

**Team Adoption Problems**
- Simplify initial templates if too complex
- Focus on high-value documentation first
- Provide clear examples and success stories
- Regular check-ins on process effectiveness

**Tool Integration Issues**
- Start with manual processes before automating
- Test integrations with small data sets first
- Document API limits and rate limiting
- Plan for tool changes and migrations

## Support and Community

### Getting Help
- Check the README.md for conceptual overview
- Review session templates for usage examples
- Validate JSON files against provided schemas
- Test with small projects before full implementation

### Contributing Improvements
- Submit schema enhancements via pull requests
- Share automation scripts and integration examples  
- Document lessons learned and best practices
- Help improve templates based on real-world usage

---

**Remember**: This system grows with your project. Start simple, validate effectiveness, then enhance based on actual team needs and feedback.