<script lang="ts">
	export let opacity = 0.05;
</script>

<div
	class="overlay"
	style="
		background-image: url('/images/noise2.png');
		opacity: {opacity}
        "
/>

<style>
	.overlay {
		position: fixed;
		inset: 0;
		z-index: 2;
		background-size: var(--noise-size);
		pointer-events: none;
	}
</style>
