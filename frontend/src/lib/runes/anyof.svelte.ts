
export function registerDeep(data: unknown) {
    if (typeof data === 'object' && data !== null)
    for (const k in data) {
        registerDeep(data[k])
    }
}