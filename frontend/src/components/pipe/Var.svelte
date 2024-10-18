<script lang="ts">
    export let value: string;


    $: [varName, ...rest] = value.split('_')
</script>

{#if rest.length}
    <msub>
        <mi>{varName}</mi>
        <mn>
            {rest.join(', ')}
        </mn>
    </msub>
{/if}

<style>
    mn{
        line-height: 0.8;
    }
</style>
