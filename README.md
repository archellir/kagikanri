# Kagikanri (鍵管理) - Modern Password Manager

Kagikanri is a modern, secure, self-hosted password manager that provides a beautiful web interface for the battle-tested `pass` password store, with optional passkey support for enhanced security.

## Features

- **Battle-tested Security**: Built on top of GPG-encrypted `pass` password store
- **Modern Web UI**: Beautiful Svelte 5 + TypeScript frontend with Tailwind CSS
- **Passkey Support**: Store and manage WebAuthn passkeys for other websites
- **Git Synchronization**: Automatic Git sync for password store backup
- **Responsive Design**: Works seamlessly on desktop and mobile
- **TOTP Integration**: Built-in support for 2FA codes via pass-otp
- **High Performance**: Rust backend with <100MB RAM usage
- **Container Ready**: Docker and Kubernetes deployment support

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Svelte 5 UI   │───▶│   Rust Backend   │───▶│  Pass CLI + GPG │
│   + Tailwind    │    │   (Axum + API)   │    │  Password Store │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │  SQLCipher DB   │
                       │ (Passkey Store) │
                       └─────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.75+
- Node.js 20+ with pnpm
- `pass` CLI tool installed
- GPG key set up for pass
- Git repository for password store

### Development Setup

1. **Clone the repository**
   ```bash
   git clone <your-repo-url>
   cd kagikanri
   ```

2. **Build the frontend**
   ```bash
   cd frontend
   pnpm install
   pnpm build
   cd ..
   ```

3. **Set up environment variables**
   ```bash
   export GIT_REPO_URL="https://github.com/user/password-store.git"
   export GIT_ACCESS_TOKEN="your-github-token"
   export DATABASE_ENCRYPTION_KEY="$(openssl rand -hex 32)"
   export PASSWORD_STORE_DIR="/path/to/your/pass/store"
   ```

4. **Run the backend**
   ```bash
   cd backend
   cargo run
   ```

5. **Access the web interface**
   Open http://localhost:8080 in your browser

### Docker Deployment

```bash
# Build the Docker image
docker build -t kagikanri:latest .

# Run with environment variables
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

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `GIT_REPO_URL` | Yes | - | Git repository URL for password store |
| `GIT_ACCESS_TOKEN` | Yes | - | Git access token for private repos |
| `DATABASE_ENCRYPTION_KEY` | Yes | - | 32-byte hex key for passkey database |
| `MASTER_PASSWORD_PATH` | No | `kagikanri/master-password` | Path to master password in pass store |
| `TOTP_PATH` | No | `kagikanri/totp` | Path to TOTP secret in pass store |
| `PORT` | No | `8080` | Server port |
| `PASSWORD_STORE_DIR` | No | `/data/password-store` | Pass store directory |
| `DATABASE_URL` | No | `sqlite:///data/passkeys.db` | Passkey database URL |
| `SYNC_INTERVAL_MINUTES` | No | `5` | Git sync interval |

### Pass Store Setup

1. **Initialize pass store** (if not already done)
   ```bash
   pass init <your-gpg-key-id>
   ```

2. **Set up Kagikanri credentials**
   ```bash
   # Master password for web UI login
   pass insert kagikanri/master-password
   
   # TOTP secret for 2FA (base32 encoded)
   pass otp insert kagikanri/totp
   ```

3. **Initialize Git repository**
   ```bash
   pass git init
   pass git remote add origin <your-repo-url>
   pass git push -u origin master
   ```

## Security

### Authentication Flow

1. **Master Password**: Primary authentication credential stored in pass store
2. **TOTP Verification**: Time-based OTP for additional security
3. **Session Management**: Secure HTTP-only cookies with expiration
4. **Git Sync**: Encrypted repository synchronization with access tokens

### Passkey Storage

- **Encrypted Database**: SQLCipher with unique encryption key
- **Per-Entry Salts**: Additional security for each stored passkey
- **WebAuthn Compliance**: Full WebAuthn specification support
- **Purpose**: Store passkeys for OTHER websites (Gmail, GitHub, etc.)

### Security Best Practices

- GPG-encrypted password storage via pass
- Database encryption for passkey storage
- No plaintext secrets in logs
- Secure session management
- Git repository encryption
- Container security hardening
- Non-root container execution

## Usage

### Web Interface

1. **Login**: Enter master password + TOTP code
2. **Browse Passwords**: Search and filter through your password store
3. **Add Passwords**: Create new entries with metadata
4. **Copy Credentials**: One-click copy for passwords and TOTP codes
5. **Manage Passkeys**: Register and store passkeys for external sites
6. **Sync Status**: Monitor Git synchronization

### API Endpoints

The backend provides a REST API:

- `POST /api/auth/login` - Authenticate with master password + TOTP
- `GET /api/passwords` - List all passwords
- `GET /api/passwords/*path` - Get specific password
- `POST /api/passwords/*path` - Create/update password
- `GET /api/otp/*path` - Get TOTP code
- `POST /api/sync` - Trigger Git sync
- `GET /api/health` - Health check

## Development

### Project Structure

```
kagikanri/
├── backend/           # Rust backend (Axum + API)
│   ├── src/
│   │   ├── auth.rs   # Authentication logic
│   │   ├── pass.rs   # Pass CLI integration
│   │   ├── git.rs    # Git synchronization
│   │   └── ...
│   └── Cargo.toml
├── frontend/          # Svelte 5 + TypeScript frontend
│   ├── src/
│   │   ├── routes/   # SvelteKit routes
│   │   ├── lib/      # Components and stores
│   │   └── ...
│   └── package.json
├── k8s/              # Kubernetes manifests
├── docs/             # Documentation
└── Dockerfile        # Multi-stage Docker build
```

### Technology Stack

**Backend:**
- Rust with Axum web framework
- SQLCipher for encrypted passkey storage
- Git2 for repository synchronization
- WebAuthn-rs for passkey support
- Pass CLI integration

**Frontend:**
- Svelte 5 with TypeScript
- SvelteKit for routing and SSG
- Tailwind CSS for styling
- Modern WebAuthn API

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Performance

- **Memory Usage**: <100MB RAM
- **Build Size**: ~50MB Docker image
- **Response Time**: <2 seconds for most operations
- **Bundle Size**: <50KB gzipped frontend

## Troubleshooting

### Common Issues

**Frontend not loading**: Ensure `pnpm build` was run in the frontend directory

**Pass commands failing**: Check GPG key setup and PASSWORD_STORE_DIR

**Git sync errors**: Verify GIT_ACCESS_TOKEN and repository permissions

**TOTP authentication failing**: Ensure TOTP secret is properly base32 encoded

### Logs

Enable debug logging:
```bash
export RUST_LOG=kagikanri=debug,tower_http=debug
```

## License

MIT License - see LICENSE file for details

## Acknowledgments

- [pass](https://www.passwordstore.org/) - The standard Unix password manager
- [GPG](https://gnupg.org/) - GNU Privacy Guard
- [Svelte](https://svelte.dev/) - Cybernetically enhanced web apps
- [Axum](https://github.com/tokio-rs/axum) - Ergonomic web framework for Rust
