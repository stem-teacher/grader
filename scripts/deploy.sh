#!/bin/bash

set -e

echo "🚀 Deploying HSC Chemistry Exam to Fly.io..."

# Ensure we're in the right directory
cd "$(dirname "$0")/.."

# Check if required environment variables are set
if [ -z "$OPENAI_API_KEY" ] || [ -z "$GEMINI_API_KEY" ]; then
    echo "❌ Error: OPENAI_API_KEY and GEMINI_API_KEY must be set"
    echo "Please set these environment variables before running this script:"
    echo "export OPENAI_API_KEY='your-openai-key'"
    echo "export GEMINI_API_KEY='your-gemini-key'"
    exit 1
fi

# Check if flyctl is installed
if ! command -v flyctl &> /dev/null; then
    echo "❌ Error: flyctl is not installed"
    echo "Please install flyctl: https://fly.io/docs/hands-on/install-flyctl/"
    exit 1
fi

# Check if logged in to Fly.io
if ! flyctl auth whoami &> /dev/null; then
    echo "❌ Error: Not logged in to Fly.io"
    echo "Please run: flyctl auth login"
    exit 1
fi

# Set secrets in Fly.io
echo "🔐 Setting secrets..."
flyctl secrets set OPENAI_API_KEY="$OPENAI_API_KEY" --app hsc-chemistry-exam || {
    echo "❌ Failed to set OpenAI API key. Make sure the app exists."
    echo "Create the app with: flyctl apps create hsc-chemistry-exam"
    exit 1
}

flyctl secrets set GEMINI_API_KEY="$GEMINI_API_KEY" --app hsc-chemistry-exam

echo "📦 Building and deploying application..."
flyctl deploy --remote-only --app hsc-chemistry-exam

# Wait for deployment to complete
echo "⏳ Waiting for deployment to complete..."
flyctl status --wait-timeout=300 --app hsc-chemistry-exam

# Run health check
echo "🔍 Running health check..."
HEALTH_URL="https://hsc-chemistry-exam.fly.dev/health"
MAX_ATTEMPTS=10
ATTEMPT=1

while [ $ATTEMPT -le $MAX_ATTEMPTS ]; do
    echo "Health check attempt $ATTEMPT/$MAX_ATTEMPTS..."
    if curl -f --max-time 10 "$HEALTH_URL" > /dev/null 2>&1; then
        echo "✅ Deployment successful! Health check passed."
        echo "🌐 Application is available at: https://hsc-chemistry-exam.fly.dev"
        echo "📊 Admin dashboard: https://fly.io/apps/hsc-chemistry-exam"
        exit 0
    else
        echo "⏳ Health check failed, waiting 10 seconds before retry..."
        sleep 10
        ATTEMPT=$((ATTEMPT + 1))
    fi
done

echo "❌ Health check failed after $MAX_ATTEMPTS attempts. Checking logs:"
flyctl logs --app hsc-chemistry-exam
echo ""
echo "❓ Troubleshooting tips:"
echo "1. Check the logs above for errors"
echo "2. Verify your secrets are set correctly: flyctl secrets list --app hsc-chemistry-exam"
echo "3. Check app status: flyctl status --app hsc-chemistry-exam"
echo "4. Try redeploying: flyctl deploy --app hsc-chemistry-exam"
exit 1
