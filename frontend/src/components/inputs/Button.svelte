<script lang="ts">
    import { createBubbler } from 'svelte/legacy';

    const bubble = createBubbler();
	import type {ColorName} from '$src/stores/themeStore';

    interface Props {
        disabled?: boolean;
        style?: string;
        hasIcon?: boolean;
        color?: ColorName;
        hoverColor?: ColorName | undefined;
        border?: ColorName | undefined;
        active?: boolean;
        title?: string;
        iconLeft?: boolean;
        children?: import('svelte').Snippet;
    }

    let {
        disabled = false,
        style = '',
        hasIcon = false,
        color = 'secondary',
        hoverColor = undefined,
        border = undefined,
        active = false,
        title = '',
        iconLeft = false,
        children
    }: Props = $props();
</script>

<button
        type="button"
        class="btn"
        class:hasIcon
        class:iconLeft
        {title}
        style="
	--btn-color:var(--{color}); 
	--btn-text:var(--{color}-text); 
	--btn-color-hover:var(--{hoverColor ?? `${color}-10`});
	--border-color: var(--{border}-10);
	{style};
	"
        {disabled}
        onclick={bubble('click')}
        class:active
>
    {@render children?.()}
</button>

<style lang="scss">
  .btn {
    padding: 0.5rem 1rem;
    border-radius: 0.3rem;
    color: var(--btn-text);
    background-color: var(--btn-color);
    border: dashed 0.1rem var(--border-color, transparent);
    text-align: center;
    display: flex;
    transition: all 0.3s;
    font-size: 1em;
    align-items: center;
    justify-content: center;
    width: fit-content;
    user-select: none;
    font-family: Rubik;
    position: relative;
    cursor: pointer;
  }

  .active {
    background-color: var(--accent);
    color: var(--accent-text);
  }

  .btn:hover {
    background-color: var(--btn-color-hover);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn:disabled:hover {
    background-color: var(--btn-color) !important;
  }

  .hasIcon {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.35rem 0.6rem;
  }

  .iconLeft {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding-left: 0.6rem;
  }
</style>
