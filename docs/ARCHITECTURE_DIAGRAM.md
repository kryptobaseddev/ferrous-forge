# Ferrous Forge Architecture Diagrams

## High-Level Flow

```mermaid
graph LR
    subgraph "Input"
        Code[Rust Codebase]
    end
    
    subgraph "Detection"
        Validate[Validation Engine]
        Standards[Standards Checker]
    end
    
    subgraph "Reporting"
        AIReport[AI Report Generator]
        JSONReport[JSON Report]
        MDReport[Markdown Report]
    end
    
    subgraph "Fixing - Layer 1"
        AutoFix[Conservative Auto-Fix]
        Context[Context Analyzer]
        Safety[Safety Checker]
    end
    
    subgraph "Fixing - Layer 2"
        AIAnalyzer[AI Analyzer]
        AST[AST Parser - syn]
        Semantic[Semantic Analysis]
        Confidence[Confidence Scoring]
    end
    
    subgraph "Outputs"
        Fixed[Fixed Code]
        Analysis[Analysis Report]
        Instructions[Orchestrator Instructions]
    end
    
    subgraph "AI Integration"
        Orchestrator[Claude Orchestrator]
        LLM[LLM Agent]
    end
    
    Code --> Validate
    Validate --> Standards
    Standards --> AIReport
    AIReport --> JSONReport
    AIReport --> MDReport
    
    Standards --> AutoFix
    AutoFix --> Context
    Context --> Safety
    Safety --> Fixed
    
    AutoFix --> AIAnalyzer
    AIAnalyzer --> AST
    AIAnalyzer --> Semantic
    Semantic --> Confidence
    Confidence --> Analysis
    Confidence --> Instructions
    
    Instructions --> Orchestrator
    Orchestrator --> LLM
    LLM --> Fixed
```

## Detailed Component Interaction

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Validator
    participant Reporter
    participant Fixer
    participant AIAnalyzer
    participant Orchestrator
    
    User->>CLI: ferrous-forge validate --ai-report
    CLI->>Validator: validate_project()
    Validator->>Validator: Check standards
    Validator-->>CLI: Violations[]
    CLI->>Reporter: generate_ai_report()
    Reporter-->>User: AI Report (JSON/MD)
    
    User->>CLI: ferrous-forge fix --ai-analysis
    CLI->>Fixer: execute_with_ai()
    
    rect rgb(200, 230, 200)
        Note over Fixer: Layer 1: Conservative Fix
        Fixer->>Fixer: analyze_file_context()
        Fixer->>Fixer: check_can_use_question_mark()
        Fixer->>Fixer: fix_safe_violations()
        Fixer-->>CLI: Fixed: 3, Skipped: 269
    end
    
    rect rgb(200, 200, 230)
        Note over AIAnalyzer: Layer 2: AI Analysis
        Fixer->>AIAnalyzer: analyze_violations_for_ai()
        AIAnalyzer->>AIAnalyzer: parse_AST()
        AIAnalyzer->>AIAnalyzer: semantic_analysis()
        AIAnalyzer->>AIAnalyzer: calculate_confidence()
        AIAnalyzer-->>Fixer: AIAnalysisReport
    end
    
    Fixer->>Orchestrator: generate_instructions()
    Orchestrator-->>User: Orchestrator Instructions (MD)
    
    Note over User,Orchestrator: Manual or automated AI fixing
    User->>Orchestrator: Use instructions
    Orchestrator->>Orchestrator: Apply intelligent fixes
    Orchestrator-->>User: Fixed code
