<script lang="ts">


    import {onMount} from "svelte";

    onMount(() => {
        //this page overflows often, show the scrollbar
        document.body.style.overflowY = "scroll"
        return () => {
            document.body.style.overflowY = "hidden"
        }
    })
</script>




<slot />

