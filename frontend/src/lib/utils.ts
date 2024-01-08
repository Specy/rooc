export type Timer = number;

export function blobDownloader(blob: Blob, fileName: string) {
    const a = document.createElement('a')
    a.style.display = 'none'
    document.body.appendChild(a)
    a.href = URL.createObjectURL(blob)
    a.download = fileName
    a.click()
    a.remove()
    URL.revokeObjectURL(a.href)
}
export function textDownloader(text: string, fileName: string) {
    blobDownloader(new Blob([text], { type: "text/json" }), fileName)
}