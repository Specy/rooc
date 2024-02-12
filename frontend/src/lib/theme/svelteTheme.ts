import { TinyColor } from '@ctrl/tinycolor';
import cloneDeep from 'clone-deep'
import { get, writable, type Readable, derived } from 'svelte/store'

export type ThemeProp<T extends SerializedTheme> = {
    name: string
    color: string
    prop: ThemeKey<T>
}

export type ColorData = {
    name: string
    color: TinyColor
    hex: string
    text: string
    cssProp: string
}

export const textColor = {
    dark: '#d0d0d0',
    light: '#181818'
}

export type SerializedColorStore = {
    name: string
    cssProp: string
    hex: string
    text?: string
}

export type ThemeKey<T extends SerializedTheme> = keyof T['colors']


export function createColorStore(name: string, color: string, cssProp: string, text?: string): ColorStore {
    const _tinyColor = new TinyColor(color)
    const { subscribe, set: _set } = writable<ColorData>({
        name,
        color: _tinyColor,
        cssProp,
        hex: color,
        text: text ?? (_tinyColor.isDark() ? textColor.dark : textColor.light)
    })
    function setColor(color: string, text?: string) {
        const _tinyColor = new TinyColor(color)
        _set({
            name,
            color: _tinyColor,
            text: text ?? (_tinyColor.isDark() ? textColor.dark : textColor.light),
            cssProp,
            hex: color
        })
    }
    function layer(layer: number) {
        const color = get({ subscribe }).color
        const isDark = color.isDark()
        return isDark ? color.lighten(layer) : color.darken(layer)
    }
    function serialize(): SerializedColorStore {
        const obj = get({ subscribe })
        return {
            name,
            cssProp,
            text: obj.text,
            hex: obj.hex
        }
    }

    return {
        subscribe,
        setColor,
        serialize,
        layer
    }
}
export interface ColorStore extends Readable<ColorData> {
    setColor: (color: string, text?: string) => void
    serialize: () => SerializedColorStore
    layer: (layer: number) => TinyColor
}

export type NamesOfTheme<T extends SerializedTheme> = T extends SerializedTheme<infer K> ? K : never

export type ComputedThemeLayers<T extends string, K extends number> = T | `${T}-${K}`

export type SerializedTheme<T extends string = string> = {
    meta: {
        version: number
        id: string
        name: string
    }
    colors: Record<T, SerializedColorStore>
}
export type Theme<T extends string = string> = {
    meta: {
        version: number
        id: string
        name: string
    }
    colors: Map<T, ColorStore>
}

export function createThemeStore<T extends SerializedTheme<string>>(baseTheme: T) {

    const { subscribe, set: _set, update } = writable<Theme>({
        meta: { ...baseTheme.meta },
        colors: new Map()
    })
    function serialize(): SerializedTheme {
        const colors = Object.fromEntries(
            Array.from(
                get({ subscribe })
                    .colors
                    .entries()
            ).map(([k, v]) => [k, v.serialize()]))
        return {
            meta: get({ subscribe }).meta,
            colors
        }
    }
    function layer(key: ThemeKey<T>, layer: number) {
        return get({ subscribe }).colors.get(key as string)?.layer(layer)
    }
    function setColor(key: ThemeKey<T>, color: string, text?: string) {
        get({ subscribe }).colors.get(key as string)?.setColor(color, text)
        update(theme => theme)
    }
    function getColorStore(key: ThemeKey<T>): ColorStore | undefined {
        return get({ subscribe }).colors.get(key as string)
    }
    function getColor(key: ThemeKey<T>): TinyColor | undefined {
        const store = getColorStore(key)
        if (!store) return undefined
        return get(store).color
    }
    function getColorText(key: ThemeKey<T>): string | undefined {
        const store = getColorStore(key)
        if (!store) return undefined
        return get(store).text
    }
    function toArray(): SerializedColorStore[] {
        return Array.from(get({ subscribe }).colors.values()).map(c => c.serialize())
    }
    function toCssObject<K extends number>(layers: K[]): Record<ComputedThemeLayers<NamesOfTheme<T>, K>, string> {
        const colors = toArray();
        // Creates the different layers for each color
        const colorLayers = colors.flatMap(color =>
            layers.map(l => ({
                name: `${color.name}-${l}`,
                hex: layer(color.name, l).toHexString(),
                cssProp: `${color.cssProp}-${l}`,
                text: layer(color.name, l).isDark() ? textColor.dark : textColor.light
            } satisfies SerializedColorStore)
            )
        );
        const allColors = [...colors, ...colorLayers];
        // Merges all the colors and variables into a single object and creates the different variations of the colors
        const colorsEntries = allColors.flatMap(color => {
            const rgb = new TinyColor(color.hex).toRgb();
            const rgbArr = [rgb.r, rgb.g, rgb.b];
            return [
                [`--${color.cssProp}`, color.hex],
                [`--${color.cssProp}-text`, color.text],
                [`--${color.cssProp}-rgb`, rgbArr.join(",")]
            ];
        });
        return Object.fromEntries(colorsEntries);
    }
    function toCss<K extends number>(layers: K[]): string {
        const css = toCssObject(layers);
        return Object.entries(css).map(([k, v]) => `${k}: ${v};`).join("\n");
    }
    function loadFrom(serializedTheme: SerializedTheme) {
        const _theme = cloneDeep(serializedTheme) as SerializedTheme
        const entries = Object.entries(_theme.colors)
            .map(([k, v]) => (
                [k, createColorStore(v.name, v.hex, v.cssProp, v.text)] as [string, ColorStore]
            ))
        _set({
            meta: _theme.meta,
            colors: new Map(entries)
        })
    }

    loadFrom(baseTheme)

    return {
        subscribe,
        serialize,
        setColor,
        getColorStore,
        getColor,
        getColorText,
        layer,
        loadFrom,
        toArray,
        toCss,
        toCssObject
    }
}

