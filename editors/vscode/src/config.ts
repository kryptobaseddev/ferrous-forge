/**
 * Ferrous Forge Configuration Management
 *
 * @task T022
 * @epic T014
 *
 * Handles VS Code settings and integration with `.forge/config.toml`
 *
 * @module config
 */

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';

/**
 * Ferrous Forge project configuration from `.forge/config.toml`
 *
 * @task T022
 */
export interface ForgeProjectConfig {
    /** Required Rust edition */
    edition?: string;
    /** Required Rust version */
    rustVersion?: string;
    /** Maximum file lines */
    maxFileLines?: number;
    /** Maximum function lines */
    maxFunctionLines?: number;
    /** Maximum line length */
    maxLineLength?: number;
}

/**
 * Configuration manager for Ferrous Forge extension
 *
 * @task T022
 * @epic T014
 */
export class FerrousForgeConfig {
    private config: vscode.WorkspaceConfiguration;
    private projectConfig: ForgeProjectConfig = {};
    private configWatcher: vscode.FileSystemWatcher | undefined;

    constructor() {
        this.config = vscode.workspace.getConfiguration('ferrousForge');
        this.loadProjectConfig();
        this.watchProjectConfig();
    }

    /**
     * Reload configuration from VS Code settings
     *
     * @task T022
     */
    reload(): void {
        this.config = vscode.workspace.getConfiguration('ferrousForge');
        this.loadProjectConfig();
    }

    /**
     * Check if Ferrous Forge validation is enabled
     */
    isEnabled(): boolean {
        return this.config.get('enable', true);
    }

    /**
     * Get the path to the ferrous-forge executable
     */
    getExecutablePath(): string {
        return this.config.get('executablePath', 'ferrous-forge');
    }

    /**
     * Check if validation should run on type (debounced)
     */
    shouldValidateOnType(): boolean {
        return this.config.get('validateOnType', true);
    }

    /**
     * Check if validation should run on save
     */
    shouldValidateOnSave(): boolean {
        return this.config.get('validateOnSave', true);
    }

    /**
     * Check if validation should run on file open
     */
    shouldValidateOnOpen(): boolean {
        return this.config.get('validateOnOpen', true);
    }

    /**
     * Get the debounce delay in milliseconds
     */
    getDebounceDelay(): number {
        return this.config.get('debounceDelay', 500);
    }

    /**
     * Check if status bar indicator should be shown
     */
    shouldShowStatusBar(): boolean {
        return this.config.get('showStatusBar', true);
    }

    /**
     * Get the default diagnostic severity level
     */
    getDiagnosticSeverity(): vscode.DiagnosticSeverity {
        const severity = this.config.get('diagnosticSeverity', 'error') as string;
        switch (severity) {
            case 'error':
                return vscode.DiagnosticSeverity.Error;
            case 'warning':
                return vscode.DiagnosticSeverity.Warning;
            case 'information':
                return vscode.DiagnosticSeverity.Information;
            case 'hint':
                return vscode.DiagnosticSeverity.Hint;
            default:
                return vscode.DiagnosticSeverity.Error;
        }
    }

    /**
     * Check if quick fixes are enabled
     */
    areQuickFixesEnabled(): boolean {
        return this.config.get('enableQuickFixes', true);
    }

    /**
     * Get the loaded project configuration
     */
    getProjectConfig(): ForgeProjectConfig {
        return { ...this.projectConfig };
    }

    /**
     * Load project configuration from `.forge/config.toml`
     *
     * @task T022
     */
    private loadProjectConfig(): void {
        const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        if (!workspaceRoot) {
            return;
        }

        const configPath = path.join(workspaceRoot, '.forge', 'config.toml');

        if (!fs.existsSync(configPath)) {
            this.projectConfig = {};
            return;
        }

        try {
            const content = fs.readFileSync(configPath, 'utf-8');
            const parsed = this.parseToml(content);
            const limits = parsed?.limits as Record<string, unknown> | undefined;

            this.projectConfig = {
                edition: parsed?.edition as string | undefined,
                rustVersion: parsed?.rust_version as string | undefined,
                maxFileLines: limits?.max_file_lines as number | undefined,
                maxFunctionLines: limits?.max_function_lines as number | undefined,
                maxLineLength: limits?.max_line_length as number | undefined
            };
        } catch (error) {
            console.error('Failed to load .forge/config.toml:', error);
            this.projectConfig = {};
        }
    }

    /**
     * Simple TOML parser for basic Forge config
     *
     * @task T022
     */
    private parseToml(content: string): Record<string, unknown> | undefined {
        const result: Record<string, unknown> = {};
        let currentSection: Record<string, unknown> = result;

        for (const line of content.split('\n')) {
            const trimmed = line.trim();

            // Skip empty lines and comments
            if (!trimmed || trimmed.startsWith('#')) {
                continue;
            }

            // Handle section headers
            if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
                const sectionName = trimmed.slice(1, -1);
                currentSection = {};
                result[sectionName] = currentSection;
                continue;
            }

            // Handle key-value pairs
            const equalIndex = trimmed.indexOf('=');
            if (equalIndex > 0) {
                const key = trimmed.slice(0, equalIndex).trim();
                let value: string | number | boolean = trimmed.slice(equalIndex + 1).trim();

                // Remove quotes from strings
                if ((value.startsWith('"') && value.endsWith('"')) ||
                    (value.startsWith("'") && value.endsWith("'"))) {
                    value = value.slice(1, -1);
                }

                // Try parsing as number
                if (/^\d+$/.test(value)) {
                    value = parseInt(value, 10);
                } else if (/^\d+\.\d+$/.test(value)) {
                    value = parseFloat(value);
                }

                // Try parsing as boolean
                if (value === 'true') {
                    value = true;
                } else if (value === 'false') {
                    value = false;
                }

                currentSection[key] = value;
            }
        }

        return result;
    }

    /**
     * Watch for changes to `.forge/config.toml`
     *
     * @task T022
     */
    private watchProjectConfig(): void {
        const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        if (!workspaceRoot) {
            return;
        }

        const configPattern = new vscode.RelativePattern(workspaceRoot, '.forge/config.toml');
        this.configWatcher = vscode.workspace.createFileSystemWatcher(configPattern);

        this.configWatcher.onDidChange(() => this.loadProjectConfig());
        this.configWatcher.onDidCreate(() => this.loadProjectConfig());
    }

    /**
     * Dispose of configuration watchers
     */
    dispose(): void {
        this.configWatcher?.dispose();
    }
}
