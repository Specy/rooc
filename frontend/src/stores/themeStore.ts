import { createThemeStorage, LocalStorageThemePersistence, type SerializedTheme } from "$lib/theme/svelteTheme";

const baseDarkTheme = {
    meta: {
        version: 1,
        id: 'dark',
        name: 'dark'
    },
    colors: {
        background: {
            hex: "#171A21",
            name: 'background',
            cssProp: 'background',
        },
        primary: {
            hex: '#171A21',
            name: 'primary',
            cssProp: 'primary',
        },
        secondary: {
            hex: '#212630',
            name: 'secondary',
            cssProp: 'secondary'
        },
        tertiary: {
            hex: '#2d3950',
            name: 'tertiary',
            cssProp: 'tertiary'
        },
        footer: {
            hex: '#212630',
            name: 'footer',
            cssProp: 'footer'
        },
        accent: {
            hex: '#ad1a5b',
            name: 'accent',
            cssProp: 'accent'
        },
        accent2: {
            hex: '#38454f',
            name: 'accent2',
            cssProp: 'accent2'
        },
        shadowColor: {
            hex: '#454559',
            name: 'shadowColor',
            cssProp: 'shadow-color'
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
        }
    }
} satisfies SerializedTheme

const baseWhiteTheme = {
    meta: {
        version: 1,
        id: 'light',
        name: 'light'
    },
    colors: {
        background: {
            hex: "#fafafa",
            name: 'background',
            cssProp: 'background',
        },
        primary: {
            hex: '#fafafa',
            name: 'primary',
            cssProp: 'primary',
        },
        secondary: {
            hex: '#f6f6f6',
            name: 'secondary',
            cssProp: 'secondary'
        },
        tertiary: {
            hex: '#2d3950',
            name: 'tertiary',
            cssProp: 'tertiary'
        },
        footer: {
            hex: '#212121',
            name: 'footer',
            cssProp: 'footer'
        },
        accent: {
            hex: '#b00752',
            name: 'accent',
            cssProp: 'accent'
        },
        shadowColor: {
            hex: '#454559',
            name: 'shadowColor',
            cssProp: 'shadow-color'
        },
        accent2: {
            hex: '#38454f',
            name: 'accent2',
            cssProp: 'accent2'
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
        }
    }
} satisfies SerializedTheme


export const [themeStorage, currentTheme] = createThemeStorage(
    new LocalStorageThemePersistence("_app_themes_meta", "app_themes"), 
    baseDarkTheme,
    baseWhiteTheme, 
)   