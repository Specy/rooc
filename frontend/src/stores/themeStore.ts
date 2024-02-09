import { createThemeStorage, LocalStorageThemePersistence, type ComputedThemeLayers, type NamesOfTheme, type SerializedTheme } from "$lib/theme/svelteTheme";

/*
background: {
            hex: "#130c16",
            name: 'background',
            cssProp: 'background',
        },
        primary: {
            hex: '#1c1323',
            name: 'primary',
            cssProp: 'primary',
        },
        secondary: {
            hex: '#24192f',
            name: 'secondary',
            cssProp: 'secondary'
        },
        tertiary: {
            hex: '#322444',
            name: 'tertiary',
            cssProp: 'tertiary'
        },
 */


const baseDarkTheme = {
    meta: {
        version: 1,
        id: 'dark',
        name: 'dark'
    },
    colors: {
        background: {
            hex: "#181823",
            name: 'background',
            cssProp: 'background',
        },
        primary: {
            hex: '#222331',
            name: 'primary',
            cssProp: 'primary',
        },
        secondary: {
            hex: '#2a2b3e',
            name: 'secondary',
            cssProp: 'secondary'
        },
        tertiary: {
            hex: '#2b2d45',
            name: 'tertiary',
            cssProp: 'tertiary'
        },

        accent: {
            hex: '#b32d68',
            name: 'accent',
            cssProp: 'accent'
        },
        accent2: {
            hex: '#38454f',
            name: 'accent2',
            cssProp: 'accent2'
        },
        shadow: {
            hex: '#3b2d37',
            name: 'shadow',
            cssProp: 'shadow'
        },
        hint: {
            hex: '#939393',
            name: 'hint',
            cssProp: 'hint'
        },
        warn: {
            hex: '#ed4f4f',
            name: 'warn',
            cssProp: 'warn'
        },
        success: {
            hex: '#356a59',
            name: 'success',
            cssProp: 'success'
        },
        danger: {
            hex: '#d04434',
            name: 'danger',
            cssProp: 'danger'
        }
    }
} satisfies SerializedTheme

export type ColorName = NamesOfTheme<typeof baseDarkTheme>
export type ComputedColorNames = ComputedThemeLayers<ColorName, 5 | 10 | 15>

export const [themeStorage, currentTheme] = createThemeStorage(
    new LocalStorageThemePersistence("_app_themes_meta", "app_themes"), 
    baseDarkTheme,
)   