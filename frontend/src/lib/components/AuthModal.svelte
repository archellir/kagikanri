<script lang="ts">
	import { authStore } from '$lib/stores/auth.js';

	let masterPassword = $state('');
	let totpCode = $state('');
	let showPassword = $state(false);

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		
		if (!masterPassword.trim() || !totpCode.trim()) {
			return;
		}

		await authStore.login(masterPassword, totpCode);
		
		// Clear form on successful login
		if (authStore.authenticated) {
			masterPassword = '';
			totpCode = '';
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			const form = event.target?.closest('form');
			if (form) {
				const submitEvent = new SubmitEvent('submit', { bubbles: true, cancelable: true });
				handleSubmit(submitEvent);
			}
		}
	}
</script>

<div class="fixed inset-0 bg-gray-900 bg-opacity-50 flex items-center justify-center p-4 z-50">
	<div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6">
		<!-- Header -->
		<div class="text-center mb-8">
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
				🔐 Kagikanri
			</h1>
			<p class="text-gray-600 dark:text-gray-300">
				鍵管理 - Secure Password Management
			</p>
		</div>

		<!-- Login Form -->
		<form onsubmit={handleSubmit} class="space-y-6">
			{#if authStore.error}
				<div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-4">
					<div class="flex">
						<div class="ml-3">
							<h3 class="text-sm font-medium text-red-800 dark:text-red-200">
								Authentication Failed
							</h3>
							<div class="mt-2 text-sm text-red-700 dark:text-red-300">
								{authStore.error}
							</div>
						</div>
						<button
							type="button"
							class="ml-auto text-red-500 hover:text-red-700"
							onclick={() => authStore.clearError()}
						>
							×
						</button>
					</div>
				</div>
			{/if}

			<!-- Master Password -->
			<div>
				<label for="master-password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
					Master Password
				</label>
				<div class="relative">
					<input
						id="master-password"
						name="master-password"
						type={showPassword ? 'text' : 'password'}
						bind:value={masterPassword}
						onkeydown={handleKeydown}
						class="block w-full rounded-md border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white shadow-sm focus:border-blue-500 focus:ring-blue-500 pr-10"
						placeholder="Enter your master password"
						required
						autocomplete="current-password"
					/>
					<button
						type="button"
						class="absolute inset-y-0 right-0 pr-3 flex items-center"
						onclick={() => showPassword = !showPassword}
					>
						<span class="text-gray-400 hover:text-gray-600 text-sm">
							{showPassword ? '🙈' : '👁️'}
						</span>
					</button>
				</div>
			</div>

			<!-- TOTP Code -->
			<div>
				<label for="totp-code" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
					TOTP Code
				</label>
				<input
					id="totp-code"
					name="totp-code"
					type="text"
					bind:value={totpCode}
					onkeydown={handleKeydown}
					class="block w-full rounded-md border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white shadow-sm focus:border-blue-500 focus:ring-blue-500"
					placeholder="123456"
					pattern="[0-9]{6}"
					maxlength="6"
					required
					autocomplete="one-time-code"
				/>
				<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
					Enter the 6-digit code from your authenticator app
				</p>
			</div>

			<!-- Submit Button -->
			<button
				type="submit"
				disabled={authStore.loading || !masterPassword.trim() || !totpCode.trim()}
				class="w-full flex justify-center py-3 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
			>
				{#if authStore.loading}
					<svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
					</svg>
					Authenticating...
				{:else}
					Sign In
				{/if}
			</button>
		</form>

		<!-- Footer -->
		<div class="mt-8 text-center">
			<p class="text-xs text-gray-500 dark:text-gray-400">
				Secured with GPG encryption and Git synchronization
			</p>
		</div>
	</div>
</div>