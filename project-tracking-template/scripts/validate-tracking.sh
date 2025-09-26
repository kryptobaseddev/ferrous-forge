#!/bin/bash
# validate-tracking.sh - Validate project tracking JSON files against schemas

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

print_status $BLUE "🔍 Validating Project Tracking Files..."
echo

# Check if required files exist
if [[ ! -f "TODO.json" ]]; then
    print_status $RED "❌ TODO.json not found in current directory"
    exit 1
fi

if [[ ! -f "USER_STORIES.json" ]]; then
    print_status $RED "❌ USER_STORIES.json not found in current directory"
    exit 1
fi

# Check if schema files exist
SCHEMA_DIR="schemas"
if [[ ! -d "$SCHEMA_DIR" ]]; then
    print_status $YELLOW "⚠️  Schema directory not found, looking for schemas in parent directories..."
    # Try to find schemas in common locations
    if [[ -f "../schemas/todo-schema.json" ]]; then
        SCHEMA_DIR="../schemas"
    elif [[ -f "project-docs/schemas/todo-schema.json" ]]; then
        SCHEMA_DIR="project-docs/schemas"
    else
        print_status $RED "❌ Schema files not found. Please ensure schemas/ directory exists."
        exit 1
    fi
fi

# Function to validate JSON syntax
validate_json_syntax() {
    local file=$1
    if ! python3 -m json.tool "$file" > /dev/null 2>&1; then
        print_status $RED "❌ $file has invalid JSON syntax"
        python3 -m json.tool "$file" 2>&1 | head -5
        return 1
    fi
    return 0
}

# Function to validate against schema (if jsonschema is available)
validate_against_schema() {
    local json_file=$1
    local schema_file=$2
    
    if command -v jsonschema &> /dev/null; then
        if jsonschema -i "$json_file" "$schema_file" 2>/dev/null; then
            return 0
        else
            print_status $RED "❌ $json_file failed schema validation"
            jsonschema -i "$json_file" "$schema_file" 2>&1 | head -10
            return 1
        fi
    else
        print_status $YELLOW "⚠️  jsonschema command not found. Install with: pip install jsonschema"
        print_status $YELLOW "   Skipping schema validation, only checking JSON syntax."
        return 0
    fi
}

# Validate TODO.json
print_status $BLUE "Validating TODO.json..."
if validate_json_syntax "TODO.json"; then
    print_status $GREEN "✅ TODO.json has valid JSON syntax"
    
    if validate_against_schema "TODO.json" "$SCHEMA_DIR/todo-schema.json"; then
        print_status $GREEN "✅ TODO.json passes schema validation"
    fi
else
    exit 1
fi

# Validate USER_STORIES.json
print_status $BLUE "Validating USER_STORIES.json..."
if validate_json_syntax "USER_STORIES.json"; then
    print_status $GREEN "✅ USER_STORIES.json has valid JSON syntax"
    
    if validate_against_schema "USER_STORIES.json" "$SCHEMA_DIR/user-stories-schema.json"; then
        print_status $GREEN "✅ USER_STORIES.json passes schema validation"
    fi
else
    exit 1
fi

# Additional validation checks
print_status $BLUE "Running additional validation checks..."

# Check for required fields and common issues
python3 -c "
import json
import sys

# Load TODO.json
with open('TODO.json') as f:
    todo_data = json.load(f)

# Load USER_STORIES.json  
with open('USER_STORIES.json') as f:
    stories_data = json.load(f)

errors = []

# Check project name consistency
if todo_data.get('projectName') != stories_data.get('projectName'):
    errors.append('Project names do not match between TODO.json and USER_STORIES.json')

# Check for placeholder values that should be updated
placeholders = ['Your Project Name', 'YYYY-MM-DD', 'team-member-name', 'x.y.z']
todo_str = json.dumps(todo_data)
for placeholder in placeholders:
    if placeholder in todo_str:
        errors.append(f'TODO.json contains placeholder value: {placeholder}')

stories_str = json.dumps(stories_data)
for placeholder in placeholders:
    if placeholder in stories_str:
        errors.append(f'USER_STORIES.json contains placeholder value: {placeholder}')

# Check for empty or invalid todo IDs
for phase_key, phase in todo_data.get('phases', {}).items():
    for todo in phase.get('todos', []):
        todo_id = todo.get('id', '')
        if not todo_id.startswith('P') or '-' not in todo_id:
            errors.append(f'Invalid TODO ID format: {todo_id}')

# Check for empty or invalid story IDs
for story in stories_data.get('stories', []):
    story_id = story.get('id', '')
    if not story_id.startswith('US-'):
        errors.append(f'Invalid User Story ID format: {story_id}')

# Check metrics consistency in TODO.json
metrics = todo_data.get('globalMetrics', {})
total = metrics.get('totalTodos', 0)
completed = metrics.get('completedTodos', 0) 
in_progress = metrics.get('inProgressTodos', 0)
not_started = metrics.get('notStartedTodos', 0)

if completed + in_progress + not_started != total:
    errors.append('TODO metrics are inconsistent (total does not match sum of status counts)')

if errors:
    print(f'\n❌ Found {len(errors)} validation issues:')
    for error in errors:
        print(f'   • {error}')
    sys.exit(1)
else:
    print('✅ All additional validation checks passed')
"

if [[ $? -eq 0 ]]; then
    print_status $GREEN ""
    print_status $GREEN "🎉 All validation checks passed!"
    print_status $GREEN "Your project tracking files are valid and ready to use."
else
    print_status $RED ""
    print_status $RED "❌ Validation failed. Please fix the issues above before proceeding."
    exit 1
fi