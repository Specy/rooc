<script lang="ts">
	import Var from '$cmp/pipe/Var.svelte';
	import { formatNum } from '$cmp/pipe/utils';

	type Props = {
		constraints: Map<string, number>;
	};
	let { constraints }: Props = $props();
</script>

<div class="container">
	{#each constraints.entries() as [key, value]}
		<div class="element">
			<Var value={key} style='font-weight:bold; margin: 0 auto;'/>
			<div>{formatNum(value)}</div>
		</div>
	{/each}
</div>

<style>
	.container {
		display: flex;
        flex-wrap: wrap;
        background-color: var(--secondary-10);
        border-radius: 0.5rem;
        overflow: hidden;
        width: fit-content;
        gap: 0.1rem;
        border: solid 0.1rem var(--secondary-10);
	}
	.element {
		display: flex;
        flex-direction: column;
        gap: 1rem;
        padding: 0.5rem 1rem;
        min-width: 10rem;
        flex: 1;
        background-color: var(--secondary);
	}
</style>
