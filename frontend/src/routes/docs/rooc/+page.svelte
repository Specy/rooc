<script lang="ts">
    import Nav from '$cmp/layout/Nav.svelte';
    import Page from '$cmp/layout/Page.svelte';
    import RoocFunctionsDocs from '$cmp/roocDocs/RoocFunctionsDocs.svelte';
    import RoocSyntaxDocs from '$cmp/roocDocs/RoocSyntaxDocs.svelte';
    const url = `/projects/share?project=N4IgbgpgTgzglgewHYgFwEYA0I4BM0gwAWAhlBCNkiQLYWogCiAHrQA4A2F2uEMAxlDhsALohQMAgkgAEEVjU4QZNBLw6UQgiCRERckkWnQB2AAwnLAVhMAmdAE4AbFhABXNrl37Dx85ZMbe2dXfmQ9JCMGGjhZZgAdJBgAOhFkxJlMmQB6ACoZAHchPRkRImUwpJEoEliRGBly8hlc7Iys5hkAPgBeGQBPRIKmiHbM7OzC4uUyiuQYERJIhpGxmS4RAZk+9DNE3gAzWNHZLImZQ+PS8pU1CA4AcgawMjgSACMuFegTrJlOkgNABKOg0AF8gA`
</script>

<svelte:head>
    <title>
        Rooc documentation
    </title>
    <meta name="description" content="The documentation of ROOC, with syntax, function definitions and examples"/>
</svelte:head>

<Nav/>
<Page cropped padding='1rem' mobilePadding='1rem' gap="1rem">
    <RoocSyntaxDocs/>
    <RoocFunctionsDocs/>
    <a href="{url}" class="tryit">
        Try ROOC
    </a>
</Page>


<style>
    .tryit {
        border-radius: 0.8rem;
        width: 100%;
        background-color: var(--accent);
        color: var(--accent-text);
        padding: 0.5rem 1rem;
        text-align: center;
    }
</style>