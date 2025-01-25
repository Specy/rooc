<script lang="ts">
    interface Props {
        value: string;
        style?: string;
    }

    let { value, style }: Props = $props();


    let [varName, ...rest] = $derived.by(() => {
        let clone = value
        if(clone.startsWith("_")) clone = clone.substring(1)
        return clone.split('_')
    })
</script>

{#if rest.length}
    <msub {style}>
        <mi>{varName}</mi>
        <mn>
            {rest.join(', ')}
        </mn>
    </msub>
    {:else}
    <mi style={`display:unset; ${style};`}>{varName}</mi>
{/if}

<style>
    mn{
        line-height: 0.8;
        margin-left: 1px;
    }
    mi{
        min-height: 0.9rem;
        display: flex;
        align-items: flex-end;
    }
</style>
