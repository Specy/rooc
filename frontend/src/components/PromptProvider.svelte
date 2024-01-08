<script lang="ts">
	import { prompter, PromptType } from '$stores/promptStore'
	import { fade } from 'svelte/transition'

    import Button from './inputs/Button.svelte';
    import Input from './inputs/Input.svelte';
	import { onMount } from 'svelte';
	import { toast } from '$src/stores/toastStore';
	let input: HTMLInputElement
	let value = ''
	$: if (!$prompter.promise) {
		value = ''
		if (input) input.value = ''
	}
</script>

<slot />
{#if $prompter.promise}
	<form
		class="prompt-wrapper"
		out:fade|global={{ duration: 150 }}
		on:submit={(e) => {
			e.preventDefault()
			prompter.answer(value)
		}}
	>
		<div class="prompt-text">
			{$prompter.question}
		</div>
		{#if $prompter.type === PromptType.Text}
			<Input
				bind:el={input}
				focus
				bind:value
				style="color: var(--primary-text); background-color: var(--primary);"
			/>
		{/if}

		<div class="prompt-row">
			{#if $prompter.type === PromptType.Text}
				{#if !$prompter.cancellable}
					<Button on:click={() => prompter.answer(false)} color="secondary">Cancel</Button>
				{/if}
				<Button on:click={() => prompter.answer(value)} style="margin-left: auto;">Ok</Button>
			{:else}
				<Button on:click={() => prompter.answer(false)} color="secondary">No</Button>
				<Button on:click={() => prompter.answer(true)} color="accent2">Yes</Button>
			{/if}
		</div>
	</form>
{/if}

<style lang="scss">
	.prompt-wrapper {
		display: flex;
		position: fixed;
		top: 1rem;
		overflow: hidden;
		max-height: 10rem;
		width: 20rem;
		color: var(--secondary-text);
		backdrop-filter: blur(3px);
		border-radius: 0.5rem;
		background-color: rgba(var(--primary-rgb), 0.8);
		border: dashed 0.15rem var(--secondary);
		box-shadow: 0 3px 10px rgb(0 0 0 / 20%);
		z-index: 20;
		padding: 0.5rem;
		transition: transform 0.3s ease-out;
		flex-direction: column;
		animation: slideIn 0.25s ease-out;
		animation-fill-mode: forwards;
		transform: translateX(calc(50vw - 50%));
	}
	@keyframes slideIn {
		from {
			transform: translateY(-80%) translateX(calc(50vw - 50%)) scale(0.95);
			opacity: 0;
		}
		to {
			transform: translateY(0) translateX(calc(50vw - 50%)) scale(1);
			opacity: 1;
		}
	}
	.prompt-row {
		display: flex;
		margin-top: 0.5rem;
		justify-content: space-between;
	}
	.prompt-text {
		padding: 0.3rem;
		font-size: 0.9rem;
		display: flex;
		margin-top: auto;
	}
</style>
