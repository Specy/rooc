import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { resolve } from 'path';
import topLevelAwait from 'vite-plugin-top-level-await';
import wasm from 'vite-plugin-wasm';
import Icons from 'unplugin-icons/vite';
export default defineConfig({
	server: {
		port: 3000,
	},
	plugins: [
		sveltekit(),
		Icons({
			compiler: 'svelte',
			autoInstall: true
		}),
		topLevelAwait(),
		wasm()
	],
	resolve: {
		alias: {
			$cmp: resolve('./src/components/'),
			$src: resolve('./src/'),
			$stores: resolve('./src/stores/'),
			$utils: resolve('./src/utils/')
		}
	},
	optimizeDeps: {
		exclude: ["@specy/rooc"]
	}
});
