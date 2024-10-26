module.exports = {
    compilationOptions: {
        followSymlinks: true,
        preferredConfigPath: './tsconfig.json'
    },
    entries: [{
        filePath: './src/index.ts',
        outFile: '../static/std/index.d.ts',
        libraries: {
            inlinedLibraries: ['csv-parse', '@dagrejs/graphlib'],
            allowedTypesLibraries: ['node'],
        },
        output: {
            noBanner: true,
            sortNodes: true,
            respectPreserveConstEnum: true,
        },
    }],
}