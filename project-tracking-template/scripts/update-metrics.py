#!/usr/bin/env python3
"""
update-metrics.py - Automatically update metrics in TODO.json and USER_STORIES.json

This script recalculates all metrics based on the current state of tasks and stories,
ensuring the globalMetrics section is always accurate and up-to-date.

Usage:
    python3 update-metrics.py [--dry-run] [--verbose]
    
Options:
    --dry-run    Show what would be updated without making changes
    --verbose    Show detailed information about calculations
"""

import json
import sys
import argparse
from datetime import datetime
from typing import Dict, Any, List
from collections import defaultdict

class MetricsUpdater:
    def __init__(self, todo_file: str = 'TODO.json', stories_file: str = 'USER_STORIES.json'):
        """Initialize metrics updater with data files."""
        self.todo_file = todo_file
        self.stories_file = stories_file
        
        try:
            with open(todo_file, 'r') as f:
                self.todo_data = json.load(f)
        except FileNotFoundError:
            print(f"❌ Error: {todo_file} not found", file=sys.stderr)
            sys.exit(1)
        except json.JSONDecodeError as e:
            print(f"❌ Error: Invalid JSON in {todo_file}: {e}", file=sys.stderr)
            sys.exit(1)
            
        try:
            with open(stories_file, 'r') as f:
                self.stories_data = json.load(f)
        except FileNotFoundError:
            print(f"❌ Error: {stories_file} not found", file=sys.stderr)
            sys.exit(1)
        except json.JSONDecodeError as e:
            print(f"❌ Error: Invalid JSON in {stories_file}: {e}", file=sys.stderr)
            sys.exit(1)

    def calculate_todo_metrics(self) -> Dict[str, Any]:
        """Calculate all TODO-related metrics."""
        all_todos = []
        status_counts = defaultdict(int)
        priority_counts = defaultdict(int)
        total_estimated_hours = 0
        total_actual_hours = 0
        overdue_count = 0
        
        # Collect all todos from all phases
        for phase_key, phase in self.todo_data.get('phases', {}).items():
            for todo in phase.get('todos', []):
                all_todos.append(todo)
                
                # Count by status
                status = todo.get('status', 'NOT_STARTED')
                status_counts[status] += 1
                
                # Count by priority
                priority = todo.get('priority', 'MEDIUM')
                priority_counts[priority] += 1
                
                # Sum effort
                estimated = todo.get('estimatedHours', 0)
                actual = todo.get('actualHours', 0)
                total_estimated_hours += estimated
                total_actual_hours += actual
                
                # Check if overdue
                due_date = todo.get('dueDate')
                if due_date and todo.get('status') not in ['COMPLETED', 'CANCELLED']:
                    try:
                        due = datetime.strptime(due_date, '%Y-%m-%d')
                        if due < datetime.now():
                            overdue_count += 1
                    except ValueError:
                        pass  # Invalid date format, skip
        
        # Calculate completion percentage
        total_todos = len(all_todos)
        completed_todos = status_counts.get('COMPLETED', 0)
        completion_percentage = (completed_todos / total_todos * 100) if total_todos > 0 else 0
        
        # Calculate average hours per todo
        avg_hours = total_estimated_hours / total_todos if total_todos > 0 else 0
        
        # Calculate current phase progress
        current_phase = self.todo_data.get('currentPhase', '')
        phase_progress = 0
        if current_phase in self.todo_data.get('phases', {}):
            phase_todos = self.todo_data['phases'][current_phase].get('todos', [])
            phase_completed = sum(1 for todo in phase_todos if todo.get('status') == 'COMPLETED')
            phase_progress = (phase_completed / len(phase_todos) * 100) if phase_todos else 0
        
        return {
            'totalTodos': total_todos,
            'completedTodos': completed_todos,
            'inProgressTodos': status_counts.get('IN_PROGRESS', 0),
            'notStartedTodos': status_counts.get('NOT_STARTED', 0),
            'blockedTodos': status_counts.get('BLOCKED', 0),
            'estimatedTotalHours': round(total_estimated_hours, 1),
            'actualHoursSpent': round(total_actual_hours, 1),
            'criticalItems': priority_counts.get('CRITICAL', 0),
            'highPriorityItems': priority_counts.get('HIGH', 0),
            'mediumPriorityItems': priority_counts.get('MEDIUM', 0),
            'lowPriorityItems': priority_counts.get('LOW', 0),
            'overdueTodos': overdue_count,
            'completionPercentage': round(completion_percentage, 1),
            'averageHoursPerTodo': round(avg_hours, 1),
            'currentPhaseProgress': round(phase_progress, 1)
        }

    def calculate_story_metrics(self) -> Dict[str, Any]:
        """Calculate all user story related metrics."""
        stories = self.stories_data.get('stories', [])
        status_counts = defaultdict(int)
        priority_counts = defaultdict(int)
        epic_counts = defaultdict(int)
        effort_distribution = defaultdict(int)
        total_effort = 0
        
        for story in stories:
            # Count by status
            status = story.get('status', 'NOT_STARTED')
            status_counts[status] += 1
            
            # Count by priority  
            priority = story.get('priority', 'MEDIUM')
            priority_counts[priority] += 1
            
            # Count by epic
            epic = story.get('epic', 'Uncategorized')
            epic_counts[epic] += 1
            
            # Effort analysis
            effort = story.get('effort', 0)
            total_effort += effort
            
            # Effort distribution
            if effort <= 3:
                effort_distribution['1-3'] += 1
            elif effort <= 6:
                effort_distribution['4-6'] += 1
            elif effort <= 10:
                effort_distribution['7-10'] += 1
            else:
                effort_distribution['10+'] += 1
        
        total_stories = len(stories)
        completed_stories = status_counts.get('COMPLETED', 0)
        completion_rate = (completed_stories / total_stories * 100) if total_stories > 0 else 0
        avg_effort = total_effort / total_stories if total_stories > 0 else 0
        
        # Calculate velocity trend from sprint planning
        velocity_trend = []
        sprints = self.stories_data.get('sprintPlanning', [])
        for sprint in sprints:
            if sprint.get('status') == 'COMPLETED' and sprint.get('actualEffort'):
                velocity_trend.append({
                    'sprint': sprint.get('sprintNumber', 0),
                    'pointsCompleted': sprint.get('actualEffort', 0),
                    'storiesCompleted': len(sprint.get('plannedStories', []))
                })
        
        return {
            'totalStories': total_stories,
            'notStarted': status_counts.get('NOT_STARTED', 0),
            'inProgress': status_counts.get('IN_PROGRESS', 0),
            'inReview': status_counts.get('IN_REVIEW', 0),
            'completed': completed_stories,
            'blocked': status_counts.get('BLOCKED', 0),
            'criticalPriority': priority_counts.get('CRITICAL', 0),
            'highPriority': priority_counts.get('HIGH', 0),
            'mediumPriority': priority_counts.get('MEDIUM', 0),
            'lowPriority': priority_counts.get('LOW', 0),
            'totalEffortPoints': round(total_effort, 1),
            'averageEffortPerStory': round(avg_effort, 1),
            'storiesPerEpic': dict(epic_counts),
            'completionRate': round(completion_rate, 1),
            'velocityTrend': velocity_trend,
            'effortDistribution': dict(effort_distribution)
        }

    def update_tags(self) -> Dict[str, List[str]]:
        """Update and organize tags based on current todos."""
        tags = defaultdict(list)
        
        # Collect todo IDs by feature/tag
        for phase_key, phase in self.todo_data.get('phases', {}).items():
            for todo in phase.get('todos', []):
                todo_id = todo.get('id', '')
                feature = todo.get('feature', 'untagged')
                
                if todo_id and feature:
                    tags[feature].append(todo_id)
                
                # Also add any explicit labels
                for label in todo.get('labels', []):
                    tags[label].append(todo_id)
        
        # Ensure common categories exist even if empty
        common_tags = [
            'infrastructure', 'frontend', 'backend', 'api', 'database', 
            'testing', 'documentation', 'security', 'performance', 
            'deployment', 'monitoring', 'integration', 'bug_fix', 
            'enhancement', 'refactor', 'maintenance'
        ]
        
        for tag in common_tags:
            if tag not in tags:
                tags[tag] = []
        
        # Sort todo IDs within each tag
        for tag in tags:
            tags[tag] = sorted(list(set(tags[tag])))
        
        return dict(tags)

    def update_files(self, dry_run: bool = False, verbose: bool = False) -> None:
        """Update metrics in both JSON files."""
        # Calculate new metrics
        todo_metrics = self.calculate_todo_metrics()
        story_metrics = self.calculate_story_metrics()
        updated_tags = self.update_tags()
        
        if verbose:
            print("📊 Calculated Metrics:")
            print(f"   TODO Metrics: {len(todo_metrics)} fields updated")
            print(f"   Story Metrics: {len(story_metrics)} fields updated")
            print(f"   Tags: {len(updated_tags)} categories")
        
        # Update TODO.json
        old_todo_metrics = self.todo_data.get('globalMetrics', {})
        self.todo_data['globalMetrics'] = todo_metrics
        self.todo_data['tags'] = updated_tags
        self.todo_data['lastUpdated'] = datetime.now().strftime('%Y-%m-%d')
        
        # Update USER_STORIES.json
        old_story_metrics = self.stories_data.get('metrics', {})
        self.stories_data['metrics'] = story_metrics
        self.stories_data['lastUpdated'] = datetime.now().strftime('%Y-%m-%d')
        
        # Show changes
        print("📈 Metrics Updates:")
        print(f"   TODO completion: {old_todo_metrics.get('completionPercentage', 0)}% → {todo_metrics['completionPercentage']}%")
        print(f"   Story completion: {old_story_metrics.get('completionRate', 0)}% → {story_metrics['completionRate']}%")
        print(f"   Tasks: {todo_metrics['completedTodos']}/{todo_metrics['totalTodos']} complete")
        print(f"   Stories: {story_metrics['completed']}/{story_metrics['totalStories']} complete")
        
        if todo_metrics['blockedTodos'] > 0:
            print(f"   ⚠️  Blocked tasks: {todo_metrics['blockedTodos']}")
        if todo_metrics['overdueTodos'] > 0:
            print(f"   🚨 Overdue tasks: {todo_metrics['overdueTodos']}")
        
        if not dry_run:
            # Write updated files
            with open(self.todo_file, 'w') as f:
                json.dump(self.todo_data, f, indent=2)
            
            with open(self.stories_file, 'w') as f:
                json.dump(self.stories_data, f, indent=2)
            
            print("✅ Metrics updated successfully!")
        else:
            print("🔍 Dry run complete - no files were modified")

def main():
    parser = argparse.ArgumentParser(description='Update project tracking metrics')
    parser.add_argument('--dry-run', action='store_true', 
                       help='Show what would be updated without making changes')
    parser.add_argument('--verbose', action='store_true',
                       help='Show detailed information about calculations')
    parser.add_argument('--todo-file', default='TODO.json', help='TODO.json file path')
    parser.add_argument('--stories-file', default='USER_STORIES.json', help='USER_STORIES.json file path')
    
    args = parser.parse_args()
    
    updater = MetricsUpdater(args.todo_file, args.stories_file)
    updater.update_files(dry_run=args.dry_run, verbose=args.verbose)

if __name__ == '__main__':
    main()