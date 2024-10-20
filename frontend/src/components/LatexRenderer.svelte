<script lang='ts'>
    import katex from 'katex'
    interface Props {
        source: string;
        style?: string;
    }

    let { source, style = "" }: Props = $props();
    let html = $derived(katex.renderToString(source ?? "", {
        throwOnError: false,
        fleqn: true,
        displayMode: true,
        output: "htmlAndMathml"
    })) 
</script>


<div {style}>
    {@html html}
</div>

<style lang="scss">
    :global(.katex) {
        padding-left: 0 !important; 
    }
    :global(.katex-display) {
        margin: 0 !important;
    }
</style>
