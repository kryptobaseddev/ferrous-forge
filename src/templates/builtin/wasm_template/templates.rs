//! WASM template content strings

/// www/index.html content
pub fn index_html_content() -> String {
    format!("{}{}{}", html_header(), html_body(), html_footer())
}

/// HTML header with styles
fn html_header() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>{{project_name}} - WASM Demo</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; }
        body { padding: 20px; background-color: #f5f5f5; }
        .container { background: white; padding: 30px; border-radius: 10px; }
        .container { box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #333; text-align: center; }
        .demo-section { margin: 20px 0; padding: 15px; border: 1px solid #ddd; }
        .demo-section { border-radius: 5px; }
        button { background: #007ACC; color: white; border: none; padding: 10px 20px; }
        button { border-radius: 5px; cursor: pointer; margin: 5px; }
        button:hover { background: #005A9C; }
        input { padding: 8px; margin: 5px; border: 1px solid #ccc; }
        input { border-radius: 3px; }
        #output { margin: 10px 0; padding: 10px; background: #f0f8ff; }
        #output { border-radius: 3px; }
    </style>
</head>"#.to_string()
}

/// HTML body content
fn html_body() -> String {
    r#"<body>
    <div class="container">
        <h1>{{project_name}} 🦀</h1>
        <p>WebAssembly demo powered by Rust</p>

        <div class="demo-section">
            <h3>Greeting Function</h3>
            <input type="text" id="name-input" placeholder="Enter your name" value="World">
            <button id="greet-button">Greet</button>
        </div>

        <div class="demo-section">
            <h3>Calculator</h3>
            <input type="number" id="calc-input" placeholder="Number" value="5">
            <button id="add-button">Add</button>
            <button id="subtract-button">Subtract</button>
            <button id="reset-button">Reset</button>
            <div>Current value: <span id="calc-value">0</span></div>
        </div>

        <div id="output"></div>
    </div>"#
        .to_string()
}

/// HTML footer
fn html_footer() -> String {
    r#"
    <script src="./index.js"></script>
</body>
</html>"#
        .to_string()
}

/// www/index.js content  
pub fn index_js_content() -> String {
    r#"import init, { greet, add, Calculator } from "../pkg/{{project_name}}.js";

async function run() {
    // Initialize the WASM module
    await init();

    // Create calculator instance
    const calculator = new Calculator();
    
    // Get DOM elements
    const nameInput = document.getElementById('name-input');
    const greetButton = document.getElementById('greet-button');
    const calcInput = document.getElementById('calc-input');
    const addButton = document.getElementById('add-button');
    const subtractButton = document.getElementById('subtract-button');
    const resetButton = document.getElementById('reset-button');
    const calcValue = document.getElementById('calc-value');
    const output = document.getElementById('output');

    // Helper function to log output
    function logOutput(message) {
        const div = document.createElement('div');
        div.textContent = `${new Date().toLocaleTimeString()}: ${message}`;
        output.appendChild(div);
        output.scrollTop = output.scrollHeight;
    }

    // Greet button handler
    greetButton.addEventListener('click', () => {
        const name = nameInput.value || 'World';
        const greeting = greet(name);
        logOutput(greeting);
    });

    // Calculator button handlers
    addButton.addEventListener('click', () => {
        const value = parseFloat(calcInput.value) || 0;
        const result = calculator.add(value);
        calcValue.textContent = result;
        logOutput(`Added ${value}, result: ${result}`);
    });

    subtractButton.addEventListener('click', () => {
        const value = parseFloat(calcInput.value) || 0;
        const result = calculator.subtract(value);
        calcValue.textContent = result;
        logOutput(`Subtracted ${value}, result: ${result}`);
    });

    resetButton.addEventListener('click', () => {
        calculator.reset();
        calcValue.textContent = calculator.value;
        logOutput('Calculator reset');
    });

    // Initial greeting
    logOutput('WASM module loaded successfully! 🚀');
    
    // Demo the simple add function
    const addResult = add(2, 3);
    logOutput(`Demo: add(2, 3) = ${addResult}`);
}

run();
"#
    .to_string()
}
