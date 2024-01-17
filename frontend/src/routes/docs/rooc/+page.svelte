<script lang="ts">
	import Nav from '$cmp/layout/Nav.svelte';
import Page from '$cmp/layout/Page.svelte';
	import RoocFunctionsDocs from '$cmp/roocDocs/RoocFunctionsDocs.svelte';
	import RoocSyntaxDocs from '$cmp/roocDocs/RoocSyntaxDocs.svelte';
</script>


<Nav />
<Page cropped padding='1rem' mobilePadding='0.5rem' gap="1rem">
	<RoocSyntaxDocs />
	<RoocFunctionsDocs />
</Page>
