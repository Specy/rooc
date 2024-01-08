<script lang="ts">
	import Button from '$cmp/inputs/Button.svelte';
	import Page from '$cmp/layout/Page.svelte';
</script>

<Page padding="1rem" gap="1rem" contentStyle="justify-content:center; align-items:center">
	<a href="/projects" style="width: 100%; gap: 1rem;" class="container">
		<img class="logo" alt="rooc logo" src="logo.png" />
		<Button style="width: 100%">Go to your projects</Button>
	</a>
</Page>

<style lang="scss">
	.container {
		max-width: 20rem;
		width: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 1rem;
	}
	.logo {
		width: 100%;
		border-radius: 0.8rem;
		aspect-ratio: 1;
	}
</style>
