<script lang="ts">
	interface Props {
		onClose: () => void;
	}

	let { onClose }: Props = $props();

	let formData = $state({
		path: '',
		password: '',
		username: '',
		url: '',
		notes: ''
	});

	let loading = $state(false);
	let error = $state<string | null>(null);

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		
		if (!formData.path.trim() || !formData.password.trim()) {
			error = 'Path and password are required';
			return;
		}

		loading = true;
		error = null;

		try {
			const entry = {
				path: formData.path.trim(),
				password: formData.password,
				metadata: {
					...(formData.username && { username: formData.username }),
					...(formData.url && { url: formData.url }),
					...(formData.notes && { notes: formData.notes })
				}
			};

			const response = await fetch(`/api/passwords/${encodeURIComponent(formData.path.trim())}`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				credentials: 'include',
				body: JSON.stringify(entry)
			});

			if (!response.ok) {
				const errorData = await response.json();
				throw new Error(errorData.error || 'Failed to save password');
			}

			onClose();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to save password';
		} finally {
			loading = false;
		}
	}

	function generatePassword() {
		const length = 16;
		const charset = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*';
		let password = '';
		for (let i = 0; i < length; i++) {
			password += charset.charAt(Math.floor(Math.random() * charset.length));
		}
		formData.password = password;
	}
</script>

<div class="fixed inset-0 bg-gray-900 bg-opacity-50 flex items-center justify-center p-4 z-50">
	<div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6">
		<!-- Header -->
		<div class="flex justify-between items-center mb-6">
			<h2 class="text-xl font-bold text-gray-900 dark:text-white">Add Password</h2>
			<button
				type="button"
				class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
				onclick={onClose}
				aria-label="Close modal"
			>
				<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
				</svg>
			</button>
		</div>

		<!-- Error Message -->
		{#if error}
			<div class="mb-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-3">
				<div class="text-sm text-red-700 dark:text-red-300">{error}</div>
			</div>
		{/if}

		<!-- Form -->
		<form onsubmit={handleSubmit} class="space-y-4">
			<!-- Path -->
			<div>
				<label for="path" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
					Path <span class="text-red-500">*</span>
				</label>
				<input
					id="path"
					type="text"
					bind:value={formData.path}
					class="block w-full rounded-md border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white shadow-sm focus:border-blue-500 focus:ring-blue-500"
					placeholder="e.g., websites/github.com"
					required
				/>
				<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
					Use forward slashes to organize passwords in folders
				</p>
			</div>

			<!-- Password -->
			<div>
				<label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
					Password <span class="text-red-500">*</span>
				</label>
				<div class="flex space-x-2">
					<input
						id="password"
						type="text"
						bind:value={formData.password}
						class="block w-full rounded-md border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white shadow-sm focus:border-blue-500 focus:ring-blue-500"
						placeholder="Enter or generate password"
						required
					/>
					<button
						type="button"
						class="px-3 py-2 text-sm bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-md hover:bg-gray-200 dark:hover:bg-gray-600"
						onclick={generatePassword}
					>
						🎲
					</button>
				</div>
			</div>

			<!-- Username -->
			<div>
				<label for="username" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
					Username
				</label>
				<input
					id="username"
					type="text"
					bind:value={formData.username}
					class="block w-full rounded-md border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white shadow-sm focus:border-blue-500 focus:ring-blue-500"
					placeholder="Username or email"
				/>
			</div>

			<!-- URL -->
			<div>
				<label for="url" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
					URL
				</label>
				<input
					id="url"
					type="url"
					bind:value={formData.url}
					class="block w-full rounded-md border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white shadow-sm focus:border-blue-500 focus:ring-blue-500"
					placeholder="https://example.com"
				/>
			</div>

			<!-- Notes -->
			<div>
				<label for="notes" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
					Notes
				</label>
				<textarea
					id="notes"
					bind:value={formData.notes}
					rows="3"
					class="block w-full rounded-md border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white shadow-sm focus:border-blue-500 focus:ring-blue-500"
					placeholder="Additional notes..."
				></textarea>
			</div>

			<!-- Actions -->
			<div class="flex space-x-3 pt-4">
				<button
					type="button"
					class="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
					onclick={onClose}
					disabled={loading}
				>
					Cancel
				</button>
				<button
					type="submit"
					class="flex-1 px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
					disabled={loading || !formData.path.trim() || !formData.password.trim()}
				>
					{#if loading}
						<svg class="animate-spin -ml-1 mr-3 h-4 w-4 text-white inline" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
							<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
						</svg>
						Saving...
					{:else}
						Save Password
					{/if}
				</button>
			</div>
		</form>
	</div>
</div>