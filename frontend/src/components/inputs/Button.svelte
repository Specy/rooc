<script lang="ts">
	import type { ColorName } from '$src/stores/themeStore';
	export let disabled = false;
	export let style = '';
	export let hasIcon = false;
	export let color: ColorName = 'secondary';
	export let hoverColor: ColorName | undefined = undefined;
	export let border: ColorName | undefined = undefined;
	export let active = false;
	export let title = '';
</script>

<button
	type="button"
	class="btn"
	class:hasIcon
	{title}
	style="
	--btn-color:var(--{color}); 
	--btn-text:var(--{color}-text); 
	--btn-color-hover:var(--{hoverColor ?? `${color}-10`});
	--border-color: var(--{border}-10);
	{style};
	"
	{disabled}
	on:click
	class:active
>
	<slot />
</button>

<style lang="scss">
	.btn {
		padding: 0.5rem 1rem;
		border-radius: 0.3rem;
		color: var(--btn-text);
		background-color: var(--btn-color);
		border: dashed 0.1rem var(--border-color);
		text-align: center;
		display: flex;
		transition: all 0.3s;
		font-size: 1em;
		align-items: center;
		justify-content: center;
		width: fit-content;
		user-select: none;
		font-family: Rubik;
		position: relative;
		cursor: pointer;
	}
	.active {
		background-color: var(--accent);
		color: var(--accent-text);
	}
	.btn:hover {
		background-color: var(--btn-color-hover);
	}
	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.btn:disabled:hover {
		background-color: var(--btn-color) !important;
	}
	.hasIcon {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0.35rem 0.6rem;
	}
</style>