export type ThemeStore<T extends SerializedTheme> = ReturnType<typeof createThemeStore<T>>
export type ThemeStorageState<T extends SerializedTheme> = {
    themes: SerializedTheme[]
    current: ThemeStore<T>
}
export type ThemeStorageMeta = {
    currentId: string
}
export type SerializedThemeStorage = {
    meta: ThemeStorageMeta,
    themes: SerializedTheme[]
}


export interface ThemePersistence {
    saveMeta: (meta: ThemeStorageMeta) => Promise<void> | void,
    saveAll: (meta: ThemeStorageMeta, themes: SerializedTheme[]) => Promise<void> | void,
    load: () => Promise<Partial<SerializedThemeStorage>> | Partial<SerializedThemeStorage>
    addTheme: (theme: SerializedTheme, storage: SerializedThemeStorage) => Promise<void> | void
    removeTheme: (theme: SerializedTheme, storage: SerializedThemeStorage) => Promise<void> | void
}



export function createThemeStorage<T extends SerializedTheme>(_persistence: ThemePersistence, baseTheme: T, ...rest: T[]) {
    let persistence = _persistence
    const currentThemeRef = createThemeStore(baseTheme)
    const { subscribe, set: _set, update } = writable<ThemeStorageState<T>>({
        themes: [baseTheme, ...rest],
        current: currentThemeRef
    })
    async function addTheme(theme: SerializedTheme) {
        const state = get({ subscribe })
        const existing = state.themes.find(t => t.meta.id === theme.meta.id)
        if (existing) throw new Error('Theme already exists')
        state.themes.push(theme)
        await persistence.addTheme(theme, serialize(state))
    }
    async function removeTheme(theme: SerializedTheme) {
        const state = get({ subscribe })
        const index = state.themes.findIndex(t => t.meta.id === theme.meta.id)
        if (index >= 0) state.themes.splice(index, 1)
        await persistence.removeTheme(theme, serialize(state))
    }
    async function setPersistence(_persistence: ThemePersistence) {
        persistence = _persistence
        await load()
    }

    async function setCurrentTheme(theme: SerializedTheme) {
        const state = get({ subscribe })
        state.current.loadFrom(theme)
        await persistence.saveMeta({
            currentId: theme.meta.id
        })

        _set(state)
    }

    async function load(replace = false) {
        const storage = await persistence.load()
        if (!storage) return
        update(currentStorage => {

            const themes = storage.themes ?? []
            if (!replace) themes.push(...currentStorage.themes)
            const meta: ThemeStorageMeta = {
                currentId: "",
                ...storage.meta
            }
            currentStorage.themes = themes
            const theme = themes.find(t => t.meta.id === meta.currentId)
            if (theme) {
                currentStorage.current.loadFrom(theme)
            }
            return currentStorage
        })
    }
    function serialize(state?: ThemeStorageState<T>): SerializedThemeStorage {
        if (!state) state = get({ subscribe })
        return {
            themes: state.themes,
            meta: {
                currentId: get(state.current).meta.id
            }
        }
    }
    return [
        {
            subscribe,
            addTheme,
            setCurrentTheme,
            removeTheme,
            load,
            setPersistence,
        },
        currentThemeRef
    ] as const
}

export class LocalStorageThemePersistence implements ThemePersistence {
    private lsMetaKey: string
    private lsThemesKey: string
    constructor(metaKey: string, themesKey: string) {
        this.lsMetaKey = metaKey
        this.lsThemesKey = themesKey
    }

    saveAll(meta: ThemeStorageMeta, themes: SerializedTheme[] | undefined) {
        localStorage.setItem(this.lsThemesKey, JSON.stringify(themes))
        localStorage.setItem(this.lsMetaKey, JSON.stringify(meta))
    }
    saveMeta(meta: ThemeStorageMeta) {
        localStorage.setItem(this.lsMetaKey, JSON.stringify(meta))
    }
    load() {
        const themes = JSON.parse((localStorage.getItem(this.lsThemesKey) ?? "null")) as SerializedTheme[] | null
        const meta = JSON.parse((localStorage.getItem(this.lsMetaKey) ?? "null")) as ThemeStorageMeta | null
        return {
            themes: themes ?? undefined,
            meta: meta ?? undefined
        }
    }
    addTheme(_: SerializedTheme, storage: SerializedThemeStorage) {
        localStorage.setItem(this.lsThemesKey, JSON.stringify(storage))
    }
    removeTheme(_: SerializedTheme, storage: SerializedThemeStorage) {
        localStorage.setItem(this.lsThemesKey, JSON.stringify(storage))
    }
}


export function createDerivedThemeColors<T extends SerializedTheme>(theme: ThemeStore<T>) {
    return derived(theme, () => {
        return theme.toArray()
    }) as Readable<SerializedColorStore[]>
}

export function createDerivedTheme<T extends SerializedTheme, K extends number >(theme: ThemeStore<T>, layers: K[]) {
    return derived(theme, () => {
        return theme.toCss(layers)
    }) 
}

