# Kagikanri (鍵管理) - Password Manager Project

## 🔐 Project Overview

**Kagikanri** is a modern, secure, self-hosted password manager that provides a beautiful web interface for the battle-tested `pass` password store, with optional passkey support for enhanced security.

### 🏗️ Architecture
- **Backend**: Rust with Axum web framework, SQLCipher database, Git2 sync
- **Frontend**: Svelte 5 + TypeScript + Tailwind CSS SPA
- **Security**: GPG encryption via pass, TOTP authentication, encrypted passkey storage
- **Deployment**: Docker containerization with embedded frontend

### 📂 Project Structure
```
kagikanri/
├── backend/           # Rust backend (Axum + API)
│   ├── src/
│   │   ├── main.rs    # Server entry point + routing
│   │   ├── auth.rs    # Master password + TOTP authentication
│   │   ├── pass.rs    # Pass CLI integration for password ops
│   │   ├── git.rs     # Git repository synchronization
│   │   ├── passkey.rs # SQLCipher passkey storage (WebAuthn)
│   │   ├── state.rs   # Application state management
│   │   ├── config.rs  # Environment configuration
│   │   ├── error.rs   # Error handling and types
│   │   └── handlers/  # API endpoint handlers
│   ├── Cargo.toml     # Rust dependencies
│   └── build.rs       # Build script for frontend embedding
├── frontend/          # Svelte 5 + TypeScript frontend
│   ├── src/
│   │   ├── routes/    # SvelteKit routes
│   │   │   ├── +layout.svelte   # Main app layout
│   │   │   └── +page.svelte     # Password list page
│   │   ├── lib/
│   │   │   ├── stores/
│   │   │   │   └── auth.ts      # Auth state management
│   │   │   └── components/
│   │   │       ├── AuthModal.svelte      # Login form
│   │   │       ├── Navigation.svelte     # Top navigation
│   │   │       ├── PasswordList.svelte   # Password listing
│   │   │       ├── PasswordCard.svelte   # Individual password
│   │   │       └── AddPasswordModal.svelte # Add new password
│   │   └── app.css    # Tailwind CSS imports
│   ├── package.json   # Node.js dependencies
│   └── svelte.config.js # SvelteKit configuration (SPA mode)
├── k8s/               # Kubernetes manifests
├── docs/              # Documentation
├── Dockerfile         # Multi-stage Docker build
└── README.md          # Project documentation
```

### 🛠️ Development Commands

**Frontend Development:**
```bash
cd frontend
pnpm install          # Install dependencies
pnpm dev              # Development server
pnpm build            # Production build
pnpm check            # Type checking
```

**Backend Development:**
```bash
cd backend
cargo check           # Type checking
cargo run             # Development server
cargo build --release # Production build
```

**Full Development Setup:**
```bash
# 1. Build frontend
cd frontend && pnpm install && pnpm build && cd ..

# 2. Set environment variables
export GIT_REPO_URL="https://github.com/user/password-store.git"
export GIT_ACCESS_TOKEN="your-github-token"
export DATABASE_ENCRYPTION_KEY="$(openssl rand -hex 32)"
export PASSWORD_STORE_DIR="/path/to/your/pass/store"

# 3. Run backend
cd backend && cargo run
```

### 🔑 Key Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `GIT_REPO_URL` | Pass store Git repository | `https://github.com/user/passwords.git` |
| `GIT_ACCESS_TOKEN` | Git authentication token | `ghp_xxxxxxxxxxxx` |
| `DATABASE_ENCRYPTION_KEY` | SQLCipher database key | `$(openssl rand -hex 32)` |
| `MASTER_PASSWORD_PATH` | Path to master password in pass | `kagikanri/master-password` |
| `TOTP_PATH` | Path to TOTP secret in pass | `kagikanri/totp` |

### 🧪 Testing & Verification

**Test backend API:**
```bash
# Health check
curl http://localhost:8080/api/health

# Test authentication (after setting up credentials)
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"master_password":"your-pass","totp_code":"123456"}'
```

**Frontend verification:**
```bash
# Check build output
ls -la frontend/build/

# Test SPA serving
curl http://localhost:8080/
```

### 🏭 Production Deployment

**Docker Build:**
```bash
docker build -t kagikanri:latest .
```

**Docker Run:**
```bash
docker run -d \
  --name kagikanri \
  -p 8080:8080 \
  -e GIT_REPO_URL="https://github.com/user/password-store.git" \
  -e GIT_ACCESS_TOKEN="your-token" \
  -e DATABASE_ENCRYPTION_KEY="$(openssl rand -hex 32)" \
  -v /path/to/gpg:/home/app/.gnupg:ro \
  -v kagikanri-data:/data \
  kagikanri:latest
```

### 🔧 Common Development Tasks

**Add new API endpoint:**
1. Create handler in `backend/src/handlers/`
2. Add route in `backend/src/main.rs`
3. Update frontend API calls in Svelte components

**Add new frontend component:**
1. Create `.svelte` file in `frontend/src/lib/components/`
2. Import and use in routes or other components
3. Add to TypeScript types if needed

**Update dependencies:**
```bash
# Frontend
cd frontend && pnpm update

# Backend  
cd backend && cargo update
```

