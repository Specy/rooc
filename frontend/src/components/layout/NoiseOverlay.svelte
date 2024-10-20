<script lang="ts">
	interface Props {
		opacity?: number;
	}

	let { opacity = 0.05 }: Props = $props();
</script>

<div
	class="overlay"
	style="
		background-image: url('/images/noise2.png');
		opacity: {opacity}
    "
></div>

<style>
	.overlay {
		position: absolute;
		inset: 0;
		z-index: 2;
		background-size: var(--noise-size);
		pointer-events: none;
	}
</style>