#!/usr/bin/env python3
"""
generate-report.py - Generate status reports from project tracking data

Usage:
    python3 generate-report.py [--format=text|markdown|html|json] [--output=filename]
    
Examples:
    python3 generate-report.py                           # Text output to console
    python3 generate-report.py --format=markdown         # Markdown to console
    python3 generate-report.py --output=report.md        # Save to file
    python3 generate-report.py --format=html --output=dashboard.html
"""

import json
import sys
import argparse
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional

class ProjectReportGenerator:
    def __init__(self, todo_file: str = 'TODO.json', stories_file: str = 'USER_STORIES.json'):
        """Initialize report generator with data files."""
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

    def generate_summary_stats(self) -> Dict[str, Any]:
        """Generate high-level project statistics."""
        todo_metrics = self.todo_data.get('globalMetrics', {})
        story_metrics = self.stories_data.get('metrics', {})
        
        # Calculate phase progress
        current_phase = self.todo_data.get('currentPhase', 'Unknown')
        phase_data = self.todo_data.get('phases', {}).get(current_phase, {})
        phase_todos = phase_data.get('todos', [])
        phase_completed = sum(1 for todo in phase_todos if todo.get('status') == 'COMPLETED')
        phase_progress = (phase_completed / len(phase_todos) * 100) if phase_todos else 0
        
        # Calculate velocity (if sprint data available)
        sprints = self.stories_data.get('sprintPlanning', [])
        recent_velocity = 0
        if sprints:
            completed_sprints = [s for s in sprints if s.get('status') == 'COMPLETED' and s.get('actualEffort')]
            if completed_sprints:
                recent_velocity = sum(s.get('actualEffort', 0) for s in completed_sprints[-3:]) / min(3, len(completed_sprints))
        
        return {
            'project_name': self.todo_data.get('projectName', 'Unknown'),
            'project_version': self.todo_data.get('projectVersion', 'Unknown'),
            'last_updated': self.todo_data.get('lastUpdated', 'Unknown'),
            'current_phase': current_phase,
            'phase_progress': round(phase_progress, 1),
            'total_todos': todo_metrics.get('totalTodos', 0),
            'completed_todos': todo_metrics.get('completedTodos', 0),
            'in_progress_todos': todo_metrics.get('inProgressTodos', 0),
            'blocked_todos': todo_metrics.get('blockedTodos', 0),
            'total_stories': story_metrics.get('totalStories', 0),
            'completed_stories': story_metrics.get('completed', 0),
            'story_completion_rate': story_metrics.get('completionRate', 0),
            'total_effort_points': story_metrics.get('totalEffortPoints', 0),
            'recent_velocity': round(recent_velocity, 1),
            'critical_items': todo_metrics.get('criticalItems', 0),
            'high_priority_items': todo_metrics.get('highPriorityItems', 0)
        }

    def get_risks_and_blockers(self) -> Dict[str, List[str]]:
        """Extract current risks and blockers."""
        risks = []
        blockers = []
        
        # Get risks from risk register
        for risk in self.todo_data.get('riskRegister', []):
            if risk.get('status') not in ['closed', 'mitigated']:
                severity = risk.get('severity', 'unknown')
                description = risk.get('description', 'No description')
                risks.append(f"[{severity.upper()}] {description}")
        
        # Get blocked tasks
        for phase_key, phase in self.todo_data.get('phases', {}).items():
            for todo in phase.get('todos', []):
                if todo.get('status') == 'BLOCKED':
                    title = todo.get('title', 'Untitled task')
                    blockers.append(f"{todo.get('id', 'Unknown')}: {title}")
        
        return {'risks': risks, 'blockers': blockers}

    def get_upcoming_milestones(self, days_ahead: int = 30) -> List[Dict[str, Any]]:
        """Get milestones due in the next N days."""
        milestones = []
        cutoff_date = datetime.now() + timedelta(days=days_ahead)
        
        for milestone in self.todo_data.get('milestones', []):
            if milestone.get('targetDate'):
                try:
                    target_date = datetime.strptime(milestone['targetDate'], '%Y-%m-%d')
                    if target_date <= cutoff_date and milestone.get('status') != 'COMPLETED':
                        days_until = (target_date - datetime.now()).days
                        milestones.append({
                            'name': milestone.get('name', 'Unnamed milestone'),
                            'target_date': milestone['targetDate'],
                            'days_until': days_until,
                            'status': milestone.get('status', 'Unknown')
                        })
                except ValueError:
                    continue  # Skip malformed dates
        
        return sorted(milestones, key=lambda x: x['days_until'])

    def generate_text_report(self) -> str:
        """Generate a text-based status report."""
        stats = self.generate_summary_stats()
        risks_blockers = self.get_risks_and_blockers()
        milestones = self.get_upcoming_milestones()
        
        report = []
        report.append("=" * 60)
        report.append(f"PROJECT STATUS REPORT - {stats['project_name']}")
        report.append("=" * 60)
        report.append(f"Version: {stats['project_version']}")
        report.append(f"Last Updated: {stats['last_updated']}")
        report.append(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        report.append("")
        
        # Current Phase Progress
        report.append("CURRENT PHASE PROGRESS")
        report.append("-" * 25)
        report.append(f"Phase: {stats['current_phase']}")
        report.append(f"Progress: {stats['phase_progress']}% complete")
        report.append("")
        
        # Task Summary
        report.append("TASK SUMMARY")
        report.append("-" * 13)
        report.append(f"Total Tasks: {stats['total_todos']}")
        report.append(f"Completed:   {stats['completed_todos']} ({round(stats['completed_todos']/max(stats['total_todos'],1)*100, 1)}%)")
        report.append(f"In Progress: {stats['in_progress_todos']}")
        report.append(f"Blocked:     {stats['blocked_todos']}")
        report.append("")
        
        # User Stories Summary
        report.append("USER STORIES SUMMARY")
        report.append("-" * 20)
        report.append(f"Total Stories: {stats['total_stories']}")
        report.append(f"Completed:     {stats['completed_stories']} ({stats['story_completion_rate']}%)")
        report.append(f"Effort Points: {stats['total_effort_points']}")
        if stats['recent_velocity'] > 0:
            report.append(f"Recent Velocity: {stats['recent_velocity']} points/sprint")
        report.append("")
        
        # Priority Items
        if stats['critical_items'] > 0 or stats['high_priority_items'] > 0:
            report.append("HIGH PRIORITY ITEMS")
            report.append("-" * 19)
            report.append(f"Critical: {stats['critical_items']}")
            report.append(f"High:     {stats['high_priority_items']}")
            report.append("")
        
        # Risks and Blockers
        if risks_blockers['risks'] or risks_blockers['blockers']:
            report.append("RISKS AND BLOCKERS")
            report.append("-" * 18)
            
            if risks_blockers['risks']:
                report.append("Active Risks:")
                for risk in risks_blockers['risks'][:5]:  # Show top 5
                    report.append(f"  • {risk}")
            
            if risks_blockers['blockers']:
                if risks_blockers['risks']:
                    report.append("")
                report.append("Blocked Tasks:")
                for blocker in risks_blockers['blockers']:
                    report.append(f"  • {blocker}")
            report.append("")
        
        # Upcoming Milestones
        if milestones:
            report.append("UPCOMING MILESTONES (Next 30 Days)")
            report.append("-" * 35)
            for milestone in milestones[:3]:  # Show next 3
                days_text = f"in {milestone['days_until']} days" if milestone['days_until'] > 0 else "TODAY" if milestone['days_until'] == 0 else f"{abs(milestone['days_until'])} days overdue"
                report.append(f"  • {milestone['name']} - {milestone['target_date']} ({days_text})")
            report.append("")
        
        # Team Summary
        team_members = self.todo_data.get('teamMembers', [])
        if team_members:
            report.append("TEAM SUMMARY")
            report.append("-" * 12)
            report.append(f"Team Size: {len(team_members)}")
            active_members = sum(1 for member in team_members if member.get('assignedTodos'))
            report.append(f"Active Members: {active_members}")
            report.append("")
        
        return "\n".join(report)

    def generate_markdown_report(self) -> str:
        """Generate a Markdown-formatted status report."""
        stats = self.generate_summary_stats()
        risks_blockers = self.get_risks_and_blockers()
        milestones = self.get_upcoming_milestones()
        
        report = []
        report.append(f"# Project Status Report - {stats['project_name']}")
        report.append("")
        report.append(f"**Version:** {stats['project_version']}  ")
        report.append(f"**Last Updated:** {stats['last_updated']}  ")
        report.append(f"**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        report.append("")
        
        # Current Phase Progress
        report.append("## Current Phase Progress")
        report.append("")
        report.append(f"- **Phase:** {stats['current_phase']}")
        report.append(f"- **Progress:** {stats['phase_progress']}% complete")
        report.append("")
        
        # Task Summary with progress bars
        completion_pct = round(stats['completed_todos']/max(stats['total_todos'],1)*100, 1)
        progress_bar = "█" * int(completion_pct / 5) + "░" * (20 - int(completion_pct / 5))
        
        report.append("## Task Summary")
        report.append("")
        report.append(f"```")
        report.append(f"Progress: [{progress_bar}] {completion_pct}%")
        report.append(f"```")
        report.append("")
        report.append(f"| Status | Count | Percentage |")
        report.append(f"|--------|-------|------------|")
        report.append(f"| Total | {stats['total_todos']} | 100% |")
        report.append(f"| Completed | {stats['completed_todos']} | {completion_pct}% |")
        report.append(f"| In Progress | {stats['in_progress_todos']} | {round(stats['in_progress_todos']/max(stats['total_todos'],1)*100, 1)}% |")
        report.append(f"| Blocked | {stats['blocked_todos']} | {round(stats['blocked_todos']/max(stats['total_todos'],1)*100, 1)}% |")
        report.append("")
        
        # User Stories Summary
        report.append("## User Stories Summary")
        report.append("")
        report.append(f"- **Total Stories:** {stats['total_stories']}")
        report.append(f"- **Completed:** {stats['completed_stories']} ({stats['story_completion_rate']}%)")
        report.append(f"- **Total Effort Points:** {stats['total_effort_points']}")
        if stats['recent_velocity'] > 0:
            report.append(f"- **Recent Velocity:** {stats['recent_velocity']} points/sprint")
        report.append("")
        
        # High Priority Items
        if stats['critical_items'] > 0 or stats['high_priority_items'] > 0:
            report.append("## High Priority Items")
            report.append("")
            if stats['critical_items'] > 0:
                report.append(f"🔴 **Critical Items:** {stats['critical_items']}")
            if stats['high_priority_items'] > 0:
                report.append(f"🟠 **High Priority Items:** {stats['high_priority_items']}")
            report.append("")
        
        # Risks and Blockers
        if risks_blockers['risks'] or risks_blockers['blockers']:
            report.append("## Risks and Blockers")
            report.append("")
            
            if risks_blockers['risks']:
                report.append("### Active Risks")
                for risk in risks_blockers['risks'][:5]:
                    report.append(f"- ⚠️ {risk}")
                report.append("")
            
            if risks_blockers['blockers']:
                report.append("### Blocked Tasks")
                for blocker in risks_blockers['blockers']:
                    report.append(f"- 🚫 {blocker}")
                report.append("")
        
        # Upcoming Milestones
        if milestones:
            report.append("## Upcoming Milestones")
            report.append("")
            for milestone in milestones[:3]:
                icon = "🎯" if milestone['days_until'] > 7 else "⏰" if milestone['days_until'] > 0 else "🚨"
                days_text = f"in {milestone['days_until']} days" if milestone['days_until'] > 0 else "TODAY" if milestone['days_until'] == 0 else f"{abs(milestone['days_until'])} days overdue"
                report.append(f"- {icon} **{milestone['name']}** - {milestone['target_date']} ({days_text})")
            report.append("")
        
        return "\n".join(report)

    def generate_html_report(self) -> str:
        """Generate an HTML dashboard report."""
        stats = self.generate_summary_stats()
        risks_blockers = self.get_risks_and_blockers()
        milestones = self.get_upcoming_milestones()
        
        completion_pct = round(stats['completed_todos']/max(stats['total_todos'],1)*100, 1)
        
        html = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Project Dashboard - {stats['project_name']}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        .header {{ background: white; border-radius: 8px; padding: 20px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; margin-bottom: 20px; }}
        .stat-card {{ background: white; border-radius: 8px; padding: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .stat-value {{ font-size: 2em; font-weight: bold; color: #2563eb; }}
        .stat-label {{ font-size: 0.9em; color: #6b7280; margin-top: 5px; }}
        .progress-bar {{ width: 100%; height: 20px; background: #e5e7eb; border-radius: 10px; overflow: hidden; }}
        .progress-fill {{ height: 100%; background: linear-gradient(90deg, #10b981, #3b82f6); transition: width 0.3s ease; }}
        .section {{ background: white; border-radius: 8px; padding: 20px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .section h2 {{ margin-top: 0; color: #1f2937; border-bottom: 2px solid #e5e7eb; padding-bottom: 10px; }}
        .risk-item {{ padding: 10px; background: #fef2f2; border-left: 4px solid #ef4444; margin-bottom: 10px; border-radius: 4px; }}
        .blocker-item {{ padding: 10px; background: #fffbeb; border-left: 4px solid #f59e0b; margin-bottom: 10px; border-radius: 4px; }}
        .milestone-item {{ padding: 10px; background: #f0f9ff; border-left: 4px solid #3b82f6; margin-bottom: 10px; border-radius: 4px; }}
        .timestamp {{ color: #6b7280; font-size: 0.9em; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{stats['project_name']} - Project Dashboard</h1>
            <p><strong>Version:</strong> {stats['project_version']} | <strong>Phase:</strong> {stats['current_phase']}</p>
            <p class="timestamp">Last updated: {stats['last_updated']} | Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</p>
        </div>
        
        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-value">{completion_pct}%</div>
                <div class="stat-label">Tasks Complete</div>
                <div class="progress-bar" style="margin-top: 10px;">
                    <div class="progress-fill" style="width: {completion_pct}%;"></div>
                </div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{stats['completed_todos']}/{stats['total_todos']}</div>
                <div class="stat-label">Tasks Done</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{stats['completed_stories']}/{stats['total_stories']}</div>
                <div class="stat-label">Stories Complete</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{stats['phase_progress']}%</div>
                <div class="stat-label">Phase Progress</div>
            </div>
        </div>"""
        
        # Risks and Blockers section
        if risks_blockers['risks'] or risks_blockers['blockers']:
            html += """
        <div class="section">
            <h2>⚠️ Risks and Blockers</h2>"""
            
            for risk in risks_blockers['risks'][:3]:
                html += f'<div class="risk-item">🔥 {risk}</div>'
            
            for blocker in risks_blockers['blockers']:
                html += f'<div class="blocker-item">🚫 {blocker}</div>'
            
            html += "</div>"
        
        # Milestones section
        if milestones:
            html += """
        <div class="section">
            <h2>🎯 Upcoming Milestones</h2>"""
            
            for milestone in milestones[:3]:
                days_text = f"in {milestone['days_until']} days" if milestone['days_until'] > 0 else "TODAY" if milestone['days_until'] == 0 else f"{abs(milestone['days_until'])} days overdue"
                html += f"""<div class="milestone-item">
                    <strong>{milestone['name']}</strong><br>
                    📅 {milestone['target_date']} ({days_text})
                </div>"""
            
            html += "</div>"
        
        html += """
    </div>
</body>
</html>"""
        
        return html

    def generate_json_report(self) -> str:
        """Generate a JSON report for API consumption."""
        stats = self.generate_summary_stats()
        risks_blockers = self.get_risks_and_blockers()
        milestones = self.get_upcoming_milestones()
        
        report_data = {
            'generated_at': datetime.now().isoformat(),
            'project_info': {
                'name': stats['project_name'],
                'version': stats['project_version'],
                'last_updated': stats['last_updated']
            },
            'summary': stats,
            'risks': risks_blockers['risks'],
            'blockers': risks_blockers['blockers'],
            'upcoming_milestones': milestones,
            'team_size': len(self.todo_data.get('teamMembers', [])),
            'active_epics': list(self.stories_data.get('epics', {}).keys())
        }
        
        return json.dumps(report_data, indent=2)

def main():
    parser = argparse.ArgumentParser(description='Generate project status reports')
    parser.add_argument('--format', choices=['text', 'markdown', 'html', 'json'], 
                       default='text', help='Output format (default: text)')
    parser.add_argument('--output', help='Output file (default: stdout)')
    parser.add_argument('--todo-file', default='TODO.json', help='TODO.json file path')
    parser.add_argument('--stories-file', default='USER_STORIES.json', help='USER_STORIES.json file path')
    
    args = parser.parse_args()
    
    # Generate report
    generator = ProjectReportGenerator(args.todo_file, args.stories_file)
    
    if args.format == 'text':
        report = generator.generate_text_report()
    elif args.format == 'markdown':
        report = generator.generate_markdown_report()
    elif args.format == 'html':
        report = generator.generate_html_report()
    elif args.format == 'json':
        report = generator.generate_json_report()
    
    # Output report
    if args.output:
        with open(args.output, 'w') as f:
            f.write(report)
        print(f"✅ Report generated: {args.output}")
    else:
        print(report)

if __name__ == '__main__':
    main()