```

## Data Flow

```mermaid
graph TD
    subgraph "Violation Detection"
        Source[Source Code] --> Parse[Parse Files]
        Parse --> Check{Check Standards}
        Check -->|Violation| ViolationObj[Violation Object]
    end
    
    subgraph "Violation Object"
        ViolationObj --> Type[ViolationType]
        ViolationObj --> File[File Path]
        ViolationObj --> Line[Line Number]
        ViolationObj --> Msg[Message]
        ViolationObj --> Severity[Severity Level]
    end
    
    subgraph "AI Report Generation"
        ViolationObj --> AIGen[AI Report Generator]
        AIGen --> Metadata[Metadata]
        AIGen --> Summary[Summary Stats]
        AIGen --> Details[Violation Details]
        AIGen --> FixInstr[Fix Instructions]
    end
    
    subgraph "Fix Process"
        ViolationObj --> FixEngine[Fix Engine]
        FixEngine --> Layer1{Conservative Fix?}
        Layer1 -->|Yes| Apply[Apply Fix]
        Layer1 -->|No| Layer2[AI Analysis]
        
        Layer2 --> Context[Extract Context]
        Layer2 --> Semantic[Semantic Analysis]
        Layer2 --> Score[Confidence Score]
        
        Score --> Strategy[Fix Strategy]
        Strategy --> OrchestratorInst[Orchestrator Instructions]
    end
    
    subgraph "Output Files"
        AIGen --> JSONFile[.json Report]
        AIGen --> MDFile[.md Report]
        Layer2 --> AnalysisJSON[Analysis .json]
        OrchestratorInst --> InstructMD[Instructions .md]
    end
```

## Fix Decision Tree

```mermaid
graph TD
    Start[Violation Detected] --> CheckType{Violation Type?}
    
    CheckType -->|UnwrapInProduction| CheckContext1[Check Context]
    CheckType -->|UnderscoreBandaid| CheckContext2[Check Context]
    CheckType -->|Other| Skip[Skip Auto-Fix]
    
    CheckContext1 --> IsTest{Test File?}
    IsTest -->|Yes| SkipTest[Skip - Tests OK]
    IsTest -->|No| HasResult{Returns Result?}
    
    HasResult -->|Yes| CanUseQ{Can Use '?'}
    HasResult -->|No| UseExpect[Convert to .expect()]
    
    CanUseQ -->|Yes| ApplyQ[Apply '?' Fix]
    CanUseQ -->|No| NeedContext{Has anyhow?}
    
    NeedContext -->|Yes| UseContext[Use .context()?]
    NeedContext -->|No| SkipNoContext[Skip - Need Manual]
    
    CheckContext2 --> IsTrait{Trait Method?}
    IsTrait -->|Yes| SkipTrait[Skip - Can't Modify]
    IsTrait -->|No| IsDropPattern{Drop Pattern?}
    
    IsDropPattern -->|Yes| SkipDrop[Skip - Intentional]
    IsDropPattern -->|No| ReturnsResult{Returns Result?}
    
    ReturnsResult -->|Yes| ApplyFix[Apply Fix]
    ReturnsResult -->|No| UseErrorHandle[if let Err(e)]
    
    Skip --> AIAnalysis[Run AI Analysis]
    SkipTest --> AIAnalysis
    SkipNoContext --> AIAnalysis
    SkipTrait --> AIAnalysis
    SkipDrop --> AIAnalysis
    
    AIAnalysis --> GenerateInstructions[Generate Instructions]
    GenerateInstructions --> Orchestrator[Claude Orchestrator]
```

## Confidence Score Calculation

```mermaid
graph LR
    Base[Base Score: 0.5] --> Check1{Has Function Name?}
    Check1 -->|Yes +0.1| Check2{Has Return Type?}
    Check1 -->|No| Check2
    
    Check2 -->|Yes +0.15| Check3{Has Function Calls?}
    Check2 -->|No| Check3
    
    Check3 -->|Yes +0.1| Check4{Has Error Imports?}
    Check3 -->|No| Check4
    
    Check4 -->|Yes +0.15| Final[Final Score]
    Check4 -->|No| Final
    
    Final --> Normalize[Min(score, 1.0)]
    Normalize --> Output[Confidence: 0.0-1.0]
```

## File System Structure

```
ferrous-forge/
├── src/
│   ├── validation.rs         # Core validation engine
│   ├── ai_analyzer.rs        # AI-powered analysis
│   └── commands/
│       ├── validate.rs       # Validate command + AI report
│       └── fix.rs            # Fix command (2-layer system)
│
├── .ferrous-forge/
│   ├── reports/              # AI compliance reports
│   │   ├── ai_compliance_*.json
│   │   ├── ai_compliance_*.md
│   │   ├── latest_ai_report.json
│   │   └── latest_ai_report.md
│   │
│   └── ai-analysis/          # AI analysis outputs
│       ├── ai_analysis_*.json
│       └── orchestrator_instructions_*.md
│
└── docs/
    ├── VIOLATION_FIX_FLOW.md
    └── ARCHITECTURE_DIAGRAM.md
```