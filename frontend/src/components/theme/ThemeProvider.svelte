<script lang="ts">
    import {createDerivedTheme, textColor, type ThemeStore} from '$lib/theme/svelteTheme';
    import {TinyColor} from '@ctrl/tinycolor';
    import type {Action} from "svelte/action";


    interface Props {
        style?: string;
        theme: ThemeStore<any>;
        children?: import('svelte').Snippet;
    }

    type CSSProperties = Record<string, string>
    export const styleAction: Action<HTMLElement, CSSProperties | string> = (
        node,
        styleData = {},
    ) => {
        // Pseudo Element for style parsing and keeping track of styles
        const pseudoElement = document.createElement('div');

        const update = (styleData: CSSProperties | string = {}) => {
            if (typeof styleData == 'string')
                pseudoElement.style.cssText = styleData;

            if (typeof styleData == 'object')
                for (const [property, value] of Object.entries(styleData)) {
                    // Do a setProperty in case it's a CSS variable
                    if (property.startsWith('--')) {
                        pseudoElement.style.setProperty(property, value);
                    } else {
                        pseudoElement.style[property] = value;
                    }
                }

            // Combine body's existing styles with computed ones
            node.style.cssText = `
					${node.style.cssText};
					${pseudoElement.style.cssText};
				`;
        };

        // Initial Update
        update(styleData);

        const unset = () => {
            // Remove the pseudoElements styles on the body
            node.style.cssText = node.style.cssText.replace(
                pseudoElement.style.cssText,
                '',
            );

            // Clear pseudoElement
            pseudoElement.style.cssText = '';
        };

        return {
            update: (styleData) => {
                unset();
                update(styleData);
            },

            destroy: unset,
        };
    };


    let {style = '', theme, children}: Props = $props();
    let themeCss = createDerivedTheme(theme, [5, 10, 15]);
    let accent = theme.getColorStore('accent');
    let background = theme.getColorStore('background');
</script>

<svelte:body
        use:styleAction={`
		--scroll-accent: ${$accent.color};
		background-color: ${$background.color};
		color: ${new TinyColor($background.color).isDark() ? textColor.light : textColor.dark};
	`}
/>

<div
        style={`
	display: flex; 
	flex-direction: column;
    ${$themeCss}
    ${style}
`}
>
    {@render children?.()}
</div>
