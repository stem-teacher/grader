#!/bin/bash

set -e

echo "🛠️  Setting up HSC Chemistry Grader development environment..."

# Update package lists
sudo apt-get update

# Install system dependencies
echo "📦 Installing system dependencies..."
sudo apt-get install -y \
    curl \
    wget \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    libpq-dev \
    wkhtmltopdf \
    xvfb \
    r-base \
    r-base-dev

# Install Quarto
echo "📚 Installing Quarto..."
wget -q https://github.com/quarto-dev/quarto-cli/releases/download/v1.4.550/quarto-1.4.550-linux-amd64.deb
sudo dpkg -i quarto-1.4.550-linux-amd64.deb || sudo apt-get install -f -y
rm quarto-1.4.550-linux-amd64.deb

# Install R packages needed for the frontend
echo "📊 Installing R packages..."
sudo R -e "
install.packages(c(
  'shiny',
  'learnr',
  'jsonlite',
  'htmltools',
  'httr',
  'DT'
), repos='https://cran.rstudio.com/')
"

# Install Fly.io CLI
echo "🚁 Installing Fly.io CLI..."
curl -L https://fly.io/install.sh | sh
echo 'export FLYCTL_INSTALL="/home/vscode/.fly"' >> ~/.bashrc
echo 'export PATH="$FLYCTL_INSTALL/bin:$PATH"' >> ~/.bashrc

# Set up Rust environment
echo "🦀 Setting up Rust environment..."
rustup component add clippy rustfmt
rustup target add x86_64-unknown-linux-gnu

# Install additional cargo tools
cargo install cargo-watch cargo-edit

# Set up backend environment
echo "⚙️  Setting up backend..."
cd backend
if [ ! -f ".env" ]; then
    cp .env.example .env
    echo "✅ Created .env file - please add your API keys!"
fi

# Check Rust dependencies
cargo check

cd ..

# Set up frontend environment
echo "🎨 Setting up frontend..."
cd frontend
npm install

cd ..

# Create development scripts
echo "📝 Creating development helpers..."

# Create a VS Code tasks configuration
mkdir -p .vscode
cat > .vscode/tasks.json << 'EOF'
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Run Backend",
            "type": "shell",
            "command": "cargo",
            "args": ["run"],
            "group": "build",
            "options": {
                "cwd": "${workspaceFolder}/backend"
            },
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "new"
            },
            "problemMatcher": "$rustc"
        },
        {
            "label": "Run Frontend",
            "type": "shell",
            "command": "quarto",
            "args": ["preview", "chemistry-exam.qmd", "--port", "3000"],
            "group": "build",
            "options": {
                "cwd": "${workspaceFolder}/frontend"
            },
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "new"
            }
        },
        {
            "label": "Run Full Stack",
            "dependsOrder": "parallel",
            "dependsOn": ["Run Backend", "Run Frontend"],
            "group": {
                "kind": "build",
                "isDefault": true
            }
        },
        {
            "label": "Test Backend",
            "type": "shell",
            "command": "cargo",
            "args": ["test"],
            "group": "test",
            "options": {
                "cwd": "${workspaceFolder}/backend"
            },
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "new"
            },
            "problemMatcher": "$rustc"
        }
    ]
}
EOF

# Create launch configuration for debugging
cat > .vscode/launch.json << 'EOF'
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Backend",
            "cargo": {
                "args": [
                    "build",
                    "--bin=hsc-chemistry-backend",
                    "--package=hsc-chemistry-backend"
                ],
                "filter": {
                    "name": "hsc-chemistry-backend",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}/backend",
            "env": {
                "RUST_LOG": "debug"
            }
        }
    ]
}
EOF

# Create workspace settings
cat > .vscode/settings.json << 'EOF'
{
    "rust-analyzer.cargo.buildScripts.enable": true,
    "rust-analyzer.checkOnSave.command": "check",
    "rust-analyzer.cargo.features": "all",
    "files.watcherExclude": {
        "**/target/**": true,
        "**/node_modules/**": true,
        "**/.git/objects/**": true,
        "**/.git/subtree-cache/**": true
    },
    "search.exclude": {
        "**/target": true,
        "**/node_modules": true,
        "**/.git": true
    },
    "terminal.integrated.defaultProfile.linux": "bash",
    "quarto.render.previewType": "external"
}
EOF

echo "✅ VS Code configuration created"

# Create storage and logs directories
mkdir -p storage logs

# Set up git hooks for development
echo "🔗 Setting up git hooks..."
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
echo "🔍 Running pre-commit checks..."

# Check Rust formatting
echo "📝 Checking Rust formatting..."
cd backend
cargo fmt -- --check || {
    echo "❌ Rust code is not formatted. Run 'cargo fmt' to fix."
    exit 1
}

# Run Rust clippy
echo "🔍 Running Rust lints..."
cargo clippy -- -D warnings || {
    echo "❌ Rust clippy found issues. Please fix them."
    exit 1
}

# Run tests
echo "🧪 Running backend tests..."
cargo test || {
    echo "❌ Backend tests failed."
    exit 1
}

echo "✅ All pre-commit checks passed!"
EOF

chmod +x .git/hooks/pre-commit

echo ""
echo "🎉 Development environment setup complete!"
echo ""
echo "📋 Available commands:"
echo "  - Start backend: cd backend && cargo run"
echo "  - Start frontend: cd frontend && quarto preview chemistry-exam.qmd"
echo "  - Run full stack: Use VS Code task 'Run Full Stack' (Ctrl+Shift+P > Tasks: Run Task)"
echo "  - Run tests: cd backend && cargo test"
echo "  - Format code: cd backend && cargo fmt"
echo "  - Lint code: cd backend && cargo clippy"
echo ""
echo "🔗 Useful URLs (when running):"
echo "  - Frontend: http://localhost:3000"
echo "  - Backend API: http://localhost:8080"
echo "  - Health check: http://localhost:8080/health"
echo ""
echo "⚙️  Next steps:"
echo "1. Edit backend/.env and add your API keys"
echo "2. Use VS Code Command Palette (Ctrl+Shift+P) > 'Tasks: Run Task' > 'Run Full Stack'"
echo "3. Open http://localhost:3000 to see the chemistry exam interface"
