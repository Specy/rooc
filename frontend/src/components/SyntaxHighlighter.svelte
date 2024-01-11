<script lang="ts">
	import { onMount } from 'svelte';
	import hljs from 'highlight.js/lib/core';
	import Card from './layout/Card.svelte';
	import type { ColorName } from '$src/stores/themeStore';
	import { highlightJsGrammar } from '$src/lib/Rooc/hljsLanguage';

    export let style: string = ""
	export let source: string;
	let code: HTMLElement;
	onMount(() => {
		hljs.registerLanguage('rooc', () => highlightJsGrammar);
	});

	function highlight(sourceCode: string, el: HTMLElement) {
		if (!el) return;
		const highlighted = hljs.highlight(sourceCode, {language: "rooc"}).value;
		el.innerHTML = highlighted;
	}

	$: highlight(source, code);
</script>

<pre class="my_hljs" {style}><code bind:this={code}></code></pre>

<style lang="scss">
	.my_hljs {
        font-family: Consolas, "Courier New", monospace !important;
        font-size: 1rem;
        font-weight: normal;
        line-height: 22px;
	}
	:global(.hljs-identifierDefine) {
		color: #bb82d2;
	}
	:global(.hljs-keyword) {
		color: #ff6087;
	}
	:global(.hljs-number) {
		color: #69ac91;
	}
	:global(.hljs-string) {
		color: #d9b33f;
	}
	:global(.hljs-bracketsExpansion) {
		color: #ff6087;
	}
	:global(.hljs-graphDeclaration) {
		color: #00fff0;
	}
	:global(.hljs-operator) {
		color: #dcdcdc;
	}
	:global(.hljs-identifier) {
		color: #b9b9b9;
	}
	:global(.hljs-built_in) {
		color: #dc8455;
	}
	:global(.hljs-literal) {
		color: #69ac91;
	}
	:global(.hljs-brackets, .hljs-operator, .hljs-delimiter) {
		color: #dcdcdc;
	}
    :global(.hljs-identifierIgnore){
        color: #6a6a6a;
    }
</style>
