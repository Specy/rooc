<script lang="ts">
    import Editor from "$cmp/editor/Editor.svelte"
    import { RoocParser} from "@specy/rooc"
	import type { CompilationError } from "@specy/rooc/";
    let source = ""
    let compiled = ""

    function compile(){
        const parser = new RoocParser(source)
        const transform = parser.compile()
        if(transform.ok){
            compiled = transform.val.stringify()
        }else{
            compiled = (transform.val as CompilationError).message()
        }
    }
</script>

<Editor
    language="rooc"
    bind:code={source}
    highlightedLine={-1}
/>
<button
    on:click={compile}
>
    compile
</button>

<textarea>
    {compiled}
</textarea>