### 🚨 Troubleshooting

**Frontend build fails:**
- Check Node.js version (requires 20+)
- Run `pnpm install` to update dependencies
- Check TypeScript errors with `pnpm check`

**Backend compilation fails:**
- Check Rust version (requires 1.75+)
- Run `cargo clean` then `cargo build`
- Check for missing system dependencies

**Pass integration issues:**
- Verify `pass` CLI is installed and working
- Check GPG key setup: `gpg --list-keys`
- Verify PASSWORD_STORE_DIR path exists

### Git Commit Guidelines

**Rules**:
- NEVER add co-authors, "Generated with" tags, or metadata
- Focus on what changed and why, not how or who
- Use present tense ("add feature" not "added feature")
- Use lowercase for description
- No period at the end of description
- Try to avoid adding commit message unless absolutely necessary
- Keep commit message under 50 characters
- Keep description line under 72 characters

Follow [Conventional Commits v1.0.0](https://www.conventionalcommits.org/en/v1.0.0/):

**Format**: `type(scope): description`

**Required components**:
- `type`: feat, fix, docs, style, refactor, test, chore
- `scope`: component/area affected (api, auth, ui, pass, git, etc.)
- `description`: concise description of changes

**Kagikanri-specific scopes**:
- `auth`: Authentication system (master password, TOTP, sessions)
- `pass`: Pass CLI integration and password operations
- `git`: Git synchronization functionality
- `passkey`: WebAuthn passkey storage
- `ui`: Frontend components and styling
- `api`: Backend API endpoints and handlers
- `config`: Configuration and environment setup
- `docker`: Container and deployment configuration

**Examples**:
- `feat(auth): add TOTP verification for login`
- `fix(pass): resolve password metadata parsing`
- `feat(ui): add password search and filtering`
- `refactor(api): extract common response handlers`
- `docs(readme): update Docker deployment guide`

### Always Use Standard CLI Tools for Initialization
- **Go Backend**: Use `go mod init` to initialize Go modules
- **React Frontend**: Use `npm create vite@latest my-app -- --template react`
- **Alpine.js**: `pnpm init vite@latest my-alpine-app -- --template vanilla-ts`
- ALWAYS use `pnpm` instead of `npm`
- ALWAYS use Typescript instead of Javascript
- **Svelte**: `npx sv create my-app`
- **Rust**: `cargo new`, `cargo add`
- **Zig**: `zig init`
- **Never manually create package.json or go.mod files**
- **ALWAYS use official scaffolding tools**

# Using Gemini CLI for Large Codebase Analysis

When analyzing large codebases or multiple files that might exceed context limits, use the Gemini CLI with its massive
context window. Use `gemini -p` to leverage Google Gemini's large context capacity.

## File and Directory Inclusion Syntax

Use the `@` syntax to include files and directories in your Gemini prompts. The paths should be relative to WHERE you run the
  gemini command:

### Examples:

**Single file analysis:**
`sh
gemini -p "@src/main.py Explain this file's purpose and structure"

#Multiple files:
gemini -p "@package.json @src/index.js Analyze the dependencies used in the code"

#Entire directory:
gemini -p "@src/ Summarize the architecture of this codebase"

#Multiple directories:
gemini -p "@src/ @tests/ Analyze test coverage for the source code"

#Current directory and subdirectories:
gemini -p "@./ Give me an overview of this entire project"

# Or use --all_files flag:
gemini --all_files -p "Analyze the project structure and dependencies"

#Implementation Verification Examples

#Check if a feature is implemented:
gemini -p "@src/ @lib/ Has dark mode been implemented in this codebase? Show me the relevant files and functions"

#Verify authentication implementation:
gemini -p "@src/ @middleware/ Is JWT authentication implemented? List all auth-related endpoints and middleware"

#Check for specific patterns:
gemini -p "@src/ Are there any React hooks that handle WebSocket connections? List them with file paths"

#Verify error handling:
gemini -p "@src/ @api/ Is proper error handling implemented for all API endpoints? Show examples of try-catch blocks"

#Check for rate limiting:
gemini -p "@backend/ @middleware/ Is rate limiting implemented for the API? Show the implementation details"

#Verify caching strategy:
gemini -p "@src/ @lib/ @services/ Is Redis caching implemented? List all cache-related functions and their usage"

#Check for specific security measures:
gemini -p "@src/ @api/ Are SQL injection protections implemented? Show how user inputs are sanitized"

#Verify test coverage for features:
gemini -p "@src/payment/ @tests/ Is the payment processing module fully tested? List all test cases"`

When to Use Gemini CLI

Use gemini -p when:
- Analyzing entire codebases or large directories
- Comparing multiple large files
- Need to understand project-wide patterns or architecture
- Current context window is insufficient for the task
- Working with files totaling more than 100KB
- Verifying if specific features, patterns, or security measures are implemented
- Checking for the presence of certain coding patterns across the entire codebase

Important Notes
- Paths in @ syntax are relative to your current working directory when invoking gemini
- The CLI will include file contents directly in the context
- No need for --yolo flag for read-only analysis
- Gemini's context window can handle entire codebases that would overflow Claude's context
- When checking implementations, be specific about what you're looking for to get accurate results