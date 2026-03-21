/**
 * Ferrous Forge Quick Fixes Provider
 *
 * @task T022
 * @epic T014
 *
 * Code Actions (lightbulb) for common Ferrous Forge violations.
 * Provides quick fixes for edition mismatches, formatting, and more.
 *
 * @module quickfixes
 */

import * as vscode from 'vscode';
import { FerrousForgeDiagnosticsProvider } from './diagnostics';

/**
 * Quick fix provider for Ferrous Forge diagnostics
 *
 * @task T022
 * @epic T014
 */
export class FerrousForgeQuickFixProvider implements vscode.CodeActionProvider {
    /**
     * Code action kinds provided by this provider
     */
    static readonly providedCodeActionKinds: vscode.CodeActionKind[] = [
        vscode.CodeActionKind.QuickFix,
        vscode.CodeActionKind.SourceFixAll
    ];

    private diagnosticProvider: FerrousForgeDiagnosticsProvider;

    constructor(diagnosticProvider: FerrousForgeDiagnosticsProvider) {
        this.diagnosticProvider = diagnosticProvider;
    }

    /**
     * Provide code actions for the given document and range
     *
     * @param document - The document to provide actions for
     * @param range - The range to provide actions for
     * @param context - The code action context
     * @param token - A cancellation token
     * @returns Array of code actions
     *
     * @task T022
     */
    provideCodeActions(
        document: vscode.TextDocument,
        range: vscode.Range | vscode.Selection,
        context: vscode.CodeActionContext,
        token: vscode.CancellationToken
    ): vscode.CodeAction[] | undefined {
        // Filter for Ferrous Forge diagnostics
        const forgeDiagnostics = context.diagnostics.filter(
            (d) => d.source === 'ferrous-forge'
        );

        if (forgeDiagnostics.length === 0) {
            return undefined;
        }

        const actions: vscode.CodeAction[] = [];

        for (const diagnostic of forgeDiagnostics) {
            // Check if diagnostic is in the requested range
            if (!range.intersection(diagnostic.range)) {
                continue;
            }

            const violationCode = diagnostic.code as string;

            // Add specific quick fixes based on violation type
            switch (violationCode) {
                case 'WrongEdition':
                    actions.push(...this.createEditionFix(document, diagnostic));
                    break;
                case 'OldRustVersion':
                    actions.push(...this.createRustVersionFix(document, diagnostic));
                    break;
                case 'UnwrapInProduction':
                    actions.push(...this.createUnwrapFix(document, diagnostic));
                    break;
                case 'LineTooLong':
                    actions.push(...this.createLineLengthFix(document, diagnostic));
                    break;
                case 'UnderscoreBandaid':
                    actions.push(...this.createUnderscoreFix(document, diagnostic));
                    break;
                case 'MissingDocs':
                    actions.push(...this.createMissingDocsFix(document, diagnostic));
                    break;
                case 'FileTooLarge':
                case 'FunctionTooLarge':
                    actions.push(this.createRefactorSuggestion(diagnostic));
                    break;
            }
        }

        // Add "Fix All" action if multiple violations
        if (forgeDiagnostics.length > 1) {
            actions.push(this.createFixAllAction(document));
        }

        // Add "Ignore This Violation" action
        actions.push(this.createIgnoreAction(document, range));

        return actions.length > 0 ? actions : undefined;
    }

    /**
     * Create quick fix for WrongEdition violation
     *
     * @task T022
     */
    private createEditionFix(
        document: vscode.TextDocument,
        diagnostic: vscode.Diagnostic
    ): vscode.CodeAction[] {
        const actions: vscode.CodeAction[] = [];

        // Fix in Cargo.toml
        const fixCargoToml = new vscode.CodeAction(
            'Update edition in Cargo.toml to 2024',
            vscode.CodeActionKind.QuickFix
        );
        fixCargoToml.diagnostics = [diagnostic];
        fixCargoToml.command = {
            command: 'ferrousForge.fix',
            title: 'Fix Edition',
            arguments: ['edition']
        };
        actions.push(fixCargoToml);

        // Run cargo fix --edition
        const runCargoFix = new vscode.CodeAction(
            'Run cargo fix --edition',
            vscode.CodeActionKind.QuickFix
        );
        runCargoFix.diagnostics = [diagnostic];
        runCargoFix.command = {
            command: 'workbench.action.terminal.sendSequence',
            title: 'Run cargo fix',
            arguments: [{ text: 'cargo fix --edition\n' }]
        };
        actions.push(runCargoFix);

        return actions;
    }

