<script lang="ts">
	import { page } from '$app/stores';
	import { authStore } from '$lib/stores/auth.js';

	let showUserMenu = $state(false);

	function handleLogout() {
		authStore.logout();
		showUserMenu = false;
	}
</script>

<nav class="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
	<div class="container mx-auto px-4">
		<div class="flex justify-between items-center h-16">
			<!-- Logo and Title -->
			<div class="flex items-center space-x-4">
				<a href="/" class="flex items-center space-x-3">
					<span class="text-2xl">🔐</span>
					<div>
						<h1 class="text-xl font-bold text-gray-900 dark:text-white">Kagikanri</h1>
						<p class="text-xs text-gray-500 dark:text-gray-400">鍵管理</p>
					</div>
				</a>
			</div>

			<!-- Navigation Links -->
			<div class="hidden md:flex items-center space-x-8">
				<a 
					href="/"
					class="text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 px-3 py-2 rounded-md text-sm font-medium transition-colors"
					class:text-blue-600={$page.url.pathname === '/'}
					class:dark:text-blue-400={$page.url.pathname === '/'}
				>
					Passwords
				</a>
				<a 
					href="/passkeys"
					class="text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 px-3 py-2 rounded-md text-sm font-medium transition-colors"
					class:text-blue-600={$page.url.pathname === '/passkeys'}
					class:dark:text-blue-400={$page.url.pathname === '/passkeys'}
				>
					Passkeys
				</a>
				<a 
					href="/sync"
					class="text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 px-3 py-2 rounded-md text-sm font-medium transition-colors"
					class:text-blue-600={$page.url.pathname === '/sync'}
					class:dark:text-blue-400={$page.url.pathname === '/sync'}
				>
					Sync
				</a>
			</div>

			<!-- User Menu -->
			<div class="relative">
				<button
					type="button"
					class="flex items-center space-x-2 text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white"
					onclick={() => showUserMenu = !showUserMenu}
				>
					<div class="w-8 h-8 bg-blue-600 rounded-full flex items-center justify-center">
						<span class="text-white text-sm font-medium">
							{authStore.user_id?.charAt(0).toUpperCase() || 'U'}
						</span>
					</div>
					<span class="hidden sm:block text-sm font-medium">
						{authStore.user_id || 'User'}
					</span>
					<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
					</svg>
				</button>

				{#if showUserMenu}
					<div class="absolute right-0 mt-2 w-48 bg-white dark:bg-gray-800 rounded-md shadow-lg py-1 z-50 border border-gray-200 dark:border-gray-700">
						<div class="px-4 py-2 text-xs text-gray-500 dark:text-gray-400 border-b border-gray-100 dark:border-gray-700">
							Signed in as {authStore.user_id}
						</div>
						<button
							type="button"
							class="block w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
							onclick={handleLogout}
						>
							Sign out
						</button>
					</div>
				{/if}
			</div>
		</div>
	</div>

	<!-- Mobile Navigation -->
	<div class="md:hidden border-t border-gray-200 dark:border-gray-700">
		<div class="px-2 pt-2 pb-3 space-y-1">
			<a 
				href="/"
				class="block px-3 py-2 rounded-md text-base font-medium text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400"
				class:text-blue-600={$page.url.pathname === '/'}
				class:dark:text-blue-400={$page.url.pathname === '/'}
			>
				Passwords
			</a>
			<a 
				href="/passkeys"
				class="block px-3 py-2 rounded-md text-base font-medium text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400"
				class:text-blue-600={$page.url.pathname === '/passkeys'}
				class:dark:text-blue-400={$page.url.pathname === '/passkeys'}
			>
				Passkeys
			</a>
			<a 
				href="/sync"
				class="block px-3 py-2 rounded-md text-base font-medium text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400"
				class:text-blue-600={$page.url.pathname === '/sync'}
				class:dark:text-blue-400={$page.url.pathname === '/sync'}
			>
				Sync
			</a>
		</div>
	</div>
</nav>

<!-- Close user menu when clicking outside -->
{#if showUserMenu}
	<button 
		type="button"
		class="fixed inset-0 z-40 bg-transparent border-0 cursor-default"
		onclick={() => showUserMenu = false}
		aria-label="Close menu"
	></button>
{/if}