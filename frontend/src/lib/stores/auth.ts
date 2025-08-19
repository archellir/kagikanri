import { browser } from '$app/environment';

interface AuthState {
	authenticated: boolean;
	user_id: string | null;
	expires_at: string | null;
	loading: boolean;
	error: string | null;
}

function createAuthStore() {
	let state = $state<AuthState>({
		authenticated: false,
		user_id: null,
		expires_at: null,
		loading: false,
		error: null
	});

	return {
		get authenticated() {
			return state.authenticated;
		},
		get user_id() {
			return state.user_id;
		},
		get expires_at() {
			return state.expires_at;
		},
		get loading() {
			return state.loading;
		},
		get error() {
			return state.error;
		},

		async login(masterPassword: string, totpCode: string) {
			state.loading = true;
			state.error = null;

			try {
				const response = await fetch('/api/auth/login', {
					method: 'POST',
					headers: {
						'Content-Type': 'application/json',
					},
					body: JSON.stringify({
						master_password: masterPassword,
						totp_code: totpCode
					})
				});

				if (!response.ok) {
					const error = await response.json();
					throw new Error(error.error || 'Login failed');
				}

				const data = await response.json();
				state.authenticated = true;
				state.user_id = 'user'; // TODO: Get from response
				state.expires_at = data.expires_at;
				
				// Store session in cookie (handled by server)
				await this.checkStatus();
			} catch (error) {
				state.error = error instanceof Error ? error.message : 'Login failed';
				state.authenticated = false;
			} finally {
				state.loading = false;
			}
		},

		async logout() {
			state.loading = true;
			
			try {
				await fetch('/api/auth/logout', {
					method: 'POST',
					credentials: 'include'
				});
			} catch (error) {
				console.error('Logout error:', error);
			}

			state.authenticated = false;
			state.user_id = null;
			state.expires_at = null;
			state.loading = false;
			state.error = null;
		},

		async checkStatus() {
			if (!browser) return;

			try {
				const response = await fetch('/api/auth/status', {
					credentials: 'include'
				});

				if (response.ok) {
					const data = await response.json();
					state.authenticated = data.authenticated;
					state.user_id = data.user_id;
					state.expires_at = data.expires_at;
				} else {
					state.authenticated = false;
					state.user_id = null;
					state.expires_at = null;
				}
			} catch (error) {
				console.error('Auth status check failed:', error);
				state.authenticated = false;
			}
		},

		clearError() {
			state.error = null;
		}
	};
}

export const authStore = createAuthStore();

// Check auth status on startup
if (browser) {
	authStore.checkStatus();
}