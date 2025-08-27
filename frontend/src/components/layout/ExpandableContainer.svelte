<script lang="ts">
    import FaChevronDown from "~icons/fa/chevron-down.svelte";
    interface Props {
        expanded?: boolean;
        disabled?: boolean;
        title?: import('svelte').Snippet;
        children?: import('svelte').Snippet;
        style?:string;
    }

    let {
        expanded = $bindable(false),
        disabled = false,
        title,
        children,
        style
    }: Props = $props();
</script>


<div class="expandable-container" class:expandable-container-open={expanded} class:disabled {style}>
    <button
            {disabled}
            onclick={() => expanded = !expanded}
            class="expandable-container-expand"
    >
        <div class="chevron-icon"
             class:chevron-icon-expanded={expanded}
             style={disabled ? "opacity: 0" : undefined}
        >
            <FaChevronDown/>
        </div>
        {@render title?.()}
    </button>
    <div class="expandable-container-content">
        {@render children?.()}
    </div>
</div>

<style>

    .expandable-container-expand {
        display: flex;
        align-items: center;
        gap: 1rem;
        padding: 0.8rem;
        background-color: transparent;
        cursor: pointer;
        color: var(--primary-text);
    }

    .expandable-container {
        display: flex;
        flex-direction: column;
        background-color: var(--primary);
        color: var(--primary-text);
        border-radius: 0.4rem;
        border: solid 0.2rem transparent;
    }
    .disabled{
        opacity: 0.5;
        background-color: var(--primary);
        pointer-events: none;
        cursor: not-allowed;
    }

    .chevron-icon {
        transition: all 0.2s;
        transform: rotate(-90deg);
    }

    .chevron-icon-expanded {
        transform: rotate(0deg);
    }

    .expandable-container-open {
        border: solid 0.2rem var(--secondary-5);
    }

    .expandable-container-content {
        display: none;
        flex-direction: column;
        border-top: solid 0.2rem var(--secondary-5);
        padding: 0.5rem 0.8rem 0.8rem;
    }

    .expandable-container-open .expandable-container-content {
        display: flex;
        animation: appear 0.2s;
    }


    @keyframes appear {
        from {
            opacity: 0;
        }
        to {
            opacity: 1;
        }
    }
</style>