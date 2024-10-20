<script lang="ts">

    interface Props {
        checked: boolean;
    }

    let { checked = $bindable() }: Props = $props();
</script>


<input type="checkbox" bind:checked />

