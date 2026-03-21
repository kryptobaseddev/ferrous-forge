/**
 * Ferrous Forge Diagnostics Provider
 *
 * @task T022
 * @epic T014
 *
 * Real-time validation with inline diagnostics for Rust files.
 * Runs ferrous-forge validate and displays violations as squiggly lines.
 *
 * @module diagnostics
 */

import * as vscode from 'vscode';
import { FerrousForgeConfig, ForgeProjectConfig } from './config';
import { FerrousForgeStatusBar } from './statusbar';

/**
 * Violation data structure from ferrous-forge JSON output
 *
 * @task T022
 */
interface Violation {
    /** Type of violation */
    violation_type: string;
    /** File where violation occurred */
    file: string;
    /** Line number (1-based) */
    line: number;
    /** Human-readable message */
    message: string;
    /** Severity: 'Error' or 'Warning' */
    severity: string;
}

/**
 * Validation result from ferrous-forge
 *
 * @task T022
 */
interface ValidationResult {
    /** Whether validation passed */
    success: boolean;
    /** Array of violations found */
    violations: Violation[];
    /** Total violation count */
    total_violations: number;
    /** Violations grouped by type */
    violations_by_type: Record<string, number>;
}

/**
 * Diagnostics provider for real-time Ferrous Forge validation
 *
 * @task T022
 * @epic T014
 */
export class FerrousForgeDiagnosticsProvider implements vscode.Disposable {
    private diagnosticCollection: vscode.DiagnosticCollection;
    private config: FerrousForgeConfig;
    private outputChannel: vscode.OutputChannel;
    private statusBar: FerrousForgeStatusBar;
    private debounceTimers: Map<string, NodeJS.Timeout> = new Map();
    private isValidating: boolean = false;

    constructor(
        config: FerrousForgeConfig,
        outputChannel: vscode.OutputChannel,
        statusBar: FerrousForgeStatusBar
    ) {
        this.config = config;
        this.outputChannel = outputChannel;
        this.statusBar = statusBar;
        this.diagnosticCollection = vscode.languages.createDiagnosticCollection('ferrousForge');

        this.registerEventHandlers();
    }

    /**
     * Register VS Code event handlers for validation triggers
     *
     * @task T022
     */
    private registerEventHandlers(): void {
        // Validate on document open
        if (this.config.shouldValidateOnOpen()) {
            vscode.workspace.onDidOpenTextDocument(
                (doc) => this.onDocumentOpen(doc),
                null,
                this.getDisposables()
            );
        }

        // Validate on document change (typing)
        if (this.config.shouldValidateOnType()) {
            vscode.workspace.onDidChangeTextDocument(
                (e) => this.onDocumentChange(e),
                null,
                this.getDisposables()
            );
        }

        // Validate on document save
        if (this.config.shouldValidateOnSave()) {
            vscode.workspace.onDidSaveTextDocument(
                (doc) => this.onDocumentSave(doc),
                null,
                this.getDisposables()
            );
        }

        // Clear diagnostics on document close
        vscode.workspace.onDidCloseTextDocument(
            (doc) => this.diagnosticCollection.delete(doc.uri),
            null,
            this.getDisposables()
        );

        // Validate all open Rust documents on startup
        vscode.workspace.textDocuments.forEach((doc) => {
            if (doc.languageId === 'rust' && this.config.shouldValidateOnOpen()) {
                this.validateDocument(doc);
            }
        });
    }

    /**
     * Handle document open event
     *
     * @task T022
     */
    private onDocumentOpen(document: vscode.TextDocument): void {
        if (document.languageId === 'rust') {
            this.validateDocument(document);
        }
    }

    /**
     * Handle document change event with debouncing
     *
     * @task T022
     */
    private onDocumentChange(event: vscode.TextDocumentChangeEvent): void {
        if (event.document.languageId !== 'rust') {
            return;
        }

        const document = event.document;
        const uri = document.uri.toString();

        // Clear existing timer
        const existingTimer = this.debounceTimers.get(uri);
        if (existingTimer) {
            clearTimeout(existingTimer);
        }

        // Set new debounced validation
        const delay = this.config.getDebounceDelay();
        const timer = setTimeout(() => {
            this.validateDocument(document);
            this.debounceTimers.delete(uri);
        }, delay);

        this.debounceTimers.set(uri, timer);
    }

    /**
     * Handle document save event
     *
     * @task T022
     */
    private onDocumentSave(document: vscode.TextDocument): void {
        if (document.languageId === 'rust') {
            // Clear debounce timer since we're validating now
            const uri = document.uri.toString();
            const timer = this.debounceTimers.get(uri);
            if (timer) {
                clearTimeout(timer);
                this.debounceTimers.delete(uri);
            }
            this.validateDocument(document);
        }
    }

