/**
 * Ferrous Forge VS Code Extension
 *
 * @task T022
 * @epic T014
 *
 * Real-time validation with inline diagnostics and quick fixes.
 * Prevents issues before commit by validating Rust files as you type.
 *
 * @module extension
 */

import * as vscode from 'vscode';
import { FerrousForgeDiagnosticsProvider } from './diagnostics';
import { FerrousForgeQuickFixProvider } from './quickfixes';
import { FerrousForgeStatusBar } from './statusbar';
import { FerrousForgeConfig } from './config';

let diagnosticProvider: FerrousForgeDiagnosticsProvider | undefined;
let quickFixProvider: FerrousForgeQuickFixProvider | undefined;
let statusBar: FerrousForgeStatusBar | undefined;
let outputChannel: vscode.OutputChannel | undefined;

/**
 * Extension activation entry point
 *
 * @param context - The extension context for registering disposables
 *
 * @task T022
 * @epic T014
 */
export function activate(context: vscode.ExtensionContext): void {
    outputChannel = vscode.window.createOutputChannel('Ferrous Forge');
    outputChannel.appendLine('Ferrous Forge extension activated');

    const config = new FerrousForgeConfig();

    // Initialize status bar
    statusBar = new FerrousForgeStatusBar(config);
    context.subscriptions.push(statusBar);

    // Initialize diagnostics provider
    diagnosticProvider = new FerrousForgeDiagnosticsProvider(config, outputChannel, statusBar);
    context.subscriptions.push(diagnosticProvider);

    // Initialize quick fixes provider
    quickFixProvider = new FerrousForgeQuickFixProvider(diagnosticProvider);
    context.subscriptions.push(
        vscode.languages.registerCodeActionsProvider(
            { language: 'rust', scheme: 'file' },
            quickFixProvider,
            {
                providedCodeActionKinds: FerrousForgeQuickFixProvider.providedCodeActionKinds
            }
        )
    );

    // Register commands
    registerCommands(context, config);

    // Set context for menu visibility
    vscode.commands.executeCommand('setContext', 'ferrousForge.enabled', config.isEnabled());

    // Watch configuration changes
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration((e) => {
            if (e.affectsConfiguration('ferrousForge')) {
                config.reload();
                vscode.commands.executeCommand('setContext', 'ferrousForge.enabled', config.isEnabled());
                diagnosticProvider?.onConfigChanged();
                statusBar?.onConfigChanged();
            }
        })
    );

    outputChannel.appendLine('Ferrous Forge extension ready');
}

/**
 * Register all extension commands
 *
 * @param context - The extension context
 * @param config - The Ferrous Forge configuration
 *
 * @task T022
 */
function registerCommands(context: vscode.ExtensionContext, config: FerrousForgeConfig): void {
    // Validate project command
    const validateProjectCmd = vscode.commands.registerCommand(
        'ferrousForge.validate',
        async () => {
            if (!config.isEnabled()) {
                vscode.window.showWarningMessage('Ferrous Forge is disabled');
                return;
            }
            await diagnosticProvider?.validateWorkspace();
        }
    );
    context.subscriptions.push(validateProjectCmd);

    // Validate current file command
    const validateFileCmd = vscode.commands.registerCommand(
        'ferrousForge.validateFile',
        async () => {
            if (!config.isEnabled()) {
                vscode.window.showWarningMessage('Ferrous Forge is disabled');
                return;
            }
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'rust') {
                await diagnosticProvider?.validateDocument(editor.document);
            } else {
                vscode.window.showWarningMessage('No Rust file is currently open');
            }
        }
    );
    context.subscriptions.push(validateFileCmd);

    // Fix all violations command
    const fixCmd = vscode.commands.registerCommand(
        'ferrousForge.fix',
        async () => {
            if (!config.isEnabled()) {
                vscode.window.showWarningMessage('Ferrous Forge is disabled');
                return;
            }
            await runFixCommand(config);
        }
    );
    context.subscriptions.push(fixCmd);

    // Show output channel command
    const showOutputCmd = vscode.commands.registerCommand(
        'ferrousForge.showOutput',
        () => {
            outputChannel?.show();
        }
    );
    context.subscriptions.push(showOutputCmd);

    // Reload configuration command
    const reloadConfigCmd = vscode.commands.registerCommand(
        'ferrousForge.reloadConfig',
        async () => {
            config.reload();
            await diagnosticProvider?.onConfigChanged();
            statusBar?.onConfigChanged();
            vscode.window.showInformationMessage('Ferrous Forge configuration reloaded');
        }
    );
    context.subscriptions.push(reloadConfigCmd);
}

/**
 * Run the ferrous-forge fix command
 *
 * @param config - The Ferrous Forge configuration
 *
 * @task T022
 */
async function runFixCommand(config: FerrousForgeConfig): Promise<void> {
    const { exec } = await import('child_process');
    const { promisify } = await import('util');
    const execAsync = promisify(exec);

    const executable = config.getExecutablePath();
    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;

    if (!workspaceRoot) {
        vscode.window.showErrorMessage('No workspace folder open');
        return;
    }

    outputChannel?.appendLine(`Running: ${executable} fix`);
    statusBar?.setValidating();

    try {
        const { stdout, stderr } = await execAsync(`${executable} fix`, {
            cwd: workspaceRoot,
            timeout: 120000
        });

        outputChannel?.appendLine(stdout);
        if (stderr) {
            outputChannel?.appendLine(stderr);
        }

        vscode.window.showInformationMessage('Ferrous Forge fix completed');

        // Re-validate after fix
        await diagnosticProvider?.validateWorkspace();
    } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        outputChannel?.appendLine(`Fix failed: ${message}`);
        vscode.window.showErrorMessage(`Ferrous Forge fix failed: ${message}`);
    } finally {
        statusBar?.setIdle();
    }
}

/**
 * Extension deactivation cleanup
 *
 * @task T022
 * @epic T014
 */
export function deactivate(): void {
    outputChannel?.appendLine('Ferrous Forge extension deactivated');
    outputChannel?.dispose();
}
