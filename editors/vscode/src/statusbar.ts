/**
 * Ferrous Forge Status Bar Indicator
 *
 * @task T022
 * @epic T014
 *
 * Status bar indicator showing validation status and violation count.
 *
 * @module statusbar
 */

import * as vscode from 'vscode';
import { FerrousForgeConfig } from './config';

/**
 * Status bar manager for Ferrous Forge extension
 *
 * @task T022
 * @epic T014
 */
export class FerrousForgeStatusBar implements vscode.Disposable {
    private statusBarItem: vscode.StatusBarItem;
    private config: FerrousForgeConfig;
    private currentStatus: 'idle' | 'validating' | 'success' | 'error' | 'violations' = 'idle';
    private violationCount: number = 0;

    constructor(config: FerrousForgeConfig) {
        this.config = config;
        this.statusBarItem = vscode.window.createStatusBarItem(
            vscode.StatusBarAlignment.Right,
            100
        );
        this.statusBarItem.command = 'ferrousForge.showOutput';
        this.updateVisibility();
        this.setIdle();
    }

    /**
     * Set status to idle
     *
     * @task T022
     */
    setIdle(): void {
        this.currentStatus = 'idle';
        this.statusBarItem.text = '$(shield) Forge';
        this.statusBarItem.tooltip = 'Ferrous Forge - Ready\nClick to show output';
        this.statusBarItem.backgroundColor = undefined;
        this.updateVisibility();
    }

    /**
     * Set status to validating
     *
     * @task T022
     */
    setValidating(): void {
        this.currentStatus = 'validating';
        this.statusBarItem.text = '$(sync~spin) Forge';
        this.statusBarItem.tooltip = 'Ferrous Forge - Validating...\nClick to show output';
        this.statusBarItem.backgroundColor = undefined;
        this.updateVisibility();
    }

    /**
     * Set status to success (no violations)
     *
     * @task T022
     */
    setSuccess(): void {
        this.currentStatus = 'success';
        this.statusBarItem.text = '$(shield) Forge';
        this.statusBarItem.tooltip = 'Ferrous Forge - All checks passed!\nClick to show output';
        this.statusBarItem.backgroundColor = undefined;
        this.updateVisibility();
    }

    /**
     * Set status to violations found
     *
     * @param count - Number of violations
     *
     * @task T022
     */
    setViolations(count: number): void {
        this.currentStatus = 'violations';
        this.violationCount = count;
        this.statusBarItem.text = `$(shield) Forge: ${count}`;
        this.statusBarItem.tooltip = `Ferrous Forge - ${count} violation(s) found\nClick to show output`;
        this.statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
        this.updateVisibility();
    }

    /**
     * Set status to error
     *
     * @task T022
     */
    setError(): void {
        this.currentStatus = 'error';
        this.statusBarItem.text = '$(shield) Forge: Error';
        this.statusBarItem.tooltip = 'Ferrous Forge - Validation error\nClick to show output';
        this.statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
        this.updateVisibility();
    }

    /**
     * Update visibility based on configuration
     *
     * @task T022
     */
    private updateVisibility(): void {
        if (this.config.shouldShowStatusBar() && this.config.isEnabled()) {
            this.statusBarItem.show();
        } else {
            this.statusBarItem.hide();
        }
    }

    /**
     * Handle configuration changes
     *
     * @task T022
     */
    onConfigChanged(): void {
        this.config.reload();
        this.updateVisibility();
    }

    /**
     * Get current status
     */
    getStatus(): string {
        return this.currentStatus;
    }

    /**
     * Get violation count
     */
    getViolationCount(): number {
        return this.violationCount;
    }

    /**
     * Dispose of status bar item
     */
    dispose(): void {
        this.statusBarItem.dispose();
    }
}
