<script lang="ts">
	interface Props {
		value: number;
	}

	let { value = $bindable() }: Props = $props();
</script>

<input type="number" bind:value />



<style lang="scss">
    
</style>