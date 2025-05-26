# GitHub Codespace Development Guide

This guide covers development in GitHub Codespaces for the HSC Chemistry Digital Exam system.

## 🚀 Quick Start in Codespace

1. **Open in Codespace**
   - Go to https://github.com/stem-teacher/grader
   - Click the green "Code" button
   - Select "Codespaces" tab
   - Click "Create codespace on main"

2. **Initial Setup** (happens automatically)
   - The devcontainer will install all dependencies
   - This takes about 5-10 minutes on first run
   - You'll see setup progress in the terminal

3. **Configure API Keys**
   ```bash
   cd backend
   # Edit the .env file with your API keys
   code .env
   ```
   Add your keys:
   ```
   OPENAI_API_KEY=your-openai-key-here
   GEMINI_API_KEY=your-gemini-key-here
   ```

4. **Start Development**
   - Press `Ctrl+Shift+P` (Command Palette)
   - Type "Tasks: Run Task"
   - Select "Run Full Stack"
   - This starts both backend and frontend servers

## 🛠️ Development Workflow

### Using VS Code Tasks

The project includes pre-configured VS Code tasks accessible via `Ctrl+Shift+P` > "Tasks: Run Task":

- **Run Full Stack**: Starts both backend (port 8080) and frontend (port 3000)
- **Run Backend**: Backend API server only
- **Run Frontend**: Quarto preview server only  
- **Test Backend**: Runs all Rust unit tests

### Manual Commands

```bash
# Backend development
cd backend
cargo run                    # Start server
cargo test                   # Run tests
cargo clippy                 # Lint code
cargo fmt                    # Format code
cargo watch -x run           # Auto-restart on changes

# Frontend development  
cd frontend
quarto preview chemistry-exam.qmd --port 3000    # Start preview
quarto render                                     # Build static files

# Full project
./scripts/setup.sh           # Initial setup
./scripts/deploy.sh          # Deploy to Fly.io (requires setup)
```

## 🌐 Accessing Your Application

GitHub Codespaces automatically forwards ports:

- **Frontend**: Click the "Open in Browser" button for port 3000
- **Backend API**: Click the "Open in Browser" button for port 8080  
- **Health Check**: Visit `{codespace-url}-8080.preview.app.github.dev/health`

The URLs follow the pattern: `{codespace-name}-{port}.preview.app.github.dev`

## 🧪 Testing the Application

1. **Backend Health Check**
   ```bash
   curl http://localhost:8080/health
   ```

2. **Submit Test Exam**
   - Open the frontend URL
   - Enter a test submission code: `TEST-2023-001-ABC`
   - Fill out some answers
   - Submit and check grading status

3. **API Testing**
   ```bash
   # Test submission endpoint
   curl -X POST http://localhost:8080/api/submissions \
     -H "Content-Type: application/json" \
     -d '{"submission_code":"TEST-001","responses":{"multiple_choice":{"q1":"A"},"extended_response":{},"time_taken_minutes":30}}'
   ```

## 🔧 Debugging

### Backend Debugging

1. **Using VS Code Debugger**
   - Set breakpoints in Rust code
   - Press `F5` or use "Run and Debug" panel
   - Select "Debug Backend" configuration

2. **Log Output**
   ```bash
   cd backend
   RUST_LOG=debug cargo run
   ```

### Frontend Debugging

1. **Quarto Development**
   - Errors appear in the terminal running the preview
   - Browser console shows JavaScript errors
   - R errors appear in the Quarto terminal

2. **Shiny Debugging**
   - Add `browser()` statements in R code for breakpoints
   - Use `print()` or `cat()` for logging
   - Check browser developer tools for JavaScript issues

## 📦 Project Structure in Codespace

```
/workspaces/grader/
├── .devcontainer/          # Codespace configuration
├── .github/workflows/      # CI/CD pipelines  
├── .vscode/               # VS Code settings & tasks
├── backend/               # Rust API server
│   ├── src/              # Source code
│   ├── Cargo.toml        # Dependencies
│   └── .env              # Environment variables
├── frontend/             # Quarto/learnr application
│   ├── chemistry-exam.qmd # Main exam interface
│   ├── assets/           # CSS, JS, images
│   └── package.json      # Node dependencies
├── deployment/           # Fly.io configuration
├── scripts/             # Setup and deployment scripts
└── marking-guidelines/  # AI grading prompts
```

## 🚀 Deployment from Codespace

1. **Install Fly.io CLI** (already done in devcontainer)
   ```bash
   flyctl auth login
   ```

2. **Create Fly.io App**
   ```bash
   flyctl apps create hsc-chemistry-exam
   flyctl volumes create hsc_chemistry_data --size 10
   ```

3. **Set Environment Variables**
   ```bash
   export OPENAI_API_KEY="your-key"
   export GEMINI_API_KEY="your-key"
   ```

4. **Deploy**
   ```bash
   ./scripts/deploy.sh
   ```

## 🔍 Monitoring and Logs

```bash
# View application logs
flyctl logs --app hsc-chemistry-exam

# Check deployment status  
flyctl status --app hsc-chemistry-exam

# Connect to production app
flyctl ssh console --app hsc-chemistry-exam
```

## 💡 Tips for Codespace Development

1. **Extensions**: The devcontainer includes all necessary VS Code extensions
2. **Terminal**: Use multiple terminals for backend/frontend simultaneously
3. **Port Forwarding**: Ports are automatically forwarded and accessible via browser
4. **File Sync**: All changes sync automatically with GitHub
5. **Environment**: Full Linux environment with sudo access for additional tools

## 🐛 Troubleshooting

### Common Issues

1. **Port Already in Use**
   ```bash
   # Kill processes on port 8080
   sudo fuser -k 8080/tcp
   ```

2. **Dependencies Not Installing**
   ```bash
   # Rebuild devcontainer
   # Ctrl+Shift+P > "Codespaces: Rebuild Container"
   ```

3. **R Package Issues**
   ```bash
   # Install missing R packages
   sudo R -e "install.packages('package_name')"
   ```

4. **Rust Build Issues**
   ```bash
   cd backend
   cargo clean
   cargo build
   ```

### Getting Help

- Check logs in VS Code terminal output
- Use GitHub Issues for bug reports
- Consult the main README.md for additional documentation
- Review Fly.io documentation for deployment issues

## 🔐 Security in Codespace

- Never commit API keys to git
- Use GitHub Secrets for production deployment
- Environment variables in `.env` files are gitignored
- Codespace secrets are isolated per user

This setup provides a complete development environment that mirrors production while being accessible from any browser!
