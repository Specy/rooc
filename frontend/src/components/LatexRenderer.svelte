<script lang='ts'>
    export let source: string;
    export let style: string = ""
    import katex from 'katex'
    $: html = katex.renderToString(source ?? "", {
        throwOnError: false,
        fleqn: true,
        displayMode: true,
        output: "htmlAndMathml"
    }) 
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
