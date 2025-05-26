#!/bin/bash

set -e

echo "🛠️  Setting up HSC Chemistry Exam Project..."

# Check if we're in the project root
if [ ! -f "README.md" ] || [ ! -d "backend" ] || [ ! -d "frontend" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# Check if required tools are installed
echo "🔍 Checking required tools..."

# Check Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
else
    echo "✅ Rust/Cargo found"
fi

# Check Node.js
if ! command -v npm &> /dev/null; then
    echo "❌ Node.js/npm not found. Please install Node.js: https://nodejs.org/"
    exit 1
else
    echo "✅ Node.js/npm found"
fi

# Check R (optional for local development)
if command -v R &> /dev/null; then
    echo "✅ R found"
else
    echo "⚠️  R not found - required for local frontend development"
    echo "   Install R: https://cran.r-project.org/"
fi

# Check Quarto (optional for local development)
if command -v quarto &> /dev/null; then
    echo "✅ Quarto found"
else
    echo "⚠️  Quarto not found - required for local frontend development"
    echo "   Install Quarto: https://quarto.org/docs/get-started/"
fi

# Check Docker (optional)
if command -v docker &> /dev/null; then
    echo "✅ Docker found"
else
    echo "⚠️  Docker not found - optional for local development"
    echo "   Install Docker: https://docs.docker.com/get-docker/"
fi

# Check flyctl (optional)
if command -v flyctl &> /dev/null; then
    echo "✅ Fly.io CLI found"
else
    echo "⚠️  Fly.io CLI not found - required for deployment"
    echo "   Install flyctl: https://fly.io/docs/hands-on/install-flyctl/"
fi

echo ""
echo "📦 Setting up backend dependencies..."
cd backend

# Check if .env file exists, if not create from example
if [ ! -f ".env" ]; then
    if [ -f ".env.example" ]; then
        cp .env.example .env
        echo "✅ Created .env file from .env.example"
        echo "⚠️  Please edit backend/.env and add your API keys!"
    else
        echo "❌ No .env.example file found"
        exit 1
    fi
else
    echo "✅ .env file already exists"
fi

# Install Rust dependencies
echo "📥 Installing Rust dependencies..."
cargo check || {
    echo "❌ Failed to check Rust dependencies"
    exit 1
}

echo "✅ Rust dependencies OK"

cd ..

echo ""
echo "📦 Setting up frontend dependencies..."
cd frontend

# Install Node.js dependencies
if [ -f "package.json" ]; then
    echo "📥 Installing Node.js dependencies..."
    npm install || {
        echo "❌ Failed to install Node.js dependencies"
        exit 1
    }
    echo "✅ Node.js dependencies installed"
else
    echo "⚠️  No package.json found in frontend directory"
fi

cd ..

echo ""
echo "🔧 Setting up development environment..."

# Create necessary directories
mkdir -p storage
mkdir -p logs

echo "✅ Created storage and logs directories"

# Create a simple local development script
cat > run-local.sh << 'EOF'
#!/bin/bash

echo "🚀 Starting HSC Chemistry Exam in development mode..."

# Start backend in background
echo "📚 Starting backend server..."
cd backend
cargo run &
BACKEND_PID=$!
cd ..

# Wait for backend to start
sleep 5

# Start frontend
echo "🎨 Starting frontend server..."
cd frontend
if command -v quarto &> /dev/null; then
    quarto preview chemistry-exam.qmd --port 3000 &
    FRONTEND_PID=$!
    cd ..
    
    echo ""
    echo "✅ Services started!"
    echo "🌐 Frontend: http://localhost:3000"
    echo "🔧 Backend: http://localhost:8080"
    echo "💊 Health check: http://localhost:8080/health"
    echo ""
    echo "Press Ctrl+C to stop both services"
    
    # Wait for interrupt
    trap "echo '🛑 Stopping services...' && kill $BACKEND_PID $FRONTEND_PID 2>/dev/null && exit" INT
    wait
else
    echo "❌ Quarto not found. Please install Quarto to run the frontend."
    kill $BACKEND_PID 2>/dev/null
    exit 1
fi
EOF

chmod +x run-local.sh

echo "✅ Created run-local.sh script for development"

echo ""
echo "🎉 Setup complete!"
echo ""
echo "📋 Next steps:"
echo "1. Edit backend/.env and add your API keys (OPENAI_API_KEY, GEMINI_API_KEY)"
echo "2. For local development: ./run-local.sh"
echo "3. For deployment setup: flyctl auth login && flyctl apps create hsc-chemistry-exam"
echo "4. To deploy: ./scripts/deploy.sh"
echo ""
echo "📚 Documentation:"
echo "- README.md - Project overview and setup instructions"
echo "- backend/.env.example - Example environment configuration"
echo "- frontend/chemistry-exam.qmd - Main exam interface"
echo ""
echo "🔧 Development commands:"
echo "- Backend only: cd backend && cargo run"
echo "- Frontend only: cd frontend && quarto preview chemistry-exam.qmd"
echo "- Full stack: ./run-local.sh"
echo "- Tests: cd backend && cargo test"
echo ""
echo "🚀 Deployment commands:"
echo "- Deploy: ./scripts/deploy.sh (after setting environment variables)"
echo "- Check status: flyctl status --app hsc-chemistry-exam"
echo "- View logs: flyctl logs --app hsc-chemistry-exam"