    /**
     * Validate a single document
     *
     * @param document - The document to validate
     *
     * @task T022
     * @epic T014
     */
    async validateDocument(document: vscode.TextDocument): Promise<void> {
        if (!this.config.isEnabled() || this.isValidating) {
            return;
        }

        const workspaceRoot = vscode.workspace.getWorkspaceFolder(document.uri)?.uri.fsPath;
        if (!workspaceRoot) {
            return;
        }

        this.isValidating = true;
        this.statusBar.setValidating();

        try {
            const result = await this.runValidation(workspaceRoot, document.fileName);

            if (result) {
                this.updateDiagnostics(document.uri, result);
                this.updateStatusBar(result);
                this.logResults(document.fileName, result);
            }
        } catch (error) {
            const message = error instanceof Error ? error.message : String(error);
            this.outputChannel.appendLine(`Validation error: ${message}`);
            this.statusBar.setError();
        } finally {
            this.isValidating = false;
            if (this.diagnosticCollection.get(document.uri)?.length === 0) {
                this.statusBar.setSuccess();
            }
        }
    }

    /**
     * Validate the entire workspace
     *
     * @task T022
     */
    async validateWorkspace(): Promise<void> {
        if (!this.config.isEnabled()) {
            return;
        }

        const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        if (!workspaceRoot) {
            vscode.window.showWarningMessage('No workspace folder open');
            return;
        }

        this.statusBar.setValidating();
        vscode.window.withProgress(
            {
                location: vscode.ProgressLocation.Window,
                title: 'Running Ferrous Forge validation...'
            },
            async () => {
                try {
                    const result = await this.runValidation(workspaceRoot);

                    if (result) {
                        // Update diagnostics for all affected files
                        const groupedViolations = this.groupViolationsByFile(result.violations, workspaceRoot);

                        for (const [filePath, violations] of groupedViolations) {
                            const uri = vscode.Uri.file(filePath);
                            const diagnostics = this.createDiagnostics(violations);
                            this.diagnosticCollection.set(uri, diagnostics);
                        }

                        this.updateStatusBar(result);
                        this.showValidationSummary(result);
                    }
                } catch (error) {
                    const message = error instanceof Error ? error.message : String(error);
                    vscode.window.showErrorMessage(`Validation failed: ${message}`);
                    this.statusBar.setError();
                }
            }
        );
    }

    /**
     * Run ferrous-forge validation command
     *
     * @param workspaceRoot - The workspace root directory
     * @param filePath - Optional specific file to validate
     * @returns Validation result or undefined
     *
     * @task T022
     */
    private async runValidation(
        workspaceRoot: string,
        filePath?: string
    ): Promise<ValidationResult | undefined> {
        const { exec } = await import('child_process');
        const { promisify } = await import('util');
        const execAsync = promisify(exec);

        const executable = this.config.getExecutablePath();

        // Check if executable exists
        try {
            await execAsync(`which ${executable}`);
        } catch {
            this.outputChannel.appendLine(`Executable not found: ${executable}`);
            this.outputChannel.appendLine('Please install Ferrous Forge: cargo install ferrous-forge');
            return undefined;
        }

        // Build command
        let command = `${executable} validate --format json`;
        if (filePath) {
            // Note: ferrous-forge validates the project, not individual files
            // We filter results by file later
            command = `${executable} validate --format json`;
        }

        this.outputChannel.appendLine(`Running: ${command}`);

        try {
            const { stdout, stderr } = await execAsync(command, {
                cwd: workspaceRoot,
                timeout: 60000,
                encoding: 'utf-8'
            });

            if (stderr) {
                this.outputChannel.appendLine(`stderr: ${stderr}`);
            }

            // Parse JSON output
            const lines = stdout.split('\n').filter((line) => line.trim());
            for (const line of lines) {
                try {
                    const parsed = JSON.parse(line);
                    if (parsed.violations !== undefined) {
                        return parsed as ValidationResult;
                    }
                } catch {
                    // Not JSON, continue
                }
            }

            // If no JSON output, construct result from stdout
            return this.parseTextOutput(stdout, stderr);
        } catch (error) {
            // Validation may return non-zero exit code with violations
            const execError = error as { stdout?: string; stderr?: string };
            if (execError.stdout || execError.stderr) {
                return this.parseTextOutput(execError.stdout || '', execError.stderr || '');
            }
            throw error;
        }
    }

    /**
     * Parse text output when JSON is not available
     *
     * @task T022
     */
    private parseTextOutput(stdout: string, stderr: string): ValidationResult {
        const violations: Violation[] = [];
        const output = stdout + stderr;

        // Parse common violation patterns
        // Format: "  file.rs:42 - Message"
        const violationRegex = /^\s+(.+?):(\d+)\s+-\s+(.+)$/gm;
        let match;

        while ((match = violationRegex.exec(output)) !== null) {
            violations.push({
                violation_type: 'Unknown',
                file: match[1],
                line: parseInt(match[2], 10) - 1, // Convert to 0-based
                message: match[3],
                severity: 'Warning'
            });
        }

        return {
            success: violations.length === 0,
            violations,
            total_violations: violations.length,
            violations_by_type: {}
        };
    }

