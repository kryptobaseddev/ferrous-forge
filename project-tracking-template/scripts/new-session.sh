#!/bin/bash
# new-session.sh - Create a new session notes file from template

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Check if we're in the right directory
if [[ ! -f "sessions/SESSION_TEMPLATE.md" ]]; then
    print_status $RED "❌ Error: SESSION_TEMPLATE.md not found in sessions/ directory"
    echo "   Make sure you're running this script from the project-docs root directory"
    exit 1
fi

# Find the next session number
SESSION_FILES=(sessions/SESSION_*_NOTES.md)
NEXT_SESSION=1

if [[ ${#SESSION_FILES[@]} -gt 0 ]] && [[ -f "${SESSION_FILES[0]}" ]]; then
    # Extract session numbers and find the highest
    for file in "${SESSION_FILES[@]}"; do
        if [[ $file =~ SESSION_([0-9]+)_NOTES\.md ]]; then
            session_num=${BASH_REMATCH[1]}
            if [[ $session_num -ge $NEXT_SESSION ]]; then
                NEXT_SESSION=$((session_num + 1))
            fi
        fi
    done
fi

NEW_SESSION_FILE="sessions/SESSION_${NEXT_SESSION}_NOTES.md"

print_status $BLUE "📝 Creating new session notes..."
print_status $BLUE "   Session number: $NEXT_SESSION"
print_status $BLUE "   File: $NEW_SESSION_FILE"

# Copy template to new session file
cp sessions/SESSION_TEMPLATE.md "$NEW_SESSION_FILE"

# Replace placeholders in the new file
TODAY=$(date +"%Y-%m-%d")
CURRENT_TIME=$(date +"%H:%M")

# Read project name from TODO.json if available
PROJECT_NAME="Your Project"
if [[ -f "TODO.json" ]]; then
    PROJECT_NAME=$(python3 -c "
import json
try:
    with open('TODO.json') as f:
        data = json.load(f)
        print(data.get('projectName', 'Your Project'))
except:
    print('Your Project')
" 2>/dev/null)
fi

# Replace placeholders using sed (works on both macOS and Linux)
sed -i.backup "s/Session {N}/Session $NEXT_SESSION/g" "$NEW_SESSION_FILE"
sed -i.backup "s/YYYY-MM-DD/$TODAY/g" "$NEW_SESSION_FILE"
sed -i.backup "s/X.Y hours/0.0 hours/g" "$NEW_SESSION_FILE"

# Remove backup file created by sed
rm "${NEW_SESSION_FILE}.backup" 2>/dev/null || true

# Get previous session info for context
PREV_SESSION=$((NEXT_SESSION - 1))
PREV_SESSION_FILE="sessions/SESSION_${PREV_SESSION}_NOTES.md"
if [[ $PREV_SESSION -gt 0 ]] && [[ -f "$PREV_SESSION_FILE" ]]; then
    print_status $YELLOW "📋 Previous session: SESSION_${PREV_SESSION}_NOTES.md"
    print_status $YELLOW "   Review the previous session notes before starting work"
fi

# Show current project status if TODO.json exists
if [[ -f "TODO.json" ]]; then
    print_status $BLUE "📊 Current project status:"
    python3 -c "
import json
try:
    with open('TODO.json') as f:
        data = json.load(f)
    metrics = data.get('globalMetrics', {})
    current_phase = data.get('currentPhase', 'Unknown')
    print(f'   Project: {data.get(\"projectName\", \"Unknown\")}')
    print(f'   Phase: {current_phase}')
    print(f'   Tasks: {metrics.get(\"completedTodos\", 0)}/{metrics.get(\"totalTodos\", 0)} completed')
    print(f'   In Progress: {metrics.get(\"inProgressTodos\", 0)}')
    if metrics.get('blockedTodos', 0) > 0:
        print(f'   Blocked: {metrics.get(\"blockedTodos\", 0)} ⚠️')
except Exception as e:
    print('   Unable to read project status')
" 2>/dev/null
fi

print_status $GREEN "✅ Session $NEXT_SESSION notes created successfully!"
print_status $GREEN "   File: $NEW_SESSION_FILE"

# Offer to open the file in an editor
read -p "Would you like to open the session notes now? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Try common editors in order of preference
    if command -v code >/dev/null 2>&1; then
        code "$NEW_SESSION_FILE"
    elif command -v subl >/dev/null 2>&1; then
        subl "$NEW_SESSION_FILE"  
    elif command -v atom >/dev/null 2>&1; then
        atom "$NEW_SESSION_FILE"
    elif command -v nano >/dev/null 2>&1; then
        nano "$NEW_SESSION_FILE"
    elif command -v vim >/dev/null 2>&1; then
        vim "$NEW_SESSION_FILE"
    else
        print_status $YELLOW "⚠️  No supported editor found. Please open the file manually:"
        print_status $YELLOW "   $NEW_SESSION_FILE"
    fi
fi

print_status $BLUE ""
print_status $BLUE "💡 Quick reminders for your session:"
print_status $BLUE "   1. Review HANDOFF.md and previous session notes first"
print_status $BLUE "   2. Update TODO.json task statuses as you work"
print_status $BLUE "   3. Document all decisions and blockers in your session notes"
print_status $BLUE "   4. Run validation before committing: ./scripts/validate-tracking.sh"
print_status $BLUE ""
print_status $BLUE "Happy coding! 🚀"