<script lang="ts">


    import {onMount} from "svelte";
    interface Props {
        children?: import('svelte').Snippet;
    }

    let { children }: Props = $props();

    onMount(() => {
        //this page overflows often, show the scrollbar
        document.body.style.overflowY = "scroll"
        return () => {
            document.body.style.overflowY = "hidden"
        }
    })
</script>




{@render children?.()}