    /**
     * Create quick fix for OldRustVersion violation
     *
     * @task T022
     */
    private createRustVersionFix(
        document: vscode.TextDocument,
        diagnostic: vscode.Diagnostic
    ): vscode.CodeAction[] {
        const actions: vscode.CodeAction[] = [];

        const updateRustVersion = new vscode.CodeAction(
            'Update rust-version in Cargo.toml',
            vscode.CodeActionKind.QuickFix
        );
        updateRustVersion.diagnostics = [diagnostic];
        updateRustVersion.command = {
            command: 'ferrousForge.fix',
            title: 'Fix Rust Version',
            arguments: ['rust-version']
        };
        actions.push(updateRustVersion);

        const rustupUpdate = new vscode.CodeAction(
            'Run rustup update',
            vscode.CodeActionKind.QuickFix
        );
        rustupUpdate.diagnostics = [diagnostic];
        rustupUpdate.command = {
            command: 'workbench.action.terminal.sendSequence',
            title: 'Update Rust',
            arguments: [{ text: 'rustup update\n' }]
        };
        actions.push(rustupUpdate);

        return actions;
    }

    /**
     * Create quick fix for UnwrapInProduction violation
     *
     * @task T022
     */
    private createUnwrapFix(
        document: vscode.TextDocument,
        diagnostic: vscode.Diagnostic
    ): vscode.CodeAction[] {
        const actions: vscode.CodeAction[] = [];
        const line = document.lineAt(diagnostic.range.start.line);
        const lineText = line.text;

        // Replace .unwrap() with ?
        if (lineText.includes('.unwrap()') && !lineText.includes('ok()')) {
            const replaceWithQuestion = new vscode.CodeAction(
                'Replace .unwrap() with ?',
                vscode.CodeActionKind.QuickFix
            );
            replaceWithQuestion.diagnostics = [diagnostic];
            replaceWithQuestion.edit = new vscode.WorkspaceEdit();

            // Simple replacement - find .unwrap() and replace
            const unwrapIndex = lineText.indexOf('.unwrap()');
            if (unwrapIndex !== -1) {
                const range = new vscode.Range(
                    diagnostic.range.start.line,
                    unwrapIndex,
                    diagnostic.range.start.line,
                    unwrapIndex + '.unwrap()'.length
                );
                replaceWithQuestion.edit.replace(document.uri, range, '?');
                actions.push(replaceWithQuestion);
            }
        }

        // Replace .expect() with proper error handling
        if (lineText.includes('.expect(')) {
            const replaceExpect = new vscode.CodeAction(
                'Replace .expect() with match',
                vscode.CodeActionKind.QuickFix
            );
            replaceExpect.diagnostics = [diagnostic];
            replaceExpect.command = {
                command: 'editor.action.triggerSuggest',
                title: 'Trigger Suggestions'
            };
            actions.push(replaceExpect);
        }

        return actions;
    }

    /**
     * Create quick fix for LineTooLong violation
     *
     * @task T022
     */
    private createLineLengthFix(
        document: vscode.TextDocument,
        diagnostic: vscode.Diagnostic
    ): vscode.CodeAction[] {
        const actions: vscode.CodeAction[] = [];

        const runFmt = new vscode.CodeAction(
            'Format with rustfmt',
            vscode.CodeActionKind.QuickFix
        );
        runFmt.diagnostics = [diagnostic];
        runFmt.command = {
            command: 'editor.action.formatDocument',
            title: 'Format Document'
        };
        actions.push(runFmt);

        return actions;
    }

    /**
     * Create quick fix for UnderscoreBandaid violation
     *
     * @task T022
     */
    private createUnderscoreFix(
        document: vscode.TextDocument,
        diagnostic: vscode.Diagnostic
    ): vscode.CodeAction[] {
        const actions: vscode.CodeAction[] = [];
        const line = document.lineAt(diagnostic.range.start.line);
        const lineText = line.text;

        // Remove underscore prefix from unused variables
        const underscoreVarMatch = lineText.match(/let\s+_([a-zA-Z_][a-zA-Z0-9_]*)/);
        if (underscoreVarMatch) {
            const removeUnderscore = new vscode.CodeAction(
                `Remove underscore prefix from '${underscoreVarMatch[1]}'`,
                vscode.CodeActionKind.QuickFix
            );
            removeUnderscore.diagnostics = [diagnostic];
            removeUnderscore.edit = new vscode.WorkspaceEdit();

            const varStart = lineText.indexOf(`_${underscoreVarMatch[1]}`);
            const range = new vscode.Range(
                diagnostic.range.start.line,
                varStart,
                diagnostic.range.start.line,
                varStart + 1
            );
            removeUnderscore.edit.delete(document.uri, range);
            actions.push(removeUnderscore);
        }

        return actions;
    }

