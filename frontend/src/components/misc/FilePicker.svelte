<script lang="ts">
    type inputTypes = 'text' | 'buffer' | 'dataUrl'


    type Props = {
        accept?: string;
        multiple?: boolean;
        children?: import('svelte').Snippet;
    } & ({
        as: 'text',
        onImport: (result: { data: string, file: File }[]) => void
    } | {
        as: 'buffer',
        onImport: (result: { data: ArrayBuffer, file: File }[]) => void
    } | {
        as: 'dataUrl',
        onImport: (result: { data: string, file: File }[]) => void
    })

    let {accept = '*', as = 'text', children, onImport, multiple}: Props = $props();

    let input: HTMLInputElement | null = $state(null)

    async function onChange(event: any) {
        if (event.target.files.length === 0) return
        const files = await Promise.all(Array.from(event.target.files).map(async (file: File) => {
            const fileReader = new FileReader()

            const p = new Promise<{ data: string, file: File }>((resolve, reject) => {
                fileReader.onloadend = () => {
                    resolve({data: fileReader.result as string, file})
                }
                fileReader.onerror = () => {
                    reject(fileReader.error)
                }
            })
            if (as === 'text') fileReader.readAsText(file)
            if (as === 'buffer') fileReader.readAsArrayBuffer(file)
            if (as === 'dataUrl') fileReader.readAsDataURL(file)
            return p
        }))
        input.value = ''
        onImport(files as any)
    }
</script>

<input type="file" bind:this={input} {accept} style="display: none;" onchange={onChange} multiple={multiple}/>
<div onclick={() => input?.click()} style='cursor:pointer; width:fit-content'>
    {@render children?.()}
</div>