    /**
     * Group violations by file path
     *
     * @task T022
     */
    private groupViolationsByFile(
        violations: Violation[],
        workspaceRoot: string
    ): Map<string, Violation[]> {
        const grouped = new Map<string, Violation[]>();

        for (const violation of violations) {
            const fullPath = violation.file.startsWith('/')
                ? violation.file
                : `${workspaceRoot}/${violation.file}`;

            const existing = grouped.get(fullPath) || [];
            existing.push(violation);
            grouped.set(fullPath, existing);
        }

        return grouped;
    }

    /**
     * Update diagnostics for a document
     *
     * @task T022
     */
    private updateDiagnostics(uri: vscode.Uri, result: ValidationResult): void {
        const workspaceRoot = vscode.workspace.getWorkspaceFolder(uri)?.uri.fsPath;
        if (!workspaceRoot) {
            return;
        }

        // Filter violations for this file
        const fileViolations = result.violations.filter((v) => {
            const violationPath = v.file.startsWith('/')
                ? v.file
                : `${workspaceRoot}/${v.file}`;
            return violationPath === uri.fsPath;
        });

        const diagnostics = this.createDiagnostics(fileViolations);
        this.diagnosticCollection.set(uri, diagnostics);
    }

    /**
     * Create VS Code diagnostics from violations
     *
     * @task T022
     */
    private createDiagnostics(violations: Violation[]): vscode.Diagnostic[] {
        return violations.map((violation) => {
            const range = new vscode.Range(
                violation.line,
                0,
                violation.line,
                Number.MAX_SAFE_INTEGER
            );

            const severity = violation.severity === 'Error'
                ? vscode.DiagnosticSeverity.Error
                : vscode.DiagnosticSeverity.Warning;

            const diagnostic = new vscode.Diagnostic(
                range,
                `[Ferrous Forge] ${violation.message}`,
                severity
            );

            diagnostic.code = violation.violation_type;
            diagnostic.source = 'ferrous-forge';

            return diagnostic;
        });
    }

    /**
     * Update status bar based on validation result
     *
     * @task T022
     */
    private updateStatusBar(result: ValidationResult): void {
        if (result.success || result.total_violations === 0) {
            this.statusBar.setSuccess();
        } else {
            this.statusBar.setViolations(result.total_violations);
        }
    }

    /**
     * Log validation results to output channel
     *
     * @task T022
     */
    private logResults(fileName: string, result: ValidationResult): void {
        this.outputChannel.appendLine(`\nValidated: ${fileName}`);
        this.outputChannel.appendLine(`Violations: ${result.total_violations}`);

        if (result.violations.length > 0) {
            for (const v of result.violations) {
                this.outputChannel.appendLine(`  ${v.file}:${v.line + 1} - ${v.message}`);
            }
        }
    }

    /**
     * Show validation summary notification
     *
     * @task T022
     */
    private showValidationSummary(result: ValidationResult): void {
        if (result.success || result.total_violations === 0) {
            vscode.window.showInformationMessage('✅ Ferrous Forge: All checks passed!');
        } else {
            const msg = `❌ Ferrous Forge: ${result.total_violations} violation(s) found`;
            vscode.window.showWarningMessage(msg, 'View Details').then((selection) => {
                if (selection === 'View Details') {
                    this.outputChannel.show();
                }
            });
        }
    }

    /**
     * Handle configuration changes
     *
     * @task T022
     */
    onConfigChanged(): void {
        this.config.reload();

        // Clear all diagnostics if disabled
        if (!this.config.isEnabled()) {
            this.diagnosticCollection.clear();
            this.statusBar.setIdle();
        } else {
            // Re-validate all open documents
            vscode.workspace.textDocuments.forEach((doc) => {
                if (doc.languageId === 'rust') {
                    this.validateDocument(doc);
                }
            });
        }
    }

    /**
     * Get diagnostics collection for external access
     */
    getDiagnosticCollection(): vscode.DiagnosticCollection {
        return this.diagnosticCollection;
    }

    /**
     * Get disposable resources
     */
    private disposables: vscode.Disposable[] = [];
    private getDisposables(): vscode.Disposable[] {
        return this.disposables;
    }

    /**
     * Dispose of resources
     */
    dispose(): void {
        // Clear all debounce timers
        for (const timer of this.debounceTimers.values()) {
            clearTimeout(timer);
        }
        this.debounceTimers.clear();

        this.diagnosticCollection.dispose();
        this.disposables.forEach((d) => d.dispose());
    }
}
