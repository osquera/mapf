import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		fs: {
			// Allow serving files from the workspace root (needed for hoisted node_modules)
			allow: ['..']
		}
	},

	build: {
		rollupOptions: {
			external: ['node:fs', 'node:crypto', 'fs', 'crypto', 'path', 'node:path']
		}
	},
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}'],
		environment: 'jsdom'
	}
});
