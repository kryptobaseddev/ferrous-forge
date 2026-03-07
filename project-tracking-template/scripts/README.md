# Automation Scripts

This directory contains a complete automation suite to help manage your project tracking system efficiently. **Use the unified `./project-tracker` script in the root directory for the best experience** - it provides a single interface to all these scripts with additional features.

## 🎯 Unified Interface - project-tracker

The recommended way to use these scripts is through the unified `project-tracker` command in the root directory:

```bash
# Single entry point for all functionality
./project-tracker help                     # Show all available commands
./project-tracker status                   # Quick project status
./project-tracker dashboard                # Generate HTML dashboard
./project-tracker validate                 # Validate all files
./project-tracker quick-check              # Fast health check
```

See the main [README.md](../README.md) for complete `project-tracker` documentation.

## Individual Scripts

These scripts can also be used directly if needed:

### 🔍 validate-tracking.sh
**Purpose**: Complete validation system with schema compliance and consistency checking

```bash
# Basic validation
./scripts/validate-tracking.sh

# Through project-tracker (recommended)
./project-tracker validate
```

**Features**:
- **Full JSON validation** with schema compliance checking
- **Consistency checks** between TODO.json and USER_STORIES.json  
- **Placeholder detection** to catch unfinished customization
- **Smart schema location detection** - works from any directory
- **Colored output** with clear error reporting
- **Project integrity checks** - validates task IDs, story IDs, metrics consistency

**What it validates**:
- JSON syntax validity in all tracking files
- Schema compliance against provided JSON schemas
- Project name consistency between files
- Placeholder values that need to be updated (e.g., "Your Project Name")
- Task ID format validation (P1-001, P2-003, etc.)
- User story ID format validation (US-001, US-002, etc.)
- Metrics accuracy (totals match actual counts)
- Cross-references between todos and user stories

**Requirements**:
- Python 3 (for JSON parsing and validation logic)
- `jsonschema` package (optional, for schema validation)
  ```bash
  pip install jsonschema
  ```

### 📊 generate-report.py
**Purpose**: Advanced reporting engine with multiple output formats and comprehensive analytics

```bash
# Direct usage
python3 scripts/generate-report.py --format=html --output=dashboard.html

# Through project-tracker (recommended)
./project-tracker report html                    # HTML dashboard
./project-tracker report json                    # JSON for APIs
./project-tracker dashboard                      # HTML + auto-open
./project-tracker daily-standup                  # Focused daily report
```

**Features**:
- **4 output formats**: Text, Markdown, HTML dashboard, JSON
- **Comprehensive metrics**: Progress bars, completion rates, velocity tracking
- **Risk & blocker identification** with prioritization and impact analysis
- **Milestone tracking** with overdue detection and timeline visualization
- **Team analytics** and effort estimation with velocity calculations
- **API-ready JSON output** for integration with external tools
- **Interactive HTML dashboards** with charts and visual indicators
- **Automated insights** and recommendations based on project health

**Report sections include**:
- **Executive Summary**: High-level project health and key metrics
- **Phase Progress**: Current phase completion with detailed breakdowns
- **Task Analytics**: Completion statistics with visual progress bars
- **User Story Tracking**: Story completion rates and epic distribution
- **Risk Management**: Active risks, blockers, and mitigation strategies
- **Timeline Analysis**: Upcoming milestones and overdue items (next 30 days)
- **Team Performance**: Velocity trends, capacity analysis, and workload distribution
- **Quality Metrics**: Test coverage, review status, and technical debt indicators

### 📝 new-session.sh
**Purpose**: Intelligent session management with smart templates and context awareness

```bash
# Direct usage
./scripts/new-session.sh

# Through project-tracker (recommended)
./project-tracker new-session
```

**Features**:
- **Auto-incrementing session numbers** with conflict detection and validation
- **Smart template filling** with current date, time, and project information
- **Previous session context** display for seamless continuity
- **Current project status** summary extracted from JSON data
- **Editor auto-launch** with intelligent editor detection and preferences
- **Workflow reminders** with built-in best practices and next steps
- **Session history tracking** with quick access to previous sessions
- **Project health indicators** shown during session creation

**Smart capabilities**:
- Automatically extracts project name, version, and phase from TODO.json
- Shows task completion status and any blockers before starting work
- Provides context from the most recent session notes
- Offers to open session file in your preferred editor immediately
- Includes helpful workflow reminders and next steps

**Supported editors** (auto-detected in order of preference):
- VS Code (`code`) - Full IDE with markdown preview
- Sublime Text (`subl`) - Fast text editor with syntax highlighting
- Atom (`atom`) - Hackable editor with markdown support  
- Nano (`nano`) - Simple terminal-based editor
- Vim (`vim`) - Powerful terminal editor

### 📈 update-metrics.py
**Purpose**: Comprehensive metrics calculation and synchronization engine

```bash
# Direct usage
python3 scripts/update-metrics.py --verbose

# Through project-tracker (recommended)
./project-tracker update-metrics                 # Update all metrics
./project-tracker update-metrics --dry-run       # Preview changes
./project-tracker fix                            # Auto-fix common issues
```

**Features**:
- **Automatic metric calculation** from current task and story state
- **Comprehensive coverage**: All counters, percentages, and distributions
- **Dry-run mode** to preview changes safely before applying
- **Verbose output** showing detailed calculation breakdowns
- **Tag organization** and cleanup with smart categorization
- **Overdue detection** and phase progress tracking with timeline analysis
- **Cross-validation** between TODO.json and USER_STORIES.json metrics
- **Smart defaults** for missing or invalid data

