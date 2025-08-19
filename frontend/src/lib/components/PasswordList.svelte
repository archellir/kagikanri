<script lang="ts">
	import { onMount } from 'svelte';
	import PasswordCard from './PasswordCard.svelte';

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

	let passwords = $state<string[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let searchQuery = $state('');

	const filteredPasswords = $derived(
		passwords.filter(path => 
			path.toLowerCase().includes(searchQuery.toLowerCase())
		)
	);

	async function loadPasswords() {
		loading = true;
		error = null;

		try {
			const response = await fetch('/api/passwords', {
				credentials: 'include'
			});

			if (!response.ok) {
				throw new Error('Failed to load passwords');
			}

			const data = await response.json();
			passwords = data.entries || [];
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load passwords';
			console.error('Error loading passwords:', err);
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		loadPasswords();
	});
</script>

<div class="space-y-4">
	<!-- Search Bar -->
	<div class="relative">
		<div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
			<svg class="h-5 w-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
			</svg>
		</div>
		<input
			type="text"
			bind:value={searchQuery}
			class="block w-full pl-10 pr-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md leading-5 bg-white dark:bg-gray-700 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
			placeholder="Search passwords..."
		/>
	</div>

	<!-- Loading State -->
	{#if loading}
		<div class="flex justify-center items-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
		</div>
	{/if}

	<!-- Error State -->
	{#if error}
		<div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-4">
			<div class="flex">
				<svg class="h-5 w-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
				</svg>
				<div class="ml-3">
					<h3 class="text-sm font-medium text-red-800 dark:text-red-200">
						Error loading passwords
					</h3>
					<div class="mt-2 text-sm text-red-700 dark:text-red-300">
						{error}
					</div>
					<div class="mt-4">
						<button
							type="button"
							class="text-sm bg-red-100 dark:bg-red-900/50 text-red-800 dark:text-red-200 px-3 py-1 rounded-md hover:bg-red-200 dark:hover:bg-red-900/70"
							onclick={() => loadPasswords()}
						>
							Try again
						</button>
					</div>
				</div>
			</div>
		</div>
	{/if}

	<!-- Empty State -->
	{#if !loading && !error && filteredPasswords.length === 0}
		<div class="text-center py-12">
			<svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
			</svg>
			<h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-white">
				{searchQuery ? 'No passwords found' : 'No passwords yet'}
			</h3>
			<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
				{searchQuery ? 'Try adjusting your search terms.' : 'Get started by adding your first password.'}
			</p>
		</div>
	{/if}

	<!-- Password Cards -->
	{#if !loading && !error && filteredPasswords.length > 0}
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each filteredPasswords as passwordPath (passwordPath)}
				<PasswordCard {passwordPath} onDeleted={() => loadPasswords()} />
			{/each}
		</div>
	{/if}
</div>