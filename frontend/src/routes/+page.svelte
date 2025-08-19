<script lang="ts">
	import PasswordList from '$lib/components/PasswordList.svelte';
	import AddPasswordModal from '$lib/components/AddPasswordModal.svelte';

	let showAddModal = $state(false);
</script>

<svelte:head>
	<title>Passwords - Kagikanri</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex justify-between items-center">
		<div>
			<h1 class="text-2xl font-bold text-gray-900 dark:text-white">Passwords</h1>
			<p class="text-gray-600 dark:text-gray-400">Manage your encrypted passwords</p>
		</div>
		<button
			type="button"
			class="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
			onclick={() => showAddModal = true}
		>
			<svg class="-ml-1 mr-2 h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
			</svg>
			Add Password
		</button>
	</div>

	<!-- Password List -->
	<PasswordList />

	<!-- Add Password Modal -->
	{#if showAddModal}
		<AddPasswordModal onClose={() => showAddModal = false} />
	{/if}
</div>
