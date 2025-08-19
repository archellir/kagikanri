<script lang="ts">
	import { onMount } from 'svelte';

	interface Props {
		passwordPath: string;
		onDeleted: () => void;
	}

	let { passwordPath, onDeleted }: Props = $props();

	interface PasswordEntry {
		path: string;
		password: string;
		metadata?: {
			username?: string;
			url?: string;
			notes?: string;
			custom_fields?: Record<string, string>;
		};
	}

	let entry = $state<PasswordEntry | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let showPassword = $state(false);
	let copied = $state(false);
	let showOTP = $state(false);
	let otpCode = $state<string | null>(null);
	let otpExpires = $state(0);

	async function loadEntry() {
		loading = true;
		error = null;

		try {
			const response = await fetch(`/api/passwords/${encodeURIComponent(passwordPath)}`, {
				credentials: 'include'
			});

			if (!response.ok) {
				throw new Error('Failed to load password');
			}

			entry = await response.json();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load password';
		} finally {
			loading = false;
		}
	}

	async function copyToClipboard(text: string, type: string) {
		try {
			await navigator.clipboard.writeText(text);
			copied = true;
			setTimeout(() => copied = false, 2000);
		} catch (err) {
			console.error('Failed to copy to clipboard:', err);
		}
	}

	async function loadOTP() {
		try {
			const response = await fetch(`/api/otp/${encodeURIComponent(passwordPath)}`, {
				credentials: 'include'
			});

			if (response.ok) {
				const data = await response.json();
				otpCode = data.code;
				otpExpires = data.expires_in;
				showOTP = true;

				// Auto-refresh OTP
				const interval = setInterval(async () => {
					if (otpExpires <= 1) {
						await loadOTP();
					} else {
						otpExpires--;
					}
				}, 1000);

				// Clean up interval when component is destroyed or OTP is hidden
				return () => clearInterval(interval);
			}
		} catch (err) {
			console.error('Failed to load OTP:', err);
		}
	}

	async function deletePassword() {
		if (!confirm(`Are you sure you want to delete "${passwordPath}"?`)) {
			return;
		}

		try {
			const response = await fetch(`/api/passwords/${encodeURIComponent(passwordPath)}`, {
				method: 'DELETE',
				credentials: 'include'
			});

			if (!response.ok) {
				throw new Error('Failed to delete password');
			}

			onDeleted();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to delete password';
		}
	}

	function getDisplayName(path: string): string {
		const parts = path.split('/');
		return parts[parts.length - 1] || path;
	}

	function getDomain(url: string): string {
		try {
			return new URL(url).hostname;
		} catch {
			return url;
		}
	}

	onMount(() => {
		loadEntry();
	});
</script>

<div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4 hover:shadow-md transition-shadow">
	{#if loading}
		<div class="animate-pulse">
			<div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-3/4 mb-2"></div>
			<div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-1/2"></div>
		</div>
	{:else if error}
		<div class="text-red-600 dark:text-red-400 text-sm">{error}</div>
	{:else if entry}
		<!-- Header -->
		<div class="flex justify-between items-start mb-3">
			<div class="flex-1 min-w-0">
				<h3 class="text-lg font-medium text-gray-900 dark:text-white truncate">
					{getDisplayName(entry.path)}
				</h3>
				{#if entry.metadata?.url}
					<a 
						href={entry.metadata.url} 
						target="_blank" 
						rel="noopener noreferrer"
						class="text-sm text-blue-600 dark:text-blue-400 hover:underline"
					>
						{getDomain(entry.metadata.url)}
					</a>
				{/if}
				{#if entry.metadata?.username}
					<p class="text-sm text-gray-600 dark:text-gray-400">
						{entry.metadata.username}
					</p>
				{/if}
			</div>
			
			<!-- Actions Menu -->
			<div class="flex space-x-1">
				<button
					type="button"
					class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
					onclick={() => loadOTP()}
					title="Show OTP"
				>
					🔢
				</button>
				<button
					type="button"
					class="p-1 text-gray-400 hover:text-red-600"
					onclick={deletePassword}
					title="Delete"
				>
					🗑️
				</button>
			</div>
		</div>

		<!-- Password Field -->
		<div class="space-y-3">
			<div class="flex items-center space-x-2">
				<div class="flex-1">
					<input
						type={showPassword ? 'text' : 'password'}
						value={entry.password}
						readonly
						class="block w-full text-sm border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-md focus:ring-blue-500 focus:border-blue-500"
					/>
				</div>
				<button
					type="button"
					class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
					onclick={() => showPassword = !showPassword}
					title={showPassword ? 'Hide password' : 'Show password'}
				>
					{showPassword ? '🙈' : '👁️'}
				</button>
				<button
					type="button"
					class="p-2 text-gray-400 hover:text-blue-600 dark:hover:text-blue-400"
					onclick={() => copyToClipboard(entry.password, 'password')}
					title="Copy password"
				>
					{copied ? '✅' : '📋'}
				</button>
			</div>

			<!-- OTP Code -->
			{#if showOTP && otpCode}
				<div class="flex items-center space-x-2 bg-gray-50 dark:bg-gray-700 rounded-md p-2">
					<span class="text-sm font-mono font-bold text-gray-900 dark:text-white flex-1">
						{otpCode}
					</span>
					<span class="text-xs text-gray-500 dark:text-gray-400">
						{otpExpires}s
					</span>
					<button
						type="button"
						class="p-1 text-gray-400 hover:text-blue-600 dark:hover:text-blue-400"
						onclick={() => copyToClipboard(otpCode, 'OTP')}
						title="Copy OTP"
					>
						📋
					</button>
				</div>
			{/if}

			<!-- Notes -->
			{#if entry.metadata?.notes}
				<div class="text-sm text-gray-600 dark:text-gray-400 bg-gray-50 dark:bg-gray-700 rounded-md p-2">
					{entry.metadata.notes}
				</div>
			{/if}

			<!-- Custom Fields -->
			{#if entry.metadata?.custom_fields && Object.keys(entry.metadata.custom_fields).length > 0}
				<div class="space-y-1">
					{#each Object.entries(entry.metadata.custom_fields) as [key, value]}
						<div class="flex justify-between items-center text-sm">
							<span class="text-gray-600 dark:text-gray-400 font-medium">{key}:</span>
							<span class="text-gray-900 dark:text-white">{value}</span>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</div>