**Automatically calculates and updates**:
- **Task Analytics**: Counts by status (completed, in-progress, blocked, not-started)
- **Priority Distribution**: Critical, high, medium, low priority item counts
- **Progress Metrics**: Completion percentages and phase progress tracking
- **Effort Analysis**: Estimated vs actual hours with variance tracking
- **User Story Metrics**: Story completion rates and velocity calculations
- **Epic Distribution**: Stories per epic with effort point analysis
- **Timeline Tracking**: Overdue task identification and milestone progress
- **Team Analytics**: Workload distribution and capacity utilization
- **Quality Indicators**: Coverage gaps and consistency metrics
- **Tag Management**: Automatic categorization and cleanup of task tags

## Setup and Permissions

### Make scripts executable:
```bash
chmod +x scripts/*.sh
```

### Install Python dependencies:
```bash
# Essential dependencies for full functionality
pip install jsonschema           # Required for schema validation

# Optional: For advanced reporting features in future versions
pip install matplotlib seaborn   # Charts and data visualization
```

## 🔧 Production-Ready Features

### **Error Handling & Reliability**:
- ✅ **Graceful failures** with helpful error messages and recovery suggestions
- ✅ **Input validation** prevents corruption of tracking files with pre-flight checks
- ✅ **Dry-run modes** provide safe preview of changes before applying
- ✅ **Verbose logging** with detailed output for debugging and troubleshooting
- ✅ **Cross-platform** compatibility - tested on Linux, macOS, Windows (WSL)

### **Integration & Automation**:
- ✅ **CI/CD ready** with GitHub Actions, Git hooks, and pipeline examples
- ✅ **API-first design** with JSON output format for dashboards and external tools
- ✅ **Webhook support** for Slack, Discord, Microsoft Teams integration
- ✅ **Cron automation** examples for daily/weekly automated reporting
- ✅ **Multi-format output** supports Text, Markdown, HTML dashboards, JSON APIs

### **Developer Experience**:
- ✅ **Smart defaults** with auto-detection of project info and settings
- ✅ **Editor integration** with automatic file opening in preferred editors
- ✅ **Progress tracking** featuring visual progress bars and status indicators
- ✅ **Context awareness** showing previous session info and current project status
- ✅ **Workflow guidance** with built-in reminders and best practices

## Integration Examples

### Git Hooks Integration
Add to `.git/hooks/pre-commit`:
```bash
#!/bin/sh
echo "Validating project tracking files..."
./project-docs/scripts/validate-tracking.sh
if [ $? -ne 0 ]; then
    echo "❌ Project tracking validation failed"
    exit 1
fi
```

### CI/CD Pipeline Integration
Add to GitHub Actions (`.github/workflows/validate-tracking.yml`):
```yaml
name: Validate Project Tracking
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Python
        uses: actions/setup-python@v3
        with:
          python-version: '3.x'
      - name: Install dependencies
        run: pip install jsonschema
      - name: Validate tracking files
        run: ./project-docs/scripts/validate-tracking.sh
      - name: Generate status report
        run: python3 project-docs/scripts/generate-report.py --format=markdown --output=status-report.md
      - name: Upload report
        uses: actions/upload-artifact@v3
        with:
          name: project-status-report
          path: status-report.md
```

### Daily Automation
Add to cron for daily reports:
```bash
# Daily status report at 9 AM
0 9 * * * cd /path/to/project && python3 project-docs/scripts/generate-report.py --format=html --output=/var/www/html/project-status.html
```

### Slack Integration
Send reports to Slack:
```bash
#!/bin/bash
# slack-report.sh
REPORT=$(python3 scripts/generate-report.py --format=markdown)
curl -X POST -H 'Content-type: application/json' \
  --data "{\"text\":\"Daily Project Report\n\`\`\`\n${REPORT}\n\`\`\`\"}" \
  YOUR_SLACK_WEBHOOK_URL
```

## Customization

### Adding Custom Validations
Edit `validate-tracking.sh` to add project-specific checks:
```bash
# Add custom validation logic here
if grep -q "TODO" TODO.json; then
    errors+=("Found TODO placeholder in TODO.json")
fi
```

### Custom Report Sections
Modify `generate-report.py` to add project-specific metrics:
```python
def generate_custom_metrics(self) -> Dict[str, Any]:
    # Add your custom calculations here
    return {"custom_metric": calculated_value}
```

### Environment-Specific Scripts
Create environment-specific versions:
```bash
cp scripts/generate-report.py scripts/generate-report-production.py
# Modify for production-specific metrics
```

## Troubleshooting

### Common Issues

**Permission Denied**:
```bash
chmod +x scripts/*.sh
```

**Python Module Not Found**:
```bash
pip install jsonschema
# or
python3 -m pip install jsonschema
```

**Schema Files Not Found**:
- Ensure you're running from the correct directory
- Check that `schemas/` directory exists
- Scripts auto-detect schema location in common paths

**JSON Validation Errors**:
- Check for trailing commas (not allowed in JSON)
- Verify all quotes are properly closed
- Use `python3 -m json.tool file.json` to identify syntax issues

### Debug Mode
Enable debug output for troubleshooting:
```bash
# For shell scripts
bash -x scripts/validate-tracking.sh

# For Python scripts  
python3 -v scripts/generate-report.py
```

## Contributing

### Adding New Scripts
1. Follow the existing naming convention (`kebab-case.sh` or `kebab-case.py`)
2. Add comprehensive help text and error handling
3. Include usage examples in comments
4. Update this README with the new script documentation
5. Test with both valid and invalid inputs

### Best Practices
- Always validate inputs and provide helpful error messages
- Support both dry-run and verbose modes where applicable
- Use consistent color coding for output (red=error, yellow=warning, green=success, blue=info)
- Include progress indicators for long-running operations
- Provide clear usage examples in help text

---

These scripts are designed to make project tracking effortless and reliable. They integrate seamlessly with your development workflow and provide valuable insights into project health and progress.