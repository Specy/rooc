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
    {:else}
    <mi style="display: unset">{varName}</mi>
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
