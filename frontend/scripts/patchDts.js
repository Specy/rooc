import fs from 'fs'

const dts = fs.readFileSync('../static/std/index.d.ts', 'utf8')

function patchDts(d){
    const start = "export function start_ROOC(): void;"
    const end = "export function end_ROOC(): void;"
    const startIndex = d.indexOf(start) + start.length
    const endIndex = d.indexOf(end)
    const between = d.slice(startIndex, endIndex)
    const newStart = start.replace(/export/g, 'declare')
    const newEnd = end.replace(/export/g, 'declare')
    const patched = d.replace(/export/g, 'declare')
    const newStartIndex = patched.indexOf(newStart) + newStart.length
    const newEndIndex = patched.indexOf(newEnd)
    const final = patched.slice(0, newStartIndex) + between + patched.slice(newEndIndex)
    return final
        .replace(newStart, "")
        .replace(newEnd, "")
        .replace("declare declare", "declare")
        .replace("declare {};", "")
        .replace("/// <reference types=\"node\" />", "")
}
fs.writeFileSync('../static/std/index.d.ts', patchDts(dts))
console.log("Patched index.d.ts")