    /**
     * Create quick fix for MissingDocs violation
     *
     * @task T022
     */
    private createMissingDocsFix(
        document: vscode.TextDocument,
        diagnostic: vscode.Diagnostic
    ): vscode.CodeAction[] {
        const actions: vscode.CodeAction[] = [];
        const line = document.lineAt(diagnostic.range.start.line);

        // Add /// doc comment
        const addDocComment = new vscode.CodeAction(
            'Add documentation comment',
            vscode.CodeActionKind.QuickFix
        );
        addDocComment.diagnostics = [diagnostic];
        addDocComment.edit = new vscode.WorkspaceEdit();

        const indent = line.text.match(/^\s*/)?.[0] || '';
        const docLine = diagnostic.range.start.line;
        const position = new vscode.Position(docLine, 0);
        addDocComment.edit.insert(document.uri, position, `${indent}/// \n`);
        actions.push(addDocComment);

        // Add #[allow(missing_docs)]
        const addAllow = new vscode.CodeAction(
            'Add #[allow(missing_docs)]',
            vscode.CodeActionKind.QuickFix
        );
        addAllow.diagnostics = [diagnostic];
        addAllow.edit = new vscode.WorkspaceEdit();
        addAllow.edit.insert(document.uri, position, `${indent}#[allow(missing_docs)]\n`);
        actions.push(addAllow);

        return actions;
    }

    /**
     * Create refactor suggestion for file/function too large
     *
     * @task T022
     */
    private createRefactorSuggestion(
        diagnostic: vscode.Diagnostic
    ): vscode.CodeAction {
        const action = new vscode.CodeAction(
            'Consider refactoring (too large)',
            vscode.CodeActionKind.RefactorRewrite
        );
        action.diagnostics = [diagnostic];
        action.command = {
            command: 'vscode.executeDocumentSymbolProvider',
            title: 'View Document Symbols'
        };

        return action;
    }

    /**
     * Create "Fix All" action
     *
     * @task T022
     */
    private createFixAllAction(document: vscode.TextDocument): vscode.CodeAction {
        const action = new vscode.CodeAction(
            'Fix all Ferrous Forge violations',
            vscode.CodeActionKind.SourceFixAll
        );
        action.command = {
            command: 'ferrousForge.fix',
            title: 'Fix All Violations'
        };

        return action;
    }

    /**
     * Create "Ignore This Violation" action
     *
     * @task T022
     */
    private createIgnoreAction(
        document: vscode.TextDocument,
        range: vscode.Range
    ): vscode.CodeAction {
        const action = new vscode.CodeAction(
            'Ignore this violation (add #[allow])',
            vscode.CodeActionKind.QuickFix
        );

        const line = document.lineAt(range.start.line);
        const indent = line.text.match(/^\s*/)?.[0] || '';
        const position = new vscode.Position(range.start.line, 0);

        action.edit = new vscode.WorkspaceEdit();
        action.edit.insert(document.uri, position, `${indent}#[allow(clippy::all)]\n`);

        return action;
    }

    /**
     * Get documentation URL for a violation type
     *
     * @task T022
     */
    private getDocumentationUrl(violationCode: string): string | undefined {
        const docs: Record<string, string> = {
            'WrongEdition': 'https://doc.rust-lang.org/edition-guide/',
            'OldRustVersion': 'https://ferrous-forge.dev/docs/rust-version',
            'UnwrapInProduction': 'https://ferrous-forge.dev/docs/error-handling',
            'MissingDocs': 'https://doc.rust-lang.org/rustdoc/write-documentation.html',
            'LineTooLong': 'https://ferrous-forge.dev/docs/formatting',
            'FileTooLarge': 'https://ferrous-forge.dev/docs/code-organization',
            'FunctionTooLarge': 'https://ferrous-forge.dev/docs/code-organization'
        };

        return docs[violationCode];
    }

    /**
     * Resolve additional information for a code action
     */
    resolveCodeAction?(
        codeAction: vscode.CodeAction,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.CodeAction> {
        // Code actions are fully resolved in provideCodeActions
        return codeAction;
    }
}
