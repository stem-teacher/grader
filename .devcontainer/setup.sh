#!/bin/bash

set -e

echo "🚀 Setting up HSC Chemistry Grader development environment..."

# Install system dependencies
sudo apt-get update
sudo apt-get install -y wkhtmltopdf xvfb

# Install Quarto
echo "📦 Installing Quarto..."
wget -q https://github.com/quarto-dev/quarto-cli/releases/latest/download/quarto-linux-amd64.deb
sudo dpkg -i quarto-linux-amd64.deb
rm quarto-linux-amd64.deb

# Install Fly.io CLI
echo "🌐 Installing Fly.io CLI..."
curl -L https://fly.io/install.sh | sh
echo 'export PATH="$HOME/.fly/bin:$PATH"' >> ~/.bashrc

# Install Rust components
echo "🦀 Setting up Rust..."
rustup component add rustfmt clippy

# Setup frontend
echo "🎨 Setting up frontend..."
cd frontend
npm install
cd ..

# Setup backend
echo "⚙️ Setting up backend..."
cd backend
cp .env.example .env
cargo build
cd ..

# Create necessary directories
mkdir -p storage
mkdir -p logs

echo "✅ Development environment setup complete!"
echo ""
echo "To get started:"
echo "  1. Copy your API keys to backend/.env"
echo "  2. Run 'cargo run' in the backend directory"
echo "  3. Run 'quarto preview chemistry-exam.qmd' in the frontend directory"
echo ""
echo "Available ports:"
echo "  - 8080: Backend API"
echo "  - 3000: Frontend Preview"
