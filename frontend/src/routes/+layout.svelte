<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { authStore } from '$lib/stores/auth.js';
	import AuthModal from '$lib/components/AuthModal.svelte';
	import Navigation from '$lib/components/Navigation.svelte';

	let { children } = $props();
</script>

<svelte:head>
	<title>Kagikanri - Password Manager</title>
	<meta name="description" content="A modern, secure, self-hosted password manager" />
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="min-h-screen bg-gray-50 dark:bg-gray-900">
	{#if !authStore.authenticated}
		<AuthModal />
	{:else}
		<Navigation />
		<main class="container mx-auto px-4 py-6">
			{@render children?.()}
		</main>
	{/if}
</div>
