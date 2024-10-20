<script lang="ts">
    interface Props {
        value: string;
    }

    let { value }: Props = $props();


    let [varName, ...rest] = $derived(value.split('_'))
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
