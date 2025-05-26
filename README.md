# HSC Chemistry Digital Grader

A comprehensive digital examination platform for NSW HSC Chemistry with AI-powered grading capabilities.

## 🎯 Features

- **Interactive Exam Interface**: Quarto/learnr-based frontend with chemical equation editing
- **AI-Powered Grading**: Dual AI provider support (OpenAI o1-mini + Google Gemini 2.5 Pro)
- **Chemical Structure Drawing**: JSME editor integration for drawing molecular structures
- **Real-time LaTeX Preview**: Live rendering of chemical equations with mhchem
- **Comprehensive Feedback**: Detailed scoring and improvement suggestions
- **PDF Report Generation**: Professional exam results with downloadable reports
- **Scalable Infrastructure**: Rust backend with SurrealDB and Fly.io deployment

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Quarto/learnr │    │   Rust Backend   │    │   AI Services   │
│   Frontend      │───▶│   REST API       │───▶│ OpenAI/Gemini   │
│                 │    │                  │    │                 │
│ - Exam Interface│    │ - Submission API │    │ - Auto Grading  │
│ - LaTeX Preview │    │ - File Storage   │    │ - Feedback Gen  │
│ - Chem Drawing  │    │ - SurrealDB      │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+
- Node.js 18+
- Quarto CLI
- OpenAI API Key
- Google Gemini API Key

### Local Development

1. **Clone the repository**:
   ```bash
   git clone https://github.com/stem-teacher/grader.git
   cd grader
   ```

2. **Setup backend**:
   ```bash
   cd backend
   cp .env.example .env
   # Edit .env with your API keys
   cargo run
   ```

3. **Setup frontend**:
   ```bash
   cd frontend
   npm install
   quarto preview chemistry-exam.qmd --port 3000
   ```

### Using GitHub Codespaces

1. Click "Code" → "Create codespace on main"
2. Wait for setup to complete
3. Add your API keys to `backend/.env`
4. Run the development servers

## 📁 Project Structure

```
grader/
├── frontend/                 # Quarto/R frontend
│   ├── chemistry-exam.qmd   # Main exam document
│   ├── _quarto.yml          # Quarto configuration
│   └── assets/              # CSS, JS, and images
├── backend/                 # Rust API server
│   ├── src/
│   │   ├── handlers/        # HTTP request handlers
│   │   ├── services/        # Business logic
│   │   └── models/          # Data structures
│   └── Cargo.toml
├── marking-guidelines/      # AI grading prompts
├── deployment/             # Docker and Fly.io config
└── .github/workflows/      # CI/CD automation
```

## 🔧 Configuration

### Environment Variables

```env
# Backend (.env)
OPENAI_API_KEY=your-openai-key
GEMINI_API_KEY=your-gemini-key
DATABASE_URL=memory  # or /data/surrealdb for production
STORAGE_PATH=./storage
```

### AI Grading

The system uses a dual-provider approach:
- **Primary**: OpenAI o1-mini for detailed reasoning
- **Fallback**: Google Gemini 2.5 Pro for reliability

Marking guidelines are loaded from `marking-guidelines/review-prompt.md`.

## 🚀 Deployment

### Fly.io (Recommended)

1. **Install Fly.io CLI**:
   ```bash
   curl -L https://fly.io/install.sh | sh
   ```

2. **Deploy**:
   ```bash
   flyctl auth login
   flyctl apps create hsc-chemistry-grader
   flyctl volumes create hsc_chemistry_data --size 10
   flyctl secrets set OPENAI_API_KEY="your-key"
   flyctl secrets set GEMINI_API_KEY="your-key"
   flyctl deploy
   ```

### GitHub Actions

Automatic deployment triggers on push to `main`:
- Runs tests
- Builds and deploys to Fly.io
- Creates release tags

## 📊 Scaling

Configured for 200 concurrent users:
- **Compute**: 2 CPU cores, 2GB RAM
- **Database**: SurrealDB with RocksDB persistence
- **Storage**: 10GB volume + optional S3 replication
- **Monitoring**: Health checks and metrics

## 🔒 Security

- API keys stored as Fly.io secrets
- HTTPS enforced for all traffic
- Input validation on all endpoints
- CORS configured for frontend domain
- Rate limiting to prevent abuse

## 🧪 Testing

```bash
# Backend tests
cd backend && cargo test

# Frontend build test
cd frontend && npm run build
```

## 📚 Usage

1. **Students**: Access exam via submission code
2. **Chemical Equations**: Use LaTeX notation with mhchem
3. **Structure Drawing**: JSME editor with SMILES export
4. **Submission**: Automatic grading begins on submit
5. **Results**: View in browser or download PDF

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests where appropriate
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Issues**: [GitHub Issues](https://github.com/stem-teacher/grader/issues)
- **Discussions**: [GitHub Discussions](https://github.com/stem-teacher/grader/discussions)
- **Email**: [Support Email](mailto:support@stem-teacher.github.io)

## 🔗 Links

- **Live Demo**: [Coming Soon]
- **Documentation**: [Wiki](https://github.com/stem-teacher/grader/wiki)
- **API Docs**: [OpenAPI Spec](docs/api.yaml)

---

Built with ❤️ for NSW HSC Chemistry education.
